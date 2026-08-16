//! Working-tree drift against the in-flight file budget (`mt check`).
//!
//! A task declares how many files it means to touch
//! (`estimate: (files: 3)`); this module reads the git working tree while the
//! work runs and reports when it has outgrown that. The intent is not to punish
//! an underestimate — an unforeseen file is normal — but to stop a session
//! tinkering onward inside a task that has outgrown itself. So a breach is
//! always a *warning*, never an error, and the message names the next move
//! rather than restating the breach.
//!
//! # Attribution
//!
//! The working tree carries one diff and nothing in it says which file belongs
//! to which task, so attribution is by count, not by path:
//!
//! - Exactly one task in flight — compare against its budget and name it.
//! - Several — compare against the *sum* of their budgets, and say so.
//!
//! "In flight" is a filter (`[tasks.estimate.drift] in-flight`), so this needs
//! no work-in-progress cap to be useful, and no status name is built in.
//!
//! # Reading the working tree
//!
//! [`working_tree_changes`] shells out to git, strictly read-only:
//! `git --no-optional-locks -C <root> status --porcelain --untracked-files=all`
//! for the changed-file count and `git … diff --numstat HEAD` for the line
//! total. `--no-optional-locks` matters — `mt check` is a read command under the
//! project's read/write split, so it must not even refresh git's index stat
//! cache. Outside a git repository, or with git unavailable, every step
//! degrades to `None`: no warning, and never an error.
//!
//! v1 compares counts only. The intended path list stays prose under the task's
//! Files heading; machine-comparing declared paths against the diff would need
//! prose parsing.
//!
//! # Excluding generated output
//!
//! `[tasks.estimate.drift] exclude` names paths the file count skips — a
//! generated output nobody hand-edits should never spend part of a budget
//! meant for the source that produced it. Each `--porcelain` line is parsed
//! back to the path it names (the destination side of a rename) and matched
//! against the configured globs; only the *file* count is filtered. The line
//! total stays unfiltered — it is already documented as a generous, distorted-
//! by-design secondary signal, never a budget of its own.

use std::fmt;
use std::path::Path;
use std::process::Command;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::{Config, DriftConfig};
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::numeric::format_num;
use crate::ready::Readiness;

/// The observed size of the git working-tree change set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChangeSet {
    /// Changed paths, tracked and untracked alike.
    pub files: usize,
    /// Added plus deleted lines across tracked changes.
    pub lines: usize,
}

/// Which budget a warning breaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftKind {
    /// The declared file count — the budget proper.
    Files,
    /// The line ceiling the file budget implies — a secondary soft signal.
    Lines,
}

/// One budget the working tree has outgrown.
#[derive(Debug, Clone, PartialEq)]
pub struct DriftWarning {
    /// Which budget was breached.
    pub kind: DriftKind,
    /// Observed count: changed files, or changed lines.
    pub observed: usize,
    /// The budget breached: the declared file count, or the line ceiling it
    /// implies.
    pub budget: f64,
    /// Ids of the in-flight tasks the budget was summed over, in load order.
    pub tasks: Vec<String>,
    /// The advice to act on — `[tasks.estimate.drift] message`, else the default.
    pub message: String,
}

impl DriftWarning {
    /// Whose budget this is: one task named outright, or a total over several.
    fn scope(&self) -> String {
        match self.tasks.as_slice() {
            [only] => format!("declared by `{only}`"),
            several => format!("declared in total by {} in-flight tasks", several.len()),
        }
    }
}

impl fmt::Display for DriftWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scope = self.scope();
        let budget = format_num(self.budget);
        let observed = self.observed;
        let message = &self.message;
        match self.kind {
            DriftKind::Files => write!(
                f,
                "working tree touches {observed} file(s), over the {budget} {scope} — {message}"
            ),
            DriftKind::Lines => write!(
                f,
                "working tree changes {observed} line(s), over the {budget} \
                 implied by the file budget {scope} — {message}"
            ),
        }
    }
}

/// The file budget currently in flight: which tasks the drift filter matched,
/// and how many files they declare between them.
#[derive(Debug, Clone, PartialEq)]
pub struct InFlightBudget {
    /// Ids of the in-flight tasks, in load order.
    pub tasks: Vec<String>,
    /// Sum of their estimates in the drift unit.
    pub files: f64,
}

impl InFlightBudget {
    /// The in-flight budget over `tasks`, or `None` when there is nothing to
    /// compare against — no task matches the in-flight filter, or none of the
    /// matched tasks declares an estimate in the drift unit.
    ///
    /// # Errors
    /// Returns [`FilterError`] when `[tasks.estimate.drift] in-flight` fails to
    /// parse or names an unknown field or view.
    pub fn of(tasks: &[Task], config: &Config) -> Result<Option<Self>, FilterError> {
        let drift = &config.tasks.estimate.drift;
        let filter = Filter::parse(&drift.in_flight)?;
        let readiness = Readiness::compute(tasks, config);
        let mut budget = Self {
            tasks: Vec::new(),
            files: 0.0,
        };
        for task in tasks {
            if !filter.matches_with(task, config, &readiness)? {
                continue;
            }
            budget.tasks.push(task.id.clone());
            budget.files += task.estimate.get(&drift.unit).copied().unwrap_or(0.0);
        }
        Ok((budget.files > 0.0).then_some(budget))
    }

