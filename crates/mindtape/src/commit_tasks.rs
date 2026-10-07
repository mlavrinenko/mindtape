//! The project as a commit sees it: its tasks as of `HEAD`, the index and the
//! working tree — the three states a `scope = "task"` commit rule reads
//! through `head(...)`, `staged(...)` and `worktree(...)`.
//!
//! One load, three views. The loaded project is the working tree; each other
//! state is that load with the task files it differs at swapped for its own
//! versions, read from git in one `git cat-file --batch` and evaluated like a
//! file that is not on disk ([`crate::validate::candidate`]). A file the state lacks is
//! absent from it, so `exists` reads false there.
//!
//! Where a state's versions come from is a [`StateSource`]: a git revision and
//! the paths it may differ at. [`StatePlan::index`] builds the hook's plan —
//! the index as `staged`, `HEAD` as `head`, the working tree kept. A plan for
//! a past commit ([`crate::commit_range`]) swaps the sources — the commit as
//! `staged`, its first parent as `head`, no working tree — and loads through
//! the same [`CommitStates::load`].

use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::commit_msg::{CommitMsgError, changed_paths, git, task_stem};
use crate::config::Config;
use crate::git_state::GitStatus;
use crate::loader::LoadError;
use crate::model::Task;
use crate::ready::Readiness;
use crate::validate::candidate::{Candidate, harvest_source};

/// Where one state's versions of the files it differs at come from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateSource {
    /// The revision `git cat-file` reads `<rev>:<path>` from; empty reads the
    /// index (`:<path>`).
    pub rev: String,
    /// Every repository-relative path at which this state may differ from the
    /// loaded working tree. A path that is not a task file is ignored.
    pub differs: Vec<String>,
}

/// Which states a commit is checked against, and where each comes from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatePlan {
    /// The project root's path within the repository (`git rev-parse
    /// --show-prefix`): empty at the top level, else `/`-terminated.
    pub root_prefix: String,
    /// The state before the commit.
    pub head: StateSource,
    /// The state the commit records.
    pub staged: StateSource,
    /// Whether the loaded working tree is a state of its own — false for a
    /// past commit, which has none.
    pub worktree: bool,
}

impl StatePlan {
    /// The plan a `commit-msg` hook checks: the index is what the commit
    /// records, `HEAD` what it follows, and the working tree stays readable.
    /// One `git status` names every path either differs at.
    ///
    /// # Errors
    /// [`CommitMsgError`] when git cannot answer.
    pub fn index(root: &Path) -> Result<Self, CommitMsgError> {
        let root_prefix = git(root, &["rev-parse", "--show-prefix"])?;
        let status = GitStatus::read_unpaired(root).map_err(|stderr| CommitMsgError::Git {
            command: "status".to_owned(),
            stderr,
        })?;
        // `HEAD`, or its parent while `git commit --amend` replaces it; the
        // paths where that differs from `HEAD` join the ones `status` names.
        let base = crate::commit_amend::base_rev(root)?;
        let behind_head = if base == "HEAD" {
            Vec::new()
        } else {
            changed_paths(root, &[&base, "HEAD"])?
        };
        let paths = |pick: fn(bool, bool) -> bool| {
            status
                .entries()
                .iter()
                .filter(|entry| pick(entry.staged, entry.worktree))
                .map(|entry| entry.path.clone())
                .collect()
        };
        Ok(Self {
            root_prefix: root_prefix.trim().to_owned(),
            // The index against the working tree.
            staged: StateSource {
                rev: String::new(),
                differs: paths(|_, worktree| worktree),
            },
            // The base differs from the working tree wherever either step does.
            head: StateSource {
                rev: base,
                differs: paths(|staged, worktree| staged || worktree)
                    .into_iter()
                    .chain(behind_head)
                    .collect::<BTreeSet<String>>()
                    .into_iter()
                    .collect(),
            },
            worktree: true,
        })
    }
}

/// One state of the project: its tasks, and the readiness computed over them
/// for `ready`, `blocked` and `count(...)`.
#[derive(Debug, Clone, Default)]
pub struct ProjectState {
    /// Every task the state holds, in id order.
    pub tasks: Vec<Task>,
    /// Readiness over [`Self::tasks`].
    pub readiness: Readiness,
    /// Task files of this state that failed to evaluate — absent from
    /// [`Self::tasks`].
    pub errors: Vec<LoadError>,
}

impl ProjectState {
    /// A state holding `tasks`, ranked and classified as a load would.
    pub(crate) fn new(mut tasks: Vec<Task>, config: &Config, errors: Vec<LoadError>) -> Self {
        tasks.sort_by(|left, right| left.id.cmp(&right.id));
        crate::dep_rank::annotate(&mut tasks, config);
        let readiness = Readiness::compute(&tasks, config);
        Self {
            tasks,
            readiness,
            errors,
        }
    }

    /// A one-task state holding [`Task::default`], whose stem is empty — the
    /// load-time dry run's.
    pub(crate) fn probe(config: &Config) -> Self {
        Self::new(vec![Task::default()], config, Vec::new())
    }

