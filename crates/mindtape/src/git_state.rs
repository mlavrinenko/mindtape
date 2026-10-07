//! What git says has changed, per state, read on demand and strictly
//! read-only.
//!
//! A change sits in one or both of two states: `staged` (the index differs
//! from `HEAD`) and `worktree` (the working tree differs from the index,
//! untracked paths included). One `git status --porcelain -z
//! --untracked-files=all` answers both — its `X` column is the staged side,
//! its `Y` column the worktree side — so a command runs it at most once,
//! through the [`GitCache`] its [`crate::config::Config`] carries.
//!
//! Consumers: the filter language's `changed(<states>, <glob>…)`
//! ([`crate::filter`]), and the commit states a `scope = "task"` commit rule
//! reads ([`crate::commit_tasks`]). Paths are relative to the repository root, whichever directory git is run from.
//!
//! Every git call passes `--no-optional-locks`: reading must not even refresh
//! `.git/index`'s stat cache, because the commands that read are read commands
//! under the project's read/write split.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock};

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::dicts::{DictCase, Dicts};

/// The state names `changed()` accepts, in the order an error lists them.
const STATE_NAMES: [&str; 2] = ["staged", "worktree"];

/// Which states a count reads: parsed from `staged`, `worktree`, or both
/// joined with `+`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateSet {
    /// The index against `HEAD`.
    pub staged: bool,
    /// The working tree against the index, untracked paths included.
    pub worktree: bool,
}

impl StateSet {
    /// Both states — every change git can see.
    pub const ALL: Self = Self {
        staged: true,
        worktree: true,
    };

    /// Parse `staged`, `worktree`, or a `+`-join of them.
    ///
    /// # Errors
    /// A message naming the unknown part and the known state names.
    pub fn parse(spec: &str) -> Result<Self, String> {
        let mut set = Self {
            staged: false,
            worktree: false,
        };
        for part in spec.split('+') {
            match part.trim() {
                "staged" => set.staged = true,
                "worktree" => set.worktree = true,
                other => {
                    return Err(format!(
                        "unknown git state {other:?} in {spec:?}; expected {} or both joined with `+`",
                        STATE_NAMES.join(" or ")
                    ));
                }
            }
        }
        Ok(set)
    }
}

/// One changed path and the states it changed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    /// Repository-root-relative path; a rename's destination.
    pub path: String,
    /// Changed in the index against `HEAD`.
    pub staged: bool,
    /// Changed in the working tree against the index, or untracked.
    pub worktree: bool,
}

impl StatusEntry {
    /// Whether this path changed in any state `states` names.
    #[must_use]
    pub const fn is_in(&self, states: StateSet) -> bool {
        (states.staged && self.staged) || (states.worktree && self.worktree)
    }
}

/// Every changed path git reports, one entry per path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitStatus {
    entries: Vec<StatusEntry>,
}

impl GitStatus {
    /// Run `git status` under `root`.
    ///
    /// # Errors
    /// The reason git gave — `not a git repository`, or that git could not
    /// be run at all.
    pub fn read(root: &Path) -> Result<Self, String> {
        git(
            root,
            &["status", "--porcelain", "-z", "--untracked-files=all"],
        )
        .map(|out| Self::parse(&out))
    }

    /// [`Self::read`] with every rename split into its two sides — the source
    /// deleted, the destination added — for a reader that needs every path a
    /// state differs at, not the path the tree now holds
    /// ([`crate::commit_tasks`]).
    ///
    /// # Errors
    /// As [`Self::read`].
    pub fn read_unpaired(root: &Path) -> Result<Self, String> {
        git(
            root,
            &[
                "status",
                "--porcelain",
                "-z",
                "--untracked-files=all",
                "--no-renames",
            ],
        )
        .map(|out| Self::parse(&out))
    }

    /// Parse `git status --porcelain -z` output. A rename or copy record is
    /// followed by a record holding its source path, which is skipped: the
    /// change counts against the path the tree now holds. A record too short
    /// to carry a path is dropped.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut entries = Vec::new();
        let mut records = text.split('\0');
        while let Some(record) = records.next() {
            let (Some(code), Some(path)) = (record.get(..2), record.get(3..)) else {
                continue;
            };
            let mut columns = code.chars();
            let (x, y) = (columns.next().unwrap_or(' '), columns.next().unwrap_or(' '));
            if matches!(x, 'R' | 'C') {
                records.next();
            }
            entries.push(StatusEntry {
                path: path.to_owned(),
                staged: !matches!(x, ' ' | '?' | '!'),
                worktree: y != ' ',
            });
        }
        Self { entries }
    }

    /// Every entry, in git's order.
    #[must_use]
    pub fn entries(&self) -> &[StatusEntry] {
        &self.entries
    }

    /// The paths changed in any of `states`, each once.
    pub fn paths(&self, states: StateSet) -> impl Iterator<Item = &str> {
        self.entries
            .iter()
            .filter(move |entry| entry.is_in(states))
            .map(|entry| entry.path.as_str())
    }
}

