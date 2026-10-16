//! The commit a `[[commits.rules]]` rule reads: its message, the files it
//! changes, and the tasks its trailers name.
//!
//! Three steps, each separable from where its input came from:
//!
//! - [`parse_message`] reads a message's text — a hook's message file today,
//!   a past commit's `%B` for a commit range — into its subject and the
//!   values of the configured trailer.
//! - [`staged_files`] reads the index with git; the paths a past commit
//!   changed come in the same [`ChangedFiles`] shape
//!   ([`crate::commit::range`]).
//! - [`CommitRecord::build`] joins the two against the loaded tasks: trailer
//!   values become stems, and a value that is a path or names no task becomes
//!   a [`RefProblem`] instead — refused whatever rules a project declares,
//!   since a trailer that names nothing is never what its author meant.
//!
//! Trailers are parsed here rather than by `git interpret-trailers`, so the
//! parse is a pure function a test can pin: the last paragraph of the message
//! (comment lines and everything below a `git commit -v` scissors line
//! dropped) is the trailer block when every line in it is `Key: value` or an
//! indented continuation, the rule git's own parser applies to a block with no
//! non-trailer lines. Keys match case-insensitively, as git's do.
//!
//! # Files
//!
//! - `amend.rs`: which commit a `commit-msg` hook's index becomes the child of.
//! - `expr.rs`: the expression language of a `scope = "commit"` rule.
//! - `range.rs`: a range of past commits, read the way the commit being written is.
//! - `refs.rs`: which tasks shaped a file, from the trailers of its commits.
//! - `task_expr.rs`: the task half of a `scope = "task"` rule's expression.
//! - `tasks.rs`: the project as `HEAD`, the index and the working tree see it.
//! - `tip.rs`: the `scope = "tip"` rules of a range, run over its newest commit's tasks.

pub(crate) mod amend;
pub mod expr;
pub mod range;
pub mod refs;
pub mod task_expr;
pub mod tasks;
pub mod tip;

use std::path::Path;
use std::process::Command;

use thiserror::Error;

use crate::config::Config;
use crate::model::Task;
use crate::problem::Problem;
use crate::suggest::Unknown;

/// Why a commit could not be read.
#[derive(Debug, Error)]
pub enum CommitMsgError {
    /// The project root is not inside a git working tree, so there is no
    /// index to read the commit's files from.
    #[error("not a git repository: {0}")]
    NotGit(String),
    /// git ran but failed.
    #[error("git {command}: {stderr}")]
    Git {
        /// The git subcommand that failed.
        command: String,
        /// What git printed on stderr.
        stderr: String,
    },
    /// git could not be run at all.
    #[error("could not run git: {0}")]
    Spawn(#[from] std::io::Error),
}

/// One value of the configured trailer, as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrailerValue {
    /// The value after `Key:`, trimmed.
    pub value: String,
    /// Its 1-based line in the message text.
    pub line: usize,
}

/// A commit message, read: its subject and the configured trailer's values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitMessage {
    /// The first non-blank, non-comment line; empty for an empty message.
    pub subject: String,
    /// Every value of the configured trailer, in message order.
    pub trailers: Vec<TrailerValue>,
    /// Every trailer line of the block, whatever its key, as `Key: value`.
    pub trailer_lines: Vec<String>,
}

/// Read `text` as a commit message whose task trailer is `key`.
#[must_use]
pub fn parse_message(text: &str, key: &str) -> CommitMessage {
    let lines: Vec<(usize, &str)> = text
        .lines()
        .take_while(|line| !is_scissors(line))
        .enumerate()
        .filter(|(_, line)| !line.starts_with('#'))
        .map(|(index, line)| (index + 1, line))
        .collect();
    let subject = lines
        .iter()
        .map(|(_, line)| line.trim())
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_owned();
    CommitMessage {
        subject,
        trailers: trailer_values(&lines, key),
        trailer_lines: trailer_lines(&lines),
    }
}

/// `# ------------------------ >8 ------------------------`, below which
/// `git commit -v` appends the diff.
fn is_scissors(line: &str) -> bool {
    line.starts_with('#') && line.contains(">8")
}