    /// The task whose file stem is `stem`, if this state holds one.
    #[must_use]
    pub fn task(&self, stem: &str) -> Option<&Task> {
        self.tasks.iter().find(|task| task_stem(task) == stem)
    }
}

/// The states a commit is checked against.
#[derive(Debug, Clone, Default)]
pub struct CommitStates {
    /// The project before the commit.
    pub head: ProjectState,
    /// The project the commit records.
    pub staged: ProjectState,
    /// The project on disk, when there is one to read.
    pub worktree: Option<ProjectState>,
}

impl CommitStates {
    /// Build every state `plan` names as overlays of `base`, the loaded
    /// working tree.
    ///
    /// # Errors
    /// [`CommitMsgError`] when git cannot read a state's files.
    pub fn load(
        root: &Path,
        base: &[Task],
        config: &Config,
        plan: &StatePlan,
    ) -> Result<Self, CommitMsgError> {
        let overlay = |source: &StateSource| overlay(root, base, config, plan, source);
        Ok(Self {
            head: overlay(&plan.head)?,
            staged: overlay(&plan.staged)?,
            worktree: plan
                .worktree
                .then(|| ProjectState::new(base.to_vec(), config, Vec::new())),
        })
    }
}

/// `base` with every task file `source` differs at replaced by the version
/// `source` holds, or dropped where it holds none.
fn overlay(
    root: &Path,
    base: &[Task],
    config: &Config,
    plan: &StatePlan,
    source: &StateSource,
) -> Result<ProjectState, CommitMsgError> {
    let files: Vec<(&str, PathBuf)> = source
        .differs
        .iter()
        .filter(|path| crate::commit_msg::is_task_file(path, &plan.root_prefix, config))
        .filter_map(|path| {
            let rel = path.strip_prefix(&plan.root_prefix)?;
            Some((path.as_str(), PathBuf::from(rel)))
        })
        .collect();
    let mut tasks: Vec<Task> = base
        .iter()
        .filter(|task| !files.iter().any(|(_, rel)| *rel == task.file))
        .cloned()
        .collect();
    let specs: Vec<String> = files
        .iter()
        .map(|(path, _)| format!("{}:{path}", source.rev))
        .collect();
    let mut errors = Vec::new();
    for ((_, rel), blob) in files.iter().zip(read_blobs(root, &specs)?) {
        let Some(text) = blob else { continue };
        let candidate = Candidate { rel, source: &text };
        match harvest_source(root, config, candidate) {
            Ok(Some(task)) => tasks.push(task),
            Ok(None) => {}
            Err(err) => errors.push(err),
        }
    }
    Ok(ProjectState::new(tasks, config, errors))
}

/// Each object `specs` names, in order, through one `git cat-file --batch`:
/// its text, or `None` where the revision holds no such path.
fn read_blobs(root: &Path, specs: &[String]) -> Result<Vec<Option<String>>, CommitMsgError> {
    if specs.is_empty() {
        return Ok(Vec::new());
    }
    let mut child = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let input: String = specs.iter().map(|spec| format!("{spec}\n")).collect();
    let mut stdin = child.stdin.take();
    // Written from its own thread: git answers as it reads, so a large batch
    // written inline would fill the output pipe before the input drains.
    let writer = std::thread::spawn(move || {
        stdin
            .as_mut()
            .map_or(Ok(()), |pipe| pipe.write_all(input.as_bytes()))
    });
    let mut out = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_end(&mut out)?;
    }
    let output = child.wait_with_output()?;
    writer.join().unwrap_or(Ok(()))?;
    if !output.status.success() {
        return Err(CommitMsgError::Git {
            command: "cat-file".to_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    Ok(parse_batch(&out, specs.len()))
}

/// Split `git cat-file --batch` output into `count` objects: `<oid> <type>
/// <size>\n<content>\n` each, or `<spec> missing\n` for an absent one.
fn parse_batch(mut out: &[u8], count: usize) -> Vec<Option<String>> {
    let mut blobs = Vec::with_capacity(count);
    while blobs.len() < count {
        let Some(end) = out.iter().position(|byte| *byte == b'\n') else {
            break;
        };
        let header = String::from_utf8_lossy(out.get(..end).unwrap_or_default());
        out = out.get(end + 1..).unwrap_or_default();
        let size = header
            .rsplit(' ')
            .next()
            .and_then(|size| size.parse::<usize>().ok())
            .filter(|_| header.split(' ').count() == 3);
        let Some(size) = size else {
            blobs.push(None);
            continue;
        };
        let body = out.get(..size).unwrap_or_default();
        blobs.push(Some(String::from_utf8_lossy(body).into_owned()));
        out = out.get(size + 1..).unwrap_or_default();
    }
    blobs.resize(count, None);
    blobs
}

#[cfg(test)]
#[path = "commit_tasks_tests.rs"]
mod tests;
