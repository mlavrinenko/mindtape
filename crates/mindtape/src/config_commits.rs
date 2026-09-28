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
//! language [`crate::commit_expr`] reads. A `scope = "task"` rule runs once
//! per task the commit changes or names, reading it as of `HEAD`, the index
//! and the working tree ([`crate::commit_task_expr`]):
//!
//! ```toml
//! [[commits.rules]]
//! name    = "flip-with-work"
//! scope   = "task"
//! when    = 'into("wip") || into("done")'
//! assert  = 'named && !tasks_only'
//! message = "start or close a task in a commit that carries its work"
//! ```
//!
//! Rules merge by `name` across rule packs and layers exactly as
//! `[[tasks.rules]]` does
//! ([`crate::config_packs`]), so a project overrides a shipped rule by
//! redeclaring its name or drops it with `disable`.
//!
//! Every expression is dry-run at config load ([`validate_commits`]), so a
//! rule naming a task field is one config error rather than a hook that fails
//! every commit.

use serde::Deserialize;

use crate::commit_expr::{CommitExpr, commit_expr_problem};
use crate::commit_msg::CommitRecord;
use crate::commit_task_expr::{TaskScope, desugar_commit_into, task_expr_problem};
use crate::commit_tasks::CommitStates;
use crate::config::Config;
use crate::filter::FilterError;
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

impl CommitsConfig {
    /// Whether any rule runs per task — so the commit's states need loading.
    #[must_use]
    pub fn has_task_rules(&self) -> bool {
        self.rules
            .iter()
            .any(|rule| rule.scope == CommitScope::Task)
    }
}

/// What a commit rule is evaluated over.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CommitScope {
    /// Once per commit, over its `subject`, `refs`, `files` and `tasks_only`.
    #[default]
    Commit,
    /// Once per task the commit changes or names, over the commit's fields
    /// plus `named` and the task as of `head(...)`, `staged(...)` and
    /// `worktree(...)`.
    Task,
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

    /// Whether the rule reads the working tree: `worktree(...)`, or
    /// `changed(...)`, whose git states are the checkout's own. A past
    /// commit has neither, so a range check skips the rule.
    #[must_use]
    pub fn reads_worktree(&self) -> bool {
        let rule = Rule::parse(self);
        [rule.when.as_ref(), Some(&rule.assertion)]
            .into_iter()
            .flatten()
            .filter_map(|expr| expr.as_ref().ok())
            .any(|expr| expr.calls("worktree") || expr.calls("changed"))
    }
}

/// One breached commit rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitBreach {
    /// The rule's label (`commits.rules.<name>`).
    pub label: String,
    /// The stem of the task a `scope = "task"` rule breached for.
    pub task: Option<String>,
    /// What to fix.
    pub message: String,
    /// Whether it fails the check.
    pub severity: Severity,
}

/// Evaluate every `[[commits.rules]]` entry over `commit`, in config order —
/// a `scope = "task"` rule once per task in [`CommitRecord::tasks`], reading
/// `states`. With no `states` the task rules are not run; with no working
/// tree among them, a rule that [reads it](CommitRuleSpec::reads_worktree)
/// is not run.
///
/// A `when` that fails to evaluate excludes the commit; an `assert` that
/// fails to evaluate is a breach naming the failure — the same reading
/// `[[tasks.rules]]` takes. Load-time validation makes both unreachable for a
/// loaded config.
#[must_use]
pub fn check_commit(
    commit: &CommitRecord,
    states: Option<&CommitStates>,
    config: &Config,
) -> Vec<CommitBreach> {
    let mut breaches = Vec::new();
    let no_worktree = states.is_none_or(|states| states.worktree.is_none());
    for (index, spec) in config.commits.rules.iter().enumerate() {
        if no_worktree && spec.reads_worktree() {
            continue;
        }
        let rule = Rule::parse(spec);
        match spec.scope {
            CommitScope::Commit => {
                breaches.extend(rule.check(spec, index, None, |expr| expr.matches(commit)));
            }
            CommitScope::Task => {
                let Some(states) = states else { continue };
                for stem in &commit.tasks {
                    let scope = TaskScope::new(stem, states, config);
                    breaches.extend(rule.check(spec, index, Some(stem), |expr| {
                        expr.matches_task(commit, &scope)
                    }));
                }
            }
        }
    }
    breaches
}

/// One rule's parsed expressions, `into(...)` rewritten for a task rule.
struct Rule {
    when: Option<Result<CommitExpr, String>>,
    assertion: Result<CommitExpr, String>,
}

impl Rule {
    fn parse(spec: &CommitRuleSpec) -> Self {
        let source = |expr: &str| match spec.scope {
            CommitScope::Commit => expr.to_owned(),
            CommitScope::Task => desugar_commit_into(expr),
        };
        let parse = |expr: &str| CommitExpr::parse(&source(expr)).map_err(|err| err.to_string());
        Self {
            when: spec.when.as_deref().map(parse),
            assertion: parse(&spec.assertion),
        }
    }

    /// The breach `spec` reports when `eval` reads the rule as applying and
    /// its assertion as failing.
    fn check(
        &self,
        spec: &CommitRuleSpec,
        index: usize,
        task: Option<&str>,
        eval: impl Fn(&CommitExpr) -> Result<bool, FilterError>,
    ) -> Option<CommitBreach> {
        let run = |expr: &Result<CommitExpr, String>| match expr {
            Ok(expr) => eval(expr).map_err(|err| err.to_string()),
            Err(err) => Err(err.clone()),
        };
        if !self
            .when
            .as_ref()
            .is_none_or(|when| run(when).unwrap_or(false))
        {
            return None;
        }
        let failure = match run(&self.assertion) {
            Ok(true) => return None,
            Ok(false) => spec
                .message
                .clone()
                .unwrap_or_else(|| format!("must satisfy `{}`", spec.assertion)),
            Err(err) => err,
        };
        Some(CommitBreach {
            label: spec.label(index),
            task: task.map(ToOwned::to_owned),
            message: failure,
            severity: spec.severity,
        })
    }
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
        let problem = |source: &str| match spec.scope {
            CommitScope::Commit => commit_expr_problem(source),
            CommitScope::Task => task_expr_problem(source, config),
        };
        if let Some(problem) = problem(&spec.assertion) {
            problems.push(format!("{path}.assert: {problem}"));
        }
        if let Some(problem) = spec.when.as_deref().and_then(problem) {
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