/// The trailer block: the last paragraph, when it is not the subject's own
/// and every line in it is a trailer or a continuation. Empty otherwise.
fn trailer_block<'a>(lines: &'a [(usize, &'a str)]) -> Vec<&'a (usize, &'a str)> {
    let content: Vec<&(usize, &str)> = {
        let end = lines
            .iter()
            .rposition(|(_, line)| !line.trim().is_empty())
            .map_or(0, |last| last + 1);
        lines.iter().take(end).collect()
    };
    let start = content
        .iter()
        .rposition(|(_, line)| line.trim().is_empty())
        .map_or(content.len(), |blank| blank + 1);
    let block = content.get(start..).unwrap_or_default();
    // No blank line before the last paragraph: it is the subject's own.
    if start == content.len() || !block.iter().all(|(_, line)| is_trailer_line(line)) {
        return Vec::new();
    }
    block.to_vec()
}

/// The values of `key` in the trailer block.
fn trailer_values(lines: &[(usize, &str)], key: &str) -> Vec<TrailerValue> {
    trailer_block(lines)
        .iter()
        .filter_map(|(number, line)| {
            let (name, value) = line.split_once(':')?;
            name.trim().eq_ignore_ascii_case(key).then(|| TrailerValue {
                value: value.trim().to_owned(),
                line: *number,
            })
        })
        .collect()
}

/// Every `Key: value` line of the trailer block, continuations left out.
fn trailer_lines(lines: &[(usize, &str)]) -> Vec<String> {
    trailer_block(lines)
        .iter()
        .filter(|(_, line)| !line.starts_with(char::is_whitespace))
        .map(|(_, line)| line.trim().to_owned())
        .collect()
}

/// `Key: value` with a token key (letters, digits, `-`), or an indented
/// continuation of the trailer above.
fn is_trailer_line(line: &str) -> bool {
    if line.starts_with(char::is_whitespace) {
        return true;
    }
    line.split_once(':').is_some_and(|(name, _)| {
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

/// The paths a commit changes, as git reports them: relative to the
/// repository's top level, plus where the project root sits within it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChangedFiles {
    /// Every changed path, repository-relative, `/`-separated. A rename is
    /// its two sides: the old path deleted and the new one added.
    pub paths: Vec<String>,
    /// The old path of every rename among `paths`: the sides a count of
    /// changed files leaves out, so a rename counts once.
    pub renamed_from: Vec<String>,
    /// The project root's path within the repository (`git rev-parse
    /// --show-prefix`): empty at the top level, else `/`-terminated.
    pub root_prefix: String,
}

/// The files staged in the index of the repository holding `root`: what the
/// commit a `commit-msg` hook guards is about to record.
///
/// While `git commit --amend` makes the commit, "staged" is judged against
/// `HEAD^`, the parent the amended commit will have.
///
/// Read-only (`--no-optional-locks`), and it inherits `GIT_INDEX_FILE`, so
/// under `git commit -a` or `git commit <paths>` it reads the temporary index
/// the commit is actually built from.
///
/// # Errors
/// [`CommitMsgError::NotGit`] when `root` is not inside a git working tree;
/// [`CommitMsgError::Git`] / [`CommitMsgError::Spawn`] when git fails.
pub fn staged_files(root: &Path) -> Result<ChangedFiles, CommitMsgError> {
    let root_prefix = git(root, &["rev-parse", "--show-prefix"]).map_err(|err| match err {
        CommitMsgError::Git { .. } => CommitMsgError::NotGit(root.display().to_string()),
        other => other,
    })?;
    let revs = index_revs(root)?;
    let revs: Vec<&str> = revs.iter().map(String::as_str).collect();
    Ok(ChangedFiles {
        paths: changed_paths(root, &revs)?,
        renamed_from: renamed_sources(root, &revs)?,
        root_prefix: root_prefix.trim().to_owned(),
    })
}

/// The `git diff` arguments naming the index against what the commit follows:
/// plain `--cached` (`HEAD`, which a repository with no commit yet lacks), or
/// `--cached <parent>` while amending ([`crate::commit::amend`]).
pub(crate) fn index_revs(root: &Path) -> Result<Vec<String>, CommitMsgError> {
    let base = crate::commit::amend::base_rev(root)?;
    Ok(if base == "HEAD" {
        vec!["--cached".to_owned()]
    } else {
        vec!["--cached".to_owned(), base]
    })
}

/// The repository-relative paths `git diff <revs>` names: renames split into
/// their two sides, top-level-relative whatever the root's prefix is, so
/// `files` reads the same from any project inside the repository.
pub(crate) fn changed_paths(root: &Path, revs: &[&str]) -> Result<Vec<String>, CommitMsgError> {
    let mut args = vec!["diff", "--name-only", "--no-renames", "--no-relative", "-z"];
    args.extend_from_slice(revs);
    args.extend_from_slice(&["--", ":/"]);
    Ok(git(root, &args)?
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

/// The old path of every rename `git diff -M <revs>` detects, top-level
/// relative like [`changed_paths`]; the new side is already in that list.
pub(crate) fn renamed_sources(root: &Path, revs: &[&str]) -> Result<Vec<String>, CommitMsgError> {
    let mut args = vec![
        "diff",
        "-M",
        "--diff-filter=R",
        "--name-status",
        "--no-relative",
        "-z",
    ];
    args.extend_from_slice(revs);
    args.extend_from_slice(&["--", ":/"]);
    let out = git(root, &args)?;
    // Records are `R<score> NUL old NUL new NUL`.
    let mut fields = out.split('\0').filter(|field| !field.is_empty());
    let mut sources = Vec::new();
    while let Some(status) = fields.next() {
        let old = fields.next();
        let new = fields.next();
        if let (true, Some(old), Some(_)) = (status.starts_with('R'), old, new) {
            sources.push(old.to_owned());
        }
    }
    Ok(sources)
}

/// Run one read-only git command in `root` and return its stdout.
pub(crate) fn git(root: &Path, args: &[&str]) -> Result<String, CommitMsgError> {
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(CommitMsgError::Git {
            command: args.first().copied().unwrap_or_default().to_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        })
    }
}

/// A trailer value no task answers to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefProblem {
    /// The value as written.
    pub value: TrailerValue,
    /// What to write instead.
    pub message: String,
    /// The nearest task, when the value names none.
    pub unknown: Option<crate::suggest::Unknown>,
}

/// The commit a `scope = "commit"` rule reads — the fields its expressions
/// name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitRecord {
    /// `subject`: the message's first line.
    pub subject: String,
    /// `refs`: the stems the configured trailer names, one per value, a path
    /// read as its stem.
    pub refs: Vec<String>,
    /// `files`: every changed path, repository-relative.
    pub files: Vec<String>,
    /// The old path of each rename among `files`; `count(files, …)` skips
    /// them, `has` and `matches` still see them.
    pub renamed_from: Vec<String>,
    /// `tasks_only`: the commit changes at least one file, and every one is a
    /// task file (a `.typ` under a `[tasks] roots` entry).
    pub tasks_only: bool,
    /// `in_range`: the commit was read from a range (`mt check --commits`),
    /// not from a message being written (`mt check --commit-msg`).
    pub in_range: bool,
    /// The stems a `scope = "task"` rule runs for: every task file the commit
    /// changes, then every ref, each once. Not an expression field.
    pub tasks: Vec<String>,
    /// `trailers`: every trailer line of the message, whatever its key, as
    /// `Key: value`.
    pub trailers: Vec<String>,
}

impl CommitRecord {
    /// Join a read message and its changed files against the loaded `tasks`.
    /// Returns the record, plus one [`RefProblem`] per trailer value that is a
    /// path or names no task.
    #[must_use]
    pub fn build(
        message: &CommitMessage,
        changed: &ChangedFiles,
        tasks: &[Task],
        config: &Config,
    ) -> (Self, Vec<RefProblem>) {
        let stems: Vec<String> = tasks.iter().map(task_stem).collect();
        let mut problems = Vec::new();
        let mut refs = Vec::new();
        for trailer in &message.trailers {
            let stem = value_stem(&trailer.value);
            if let Some(found) = ref_problem(&trailer.value, &stem, &stems, &config.commits.trailer)
            {
                problems.push(RefProblem {
                    value: trailer.clone(),
                    message: found.text,
                    unknown: found.unknown,
                });
            }
            refs.push(stem);
        }
        let tasks_only = !changed.paths.is_empty()
            && changed
                .paths
                .iter()
                .all(|path| is_task_file(path, &changed.root_prefix, config));
        let mut subjects: Vec<String> = changed
            .paths
            .iter()
            .filter(|path| is_task_file(path, &changed.root_prefix, config))
            .map(|path| value_stem(path))
            .chain(refs.iter().cloned())
            .collect();
        dedup_keeping_order(&mut subjects);
        let record = Self {
            subject: message.subject.clone(),
            refs,
            files: changed.paths.clone(),
            renamed_from: changed.renamed_from.clone(),
            tasks_only,
            in_range: false,
            tasks: subjects,
            trailers: message.trailer_lines.clone(),
        };
        (record, problems)
    }
}

/// Drop every repeat of an earlier entry.
fn dedup_keeping_order(values: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    values.retain(|value| seen.insert(value.clone()));
}

/// A task's stem: its file name without `.typ` — the identity a trailer names.
pub(crate) fn task_stem(task: &Task) -> String {
    task.file.file_stem().map_or_else(
        || task.id.clone(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

/// The stem a trailer value names: the value itself, or a path's file stem.
pub(crate) fn value_stem(value: &str) -> String {
    Path::new(value).file_stem().map_or_else(
        || value.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

/// What is wrong with one trailer value, if anything.
fn ref_problem(value: &str, stem: &str, stems: &[String], key: &str) -> Option<Problem> {
    let known = stems.iter().any(|known| known == stem);
    if value.contains(['/', '\\']) || value.ends_with(".typ") {
        let fix = if known {
            format!("write the stem: `{key}: {stem}`")
        } else {
            format!("write a stem, and `{stem}` names no task")
        };
        return Some(format!("`{value}` is a path; {fix}").into());
    }
    (!known).then(|| {
        Problem::new(format!("`{value}` names no task")).with(Unknown::near(
            value,
            &stems.iter().map(String::as_str).collect::<Vec<_>>(),
        ))
    })
}

/// Whether repository-relative `path` is a task file of this project.
pub(crate) fn is_task_file(path: &str, root_prefix: &str, config: &Config) -> bool {
    let Some(rel) = path.strip_prefix(root_prefix) else {
        return false;
    };
    rel.ends_with(".typ")
        && config.tasks.roots.iter().any(|root| {
            let dir = root.path().trim_start_matches("./").trim_end_matches('/');
            dir.is_empty()
                || dir == "."
                || rel.strip_prefix(dir).is_some_and(|r| r.starts_with('/'))
        })
}

#[cfg(test)]
mod tests;