    /// Compare `changes` against this budget: one warning per breached ceiling,
    /// the file budget first. The line ceiling is a generous multiple of the
    /// declared file count and is skipped entirely at `line-multiple = 0`.
    #[must_use]
    pub fn compare(&self, changes: ChangeSet, drift: &DriftConfig) -> Vec<DriftWarning> {
        let mut out = Vec::new();
        if changes.files as f64 > self.files {
            out.push(self.warning(DriftKind::Files, changes.files, self.files, drift));
        }
        let lines = self.files * drift.line_multiple as f64;
        if drift.line_multiple > 0 && changes.lines as f64 > lines {
            out.push(self.warning(DriftKind::Lines, changes.lines, lines, drift));
        }
        out
    }

    /// Build one warning against this budget's task set and advice.
    fn warning(
        &self,
        kind: DriftKind,
        observed: usize,
        budget: f64,
        drift: &DriftConfig,
    ) -> DriftWarning {
        DriftWarning {
            kind,
            observed,
            budget,
            tasks: self.tasks.clone(),
            message: drift.advice().to_owned(),
        }
    }
}

/// Compare the git working tree under `root` against the in-flight file budget.
///
/// Empty — silently — when no in-flight task declares a budget, when `root` is
/// not a git working copy, or when git cannot be run: a check that cannot see
/// the change set reports nothing rather than guessing.
///
/// # Errors
/// Returns [`FilterError`] when `[tasks.estimate.drift] in-flight` is unusable.
/// A breached budget is never an error — only a misconfigured filter is.
pub fn check_estimate_drift(
    tasks: &[Task],
    config: &Config,
    root: &Path,
) -> Result<Vec<DriftWarning>, FilterError> {
    let Some(budget) = InFlightBudget::of(tasks, config)? else {
        return Ok(Vec::new());
    };
    let Some(changes) = working_tree_changes(root, &config.tasks.estimate.drift) else {
        return Ok(Vec::new());
    };
    Ok(budget.compare(changes, &config.tasks.estimate.drift))
}

/// Read the working-tree change set under `root` with git, skipping any path
/// `drift.exclude` names from the file count.
///
/// `None` when `root` is not a git working copy or git is unavailable. A
/// repository with no commits yet has no `HEAD`, so the line total falls back to
/// zero while the file count still stands — the file budget is the real one.
#[must_use]
pub fn working_tree_changes(root: &Path, drift: &DriftConfig) -> Option<ChangeSet> {
    let status = git(root, &["status", "--porcelain", "--untracked-files=all"])?;
    let excluded = build_exclude_matcher(&drift.exclude);
    let files = status
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !path_excluded(line, excluded.as_ref()))
        .count();
    let lines = git(root, &["diff", "--numstat", "HEAD"]).map_or(0, |out| numstat_total(&out));
    Some(ChangeSet { files, lines })
}

/// Compile `[tasks.estimate.drift] exclude` into a matcher, or `None` when the
/// list is empty. A pattern that fails to compile is skipped here — invalid
/// syntax is reported at config load time by [`validate_exclude_globs`], not
/// re-reported on every `mt check` run.
fn build_exclude_matcher(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    builder.build().ok()
}

/// Whether one `--porcelain --untracked-files=all` line names a path
/// `matcher` covers. `false` with no matcher, or when the line is too short to
/// carry a path.
fn path_excluded(status_line: &str, matcher: Option<&GlobSet>) -> bool {
    let Some(matcher) = matcher else {
        return false;
    };
    let Some(path) = status_path(status_line) else {
        return false;
    };
    matcher.is_match(path)
}

/// The path one `--porcelain` line names, or `None` if the line is too short
/// to carry a two-character status code, a space, and a path. A rename line
/// (`R  old -> new`) resolves to the destination path — the one the working
/// tree now holds. Git quotes a path containing unusual bytes in a wrapping
/// `"..."`; that layer is stripped so the raw path reaches the glob matcher.
fn status_path(line: &str) -> Option<&str> {
    let rest = line.get(3..)?;
    let path = rest.rsplit(" -> ").next().unwrap_or(rest);
    Some(path.trim_matches('"'))
}

/// Validate that every `[tasks.estimate.drift] exclude` glob compiles. Returns
/// one message per pattern the glob syntax rejects, so a typo fails config
/// load instead of the pattern silently matching nothing forever.
#[must_use]
pub fn validate_exclude_globs(config: &Config) -> Vec<String> {
    config
        .tasks
        .estimate
        .drift
        .exclude
        .iter()
        .filter_map(|pattern| {
            Glob::new(pattern)
                .err()
                .map(|err| format!("tasks.estimate.drift.exclude: invalid glob {pattern:?}: {err}"))
        })
        .collect()
}

/// Run one read-only git command in `root` and return its stdout, or `None` on
/// any failure (git missing, not a repository, a non-zero exit).
///
/// `--no-optional-locks` keeps git from refreshing `.git/index`: `mt check` is a
/// read command, and a read must not write — not even a stat cache.
fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Sum added plus deleted lines over `git diff --numstat` output. A binary file
/// reports `-` for both counts and contributes nothing.
fn numstat_total(text: &str) -> usize {
    text.lines().filter_map(numstat_line).sum()
}

/// Added plus deleted lines for one `--numstat` row, or `None` when the row is
/// binary or malformed.
fn numstat_line(line: &str) -> Option<usize> {
    let mut columns = line.split_whitespace();
    let added: usize = columns.next()?.parse().ok()?;
    let deleted: usize = columns.next()?.parse().ok()?;
    Some(added + deleted)
}

#[cfg(test)]
#[path = "estimate_drift_tests.rs"]
mod tests;