/// Which paths count: include globs, and `!`-prefixed exclude globs. No
/// include admits every path; an exclude wins over an include.
#[derive(Debug, Clone)]
pub struct PathFilter {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl PathFilter {
    /// Compile `globs`, a leading `!` marking an exclude.
    ///
    /// # Errors
    /// A message naming the first glob that fails to compile.
    pub fn new(globs: &[String]) -> Result<Self, String> {
        let (excludes, includes): (Vec<&str>, Vec<&str>) = globs
            .iter()
            .map(String::as_str)
            .partition(|glob| glob.starts_with('!'));
        let excludes: Vec<&str> = excludes
            .iter()
            .map(|glob| glob.trim_start_matches('!'))
            .collect();
        Ok(Self {
            include: glob_set(&includes)?,
            exclude: glob_set(&excludes)?,
        })
    }

    /// [`Self::new`] after each `@name` glob is replaced by the entries of the
    /// `[dicts.<name>]` set it names, so a project writes its path set once and
    /// every `changed()` and `count(files, …)` call reads it.
    ///
    /// # Errors
    /// A message naming an unknown set, a set that folds case (which would
    /// lowercase its globs), or the first glob that fails to compile.
    pub fn resolve(globs: &[String], dicts: &Dicts) -> Result<Self, String> {
        let mut expanded = Vec::new();
        for glob in globs {
            let Some(name) = glob.strip_prefix('@') else {
                expanded.push(glob.clone());
                continue;
            };
            let dict = dicts.get(name).ok_or_else(|| {
                let known: Vec<String> = dicts.names().map(ToOwned::to_owned).collect();
                format!(
                    "unknown path set `@{name}`; declare it as [dicts.{name}]{}",
                    crate::suggest::hint("dict", name, &known)
                )
            })?;
            if dict.case() != DictCase::Exact {
                return Err(format!(
                    "path set `@{name}` folds case; declare it with case = \"exact\""
                ));
            }
            expanded.extend(dict.entries().map(ToOwned::to_owned));
        }
        Self::new(&expanded)
    }

    /// Whether `path` counts.
    #[must_use]
    pub fn admits(&self, path: &str) -> bool {
        self.include.as_ref().is_none_or(|set| set.is_match(path))
            && !self.exclude.as_ref().is_some_and(|set| set.is_match(path))
    }

    /// How many of `paths` count, a rename once: a path in `renamed_from` is
    /// the source side of a rename whose destination `paths` also lists, so it
    /// is left out. `changed()` and `count(files, …)` both count through here.
    pub fn count<'a>(
        &self,
        paths: impl IntoIterator<Item = &'a str>,
        renamed_from: &[String],
    ) -> usize {
        paths
            .into_iter()
            .filter(|path| self.admits(path) && !renamed_from.iter().any(|from| from == path))
            .count()
    }
}

/// Compile `patterns` into one set, or `None` when there are none.
fn glob_set(patterns: &[&str]) -> Result<Option<GlobSet>, String> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|err| format!("invalid glob {pattern:?}: {err}"))?;
        builder.add(glob);
    }
    builder
        .build()
        .map(Some)
        .map_err(|err| format!("invalid globs: {err}"))
}

/// The [`GitStatus`] of one project root, read the first time something asks
/// and shared by every clone after. Carried on [`crate::config::Config`] so a
/// filter reads git without a `root` parameter, like
/// [`crate::rules::RulesRoot`]. Empty root (a config built without a
/// project) reads the process's current directory. Never equal-compared.
#[derive(Debug, Clone, Default)]
pub struct GitCache {
    root: PathBuf,
    status: Arc<OnceLock<Result<GitStatus, String>>>,
    refs: crate::commit_refs::RefsCache,
}

impl PartialEq for GitCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl GitCache {
    /// Point the cache at `root`, dropping anything read before.
    pub fn set_root(&mut self, root: &Path) {
        self.root = root.to_owned();
        self.status = Arc::default();
        self.refs = crate::commit_refs::RefsCache::default();
    }

    /// The root's git status, read at most once.
    ///
    /// # Errors
    /// Why git could not answer (see [`GitStatus::read`]).
    pub fn status(&self) -> Result<&GitStatus, String> {
        self.status
            .get_or_init(|| GitStatus::read(&self.root))
            .as_ref()
            .map_err(Clone::clone)
    }

    /// The tasks the commits changing `path` name, `set` being the loaded
    /// tasks when there are any — `touches()`'s answer, read from `git log`
    /// at most once ([`crate::commit_refs`]).
    ///
    /// # Errors
    /// Why git could not answer (see [`GitStatus::read`]).
    pub(crate) fn touching(
        &self,
        path: &str,
        config: &crate::config::Config,
        set: Option<&[crate::model::Task]>,
    ) -> Result<Arc<crate::commit_refs::Touched>, String> {
        self.refs.touching(&self.root, path, config, set)
    }

    /// Every stem a `touches()` answer met that no loaded task answers to,
    /// even through a rename: the retired tasks, sorted.
    #[must_use]
    pub(crate) fn retired_refs(&self) -> Vec<String> {
        self.refs.retired()
    }
}

/// Run one read-only git command in `root` and return its stdout.
///
/// # Errors
/// Git's own first stderr line on a non-zero exit, or why it could not start.
pub(crate) fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|err| format!("git could not be run: {err}"))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let reason = stderr.lines().next().unwrap_or("git failed").trim();
    Err(reason.trim_start_matches("fatal: ").to_owned())
}

#[cfg(test)]
#[path = "git_state_tests.rs"]
mod tests;
