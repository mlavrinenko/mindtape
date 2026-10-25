//! `[tasks.comments]` — policy over review comments left in a task's Typst
//! source.
//!
//! The workflow this guards: an agent files a task, a human leaves `// no, use
//! X instead` inline, the agent is meant to reframe the task and remove the
//! note. Without a gate the comment silently becomes part of the artifact — it
//! survives into git and the next reader cannot tell a live objection from a
//! settled one.
//!
//! Detection runs on the parse plane ([`typst_cst::parse_comments`]), so a
//! `//` inside a string literal or a raw block is never reported — see that
//! crate's own tests, which pin both. [`check_comments`] scans a task's whole
//! source file, not only the body: an annotation dropped inside
//! `task.with(...)` is the same problem.

use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Deserialize;
use thiserror::Error;
use typst_cst::{Comment, CommentKind, parse_comments};

use crate::config::Config;
use crate::model::Task;
use crate::validate::severity::Severity;
use crate::validate::{Location, Violation};

/// Policy for review comments anywhere in a task's source file.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct CommentsConfig {
    /// Whether a comment is unconstrained, reported, or fails `mt check`.
    pub policy: CommentsPolicy,
    /// Regex that permits a deliberate comment form even under `warn` or
    /// `forbid`. It matches the comment's stripped, trimmed inner text.
    pub allow_pattern: Option<String>,
}

/// How `mt check` treats a comment found in a task's source.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CommentsPolicy {
    /// Comments are unconstrained. The default.
    #[default]
    Allow,
    /// A comment is reported but `mt check` still succeeds.
    Warn,
    /// A comment is reported and fails `mt check`.
    Forbid,
}

impl CommentsPolicy {
    /// The severity a found comment carries at this policy: `forbid` is an
    /// error, everything else a warning.
    #[must_use]
    pub const fn severity(self) -> Severity {
        if self.blocks() {
            Severity::Error
        } else {
            Severity::Warn
        }
    }

    /// Whether a found comment, at this policy, fails `mt check`.
    #[must_use]
    pub const fn blocks(self) -> bool {
        matches!(self, Self::Forbid)
    }
}

/// A task's source file could not be read while scanning for comments.
#[derive(Debug, Error)]
#[error("read {path}: {source}")]
pub struct CommentsError {
    /// The file that failed to read.
    pub path: PathBuf,
    #[source]
    source: std::io::Error,
}

/// Scan every task's whole raw Typst source for comments under
/// `[tasks.comments]`.
///
/// `policy = "allow"` (the default) skips scanning entirely. `"warn"` and
/// `"forbid"` both report every comment whose text does not match
/// `allow-pattern` — [`CommentsPolicy::blocks`] is what tells the caller
/// whether to treat the returned violations as failing `mt check`.
///
/// # Errors
/// Returns [`CommentsError`] if a task's source file cannot be read.
pub fn check_comments(
    tasks: &[Task],
    config: &Config,
    root: &Path,
) -> Result<Vec<Violation>, CommentsError> {
    let settings = &config.tasks.comments;
    if matches!(settings.policy, CommentsPolicy::Allow) {
        return Ok(Vec::new());
    }
    let allow = settings
        .allow_pattern
        .as_deref()
        .and_then(|pattern| Regex::new(pattern).ok());
    let mut out = Vec::new();
    for task in tasks {
        let abs = root.join(&task.file);
        let source = std::fs::read_to_string(&abs).map_err(|source| CommentsError {
            path: abs.clone(),
            source,
        })?;
        out.extend(task_violations(
            task,
            &source,
            allow.as_ref(),
            settings.policy.severity(),
        ));
    }
    Ok(out)
}

/// Violations for one task's already-read source: every comment not exempted
/// by `allow`.
fn task_violations(
    task: &Task,
    source: &str,
    allow: Option<&Regex>,
    severity: Severity,
) -> Vec<Violation> {
    parse_comments(source)
        .into_iter()
        .filter(|comment| !allow.is_some_and(|re| re.is_match(&comment.text)))
        .map(|comment| to_violation(task, &comment, severity))
        .collect()
}

/// One comment as a [`Violation`]: file and line via [`Location`], the
/// comment's own text in the message, so the offending note is visible in the
/// report rather than requiring a hunt.
fn to_violation(task: &Task, comment: &Comment, severity: Severity) -> Violation {
    Violation {
        task_id: task.id.clone(),
        field: "comments".to_owned(),
        message: format!("{} comment: \"{}\"", shape(&comment.kind), comment.text),
        location: Location::at(task.file.clone(), comment.line),
        severity,
        kind: crate::validate::violation::ViolationKind::Other,
        reads: Some(crate::validate::violation::reads_of_field("comments")),
        unknown: None,
        detail: None,
        dict_miss: None,
    }
}

/// The diagnostic label for a comment's shape.
fn shape(kind: &CommentKind) -> &'static str {
    match kind {
        CommentKind::Line => "line",
        CommentKind::Block => "block",
    }
}

/// Config-level problem with `[tasks.comments] allow-pattern`, if it fails to
/// compile as a regex. Folded into [`Config::validate`] so a malformed pattern
/// fails at load, not silently at the far end of `mt check`.
pub(crate) fn validate_comments(config: &Config) -> Vec<String> {
    let Some(pattern) = config.tasks.comments.allow_pattern.as_deref() else {
        return Vec::new();
    };
    Regex::new(pattern).err().map_or_else(Vec::new, |err| {
        vec![format!(
            "tasks.comments.allow-pattern: invalid regex `{pattern}`: {err}"
        )]
    })
}

#[cfg(test)]
#[path = "comments_tests.rs"]
mod tests;
