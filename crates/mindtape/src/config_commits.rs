//! `[commits]` and `[[commits.rules]]` — what `mt check --commit-msg` holds a
//! commit to.
//!
//! ```toml
//! [commits]
//! trailer = "Refs"
//!
//! [[commits.rules]]
//! name    = "work-names-a-task"
//! scope   = "commit"
//! when    = '!tasks_only'
//! assert  = 'has(refs)'
//! message = "name the task this commit works on: `Refs: <stem>`"
//! ```
//!
//! A rule is the `[[tasks.rules]]` shape (`name`, `when`, `assert`,
//! `message`, `severity`) over a [`CommitRecord`] instead of a task, in the
//! language [`crate::commit_expr`] reads. It merges by `name` across rule
//! packs and layers exactly as `[[tasks.rules]]` does
//! ([`crate::config_packs`]), so a project overrides a shipped rule by
//! redeclaring its name or drops it with `disable`.
//!
//! Every expression is dry-run at config load ([`validate_commits`]), so a
//! rule naming a task field is one config error rather than a hook that fails
//! every commit.

use serde::Deserialize;

use crate::commit_expr::{CommitExpr, commit_expr_problem};
use crate::commit_msg::CommitRecord;
use crate::config::Config;
use crate::severity::Severity;

/// `[commits]` — how commits name the tasks they work on, and the rules a
/// commit is checked against.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct CommitsConfig {
    /// The trailer key whose values name tasks by stem (`Refs: <stem>`).
    /// Matched case-insensitively. A value that is a path, or names no task,
    /// is always refused.
    pub trailer: String,
    /// `[[commits.rules]]` — assertions every checked commit must satisfy.
    pub rules: Vec<CommitRuleSpec>,
}

impl Default for CommitsConfig {
    fn default() -> Self {
        Self {
            trailer: "Refs".to_owned(),
            rules: Vec::new(),
        }
    }
}

/// What a commit rule is evaluated over.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CommitScope {
    /// Once per commit, over its `subject`, `refs`, `files` and `tasks_only`.
    #[default]
    Commit,
}

/// `[[commits.rules]]` — one assertion a checked commit must satisfy.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CommitRuleSpec {
    /// What the rule is called, used to report it (`commits.rules.<name>`)
    /// and to override or `disable` it across packs. Optional: an unnamed
    /// rule reports by its position (`commits.rules[0]`).
    #[serde(default)]
    pub name: Option<String>,
    /// What the rule is evaluated over. Default `commit`.
    #[serde(default)]
    pub scope: CommitScope,
    /// Commit expression that must hold for every commit the rule applies to.
    #[serde(rename = "assert")]
    pub assertion: String,
    /// Commit expression gating which commits the rule applies to. Absent
    /// means every commit.
    #[serde(default)]
    pub when: Option<String>,
    /// What a breach tells the author, in place of the default advice naming
    /// the `assert` expression.
    #[serde(default)]
    pub message: Option<String>,
    /// Whether a breach fails the check or only reports. Default `warn`.
    #[serde(default)]
    pub severity: Severity,
}

impl CommitRuleSpec {
    /// How this rule identifies itself in a report.
    #[must_use]
    pub fn label(&self, index: usize) -> String {
        match &self.name {
            Some(name) => format!("commits.rules.{name}"),
            None => format!("commits.rules[{index}]"),
        }
    }
}

/// One breached commit rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitBreach {
    /// The rule's label (`commits.rules.<name>`).
    pub label: String,
    /// What to fix.
    pub message: String,
    /// Whether it fails the check.
    pub severity: Severity,
}

/// Evaluate every `[[commits.rules]]` entry over `commit`, in config order.
///
/// A `when` that fails to evaluate excludes the commit; an `assert` that
/// fails to evaluate is a breach naming the failure — the same reading
/// `[[tasks.rules]]` takes. Load-time validation makes both unreachable for a
/// loaded config.
#[must_use]
pub fn check_commit(commit: &CommitRecord, config: &Config) -> Vec<CommitBreach> {
    let mut breaches = Vec::new();
    for (index, spec) in config.commits.rules.iter().enumerate() {
        let applies = spec.when.as_deref().is_none_or(|when| {
            CommitExpr::parse(when)
                .and_then(|expr| expr.matches(commit))
                .unwrap_or(false)
        });
        if !applies {
            continue;
        }
        let failure = match CommitExpr::parse(&spec.assertion).and_then(|e| e.matches(commit)) {
            Ok(true) => continue,
            Ok(false) => spec
                .message
                .clone()
                .unwrap_or_else(|| format!("must satisfy `{}`", spec.assertion)),
            Err(err) => err.to_string(),
        };
        breaches.push(CommitBreach {
            label: spec.label(index),
            message: failure,
            severity: spec.severity,
        });
    }
    breaches
}

/// Config-load checks for `[commits]`: a non-empty trailer key, every rule's
/// expressions dry-run, and no two rules sharing a name within one file (the
/// pack merge has already collapsed cross-source ones).
pub(crate) fn validate_commits(config: &Config) -> Vec<String> {
    let mut problems = Vec::new();
    if config.commits.trailer.trim().is_empty() {
        problems.push("commits.trailer: must name a trailer key".to_owned());
    }
    for (index, spec) in config.commits.rules.iter().enumerate() {
        let path = format!("commits.rules[{index}]");
        if let Some(problem) = commit_expr_problem(&spec.assertion) {
            problems.push(format!("{path}.assert: {problem}"));
        }
        if let Some(problem) = spec.when.as_deref().and_then(commit_expr_problem) {
            problems.push(format!("{path}.when: {problem}"));
        }
    }
    problems.extend(crate::body_headers::duplicate_rule_names(
        config.commits.rules.iter().map(|rule| rule.name.as_deref()),
        "commits.rules",
    ));
    problems
}

#[cfg(test)]
#[path = "config_commits_tests.rs"]
mod tests;
