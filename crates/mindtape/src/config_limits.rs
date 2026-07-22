//! `[[tasks.limits]]` — caps on how many tasks may sit in a matched set at once.
//!
//! A limit is a filter expression plus a `max`: bounded work-in-progress, the
//! one mechanism that stops a session ending on an open leaf task. No status
//! name appears here. The legend is project-configured — a project may name its
//! in-flight state anything, or have several — so the capped set is a filter,
//! and the same primitive caps a status, a view (`in_view("name")` / `@name`), a
//! priority band or a `kind`.
//!
//! `mt check` is the only consumer ([`check_limits`]): it counts each limit's
//! matches over the loaded set and reports every breach with the cap, the count
//! and the offending tasks. A breach carries a [`LimitSeverity`] — `warn` (the
//! default) prints and passes, `error` fails the command. Warning is the default
//! because a breach is a state of the backlog rather than a malformed artifact:
//! it clears by finishing work, not by editing a file.
//!
//! A limit filter leans on the missing-field comparison rule (see `compare` in
//! [`crate::filter`]): a field a task does not carry is unequal to every
//! literal. So `kind == "umbrella"` and `kind != "umbrella"` partition the whole
//! set, and every task with no `kind` lands in the second half by construction.

use std::fmt;

use serde::Deserialize;
use thiserror::Error;

use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::ready::Readiness;

/// Advice reported for a breach the limit declares no `message` for. Names the
/// action to take rather than restating the condition the report already prints.
const DEFAULT_ADVICE: &str = "finish or drop one before starting another";

/// `[[tasks.limits]]` — one cap on a filter-matched set of tasks.
///
/// `filter` and `max` are required: a limit with no cap is not a limit, and
/// defaulting a missing `max` to zero would silently forbid the set outright.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LimitSpec {
    /// Filter expression selecting the capped set. The `mt ls` filter language,
    /// so `@name` / `in_view("name")` caps a named view and no second spelling
    /// of "which tasks" is needed.
    pub filter: String,
    /// How many matching tasks are allowed at once; a greater count is a breach.
    pub max: usize,
    /// Replaces the default advice in the breach report.
    #[serde(default)]
    pub message: Option<String>,
    /// Whether a breach fails `mt check` or only warns. Default `warn`.
    #[serde(default)]
    pub severity: LimitSeverity,
}

/// How a breached limit reports.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LimitSeverity {
    /// Print the breach; `mt check` still succeeds. The default.
    #[default]
    Warn,
    /// Print the breach and fail `mt check`.
    Error,
}

impl LimitSeverity {
    /// Whether a breach at this severity fails the command.
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    /// Line prefix for reporting a breach at this severity.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Warn => "warning",
            Self::Error => "error",
        }
    }
}

/// One limit exceeded: its cap, the tasks that matched, and how to report it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitBreach {
    /// Position of the limit in `[[tasks.limits]]`.
    pub index: usize,
    /// The limit's filter expression.
    pub filter: String,
    /// The declared cap.
    pub max: usize,
    /// Ids of the matching tasks, in load order.
    pub tasks: Vec<String>,
    /// The limit's `message`, else [`DEFAULT_ADVICE`].
    pub message: String,
    /// The limit's severity.
    pub severity: LimitSeverity,
}

impl LimitBreach {
    /// How many tasks matched — always greater than [`max`](Self::max).
    #[must_use]
    pub fn count(&self) -> usize {
        self.tasks.len()
    }
}

impl fmt::Display for LimitBreach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{count} task(s) match `{filter}`, max {max} — {message}",
            count = self.count(),
            filter = self.filter,
            max = self.max,
            message = self.message,
        )
    }
}

/// A `[[tasks.limits]]` entry whose filter could not be parsed or evaluated.
#[derive(Debug, Error)]
#[error("tasks.limits[{index}] filter `{filter}`: {source}")]
pub struct LimitError {
    /// Position of the offending limit in `[[tasks.limits]]`.
    pub index: usize,
    /// The filter expression that failed.
    pub filter: String,
    /// The underlying filter failure.
    #[source]
    pub source: FilterError,
}

/// Evaluate every `[[tasks.limits]]` entry over `tasks`, returning one
/// [`LimitBreach`] per limit whose match count exceeds its `max`, in config
/// order. A limit that matches nothing — or at most `max` — is silent.
///
/// Readiness for the `ready` / `blocked` predicates is computed over the full
/// set passed in, so a limit can cap "what is startable" as readily as a status.
///
/// # Errors
/// Returns [`LimitError`] when a limit's filter fails to parse or names an
/// unknown field or view.
pub fn check_limits(tasks: &[Task], config: &Config) -> Result<Vec<LimitBreach>, LimitError> {
    if config.tasks.limits.is_empty() {
        return Ok(Vec::new());
    }
    let readiness = Readiness::compute(tasks, config);
    let mut breaches = Vec::new();
    for (index, limit) in config.tasks.limits.iter().enumerate() {
        let matched =
            matching_ids(limit, tasks, config, &readiness).map_err(|source| LimitError {
                index,
                filter: limit.filter.clone(),
                source,
            })?;
        if matched.len() > limit.max {
            breaches.push(LimitBreach {
                index,
                filter: limit.filter.clone(),
                max: limit.max,
                tasks: matched,
                message: limit
                    .message
                    .clone()
                    .unwrap_or_else(|| DEFAULT_ADVICE.to_owned()),
                severity: limit.severity,
            });
        }
    }
    Ok(breaches)
}

/// Check every `[[tasks.limits]]` filter at config load, returning one message
/// per unusable entry, prefixed with its dotted TOML path.
///
/// Parses each expression and dry-runs it against a probe task, exactly as
/// `required` and `[views]` are checked — a malformed limit is one config error
/// at load, not a surprise at the far end of a `mt check` run.
pub(crate) fn validate_limits(config: &Config) -> Vec<String> {
    let probe = Task::default();
    config
        .tasks
        .limits
        .iter()
        .enumerate()
        .filter_map(|(index, limit)| {
            limit_problem(&limit.filter, &probe, config)
                .map(|problem| format!("tasks.limits[{index}].filter: {problem}"))
        })
        .collect()
}

/// What makes one limit's filter unusable, if anything: a syntax error, or a
/// structural problem the probe dry-run surfaces (an unknown field or view).
fn limit_problem(source: &str, probe: &Task, config: &Config) -> Option<String> {
    match Filter::parse(source) {
        Err(err) => Some(err.to_string()),
        Ok(filter) => filter
            .matches(probe, config)
            .err()
            .map(|err| err.to_string()),
    }
}

/// Ids of the tasks matching one limit's filter, in load order.
fn matching_ids(
    limit: &LimitSpec,
    tasks: &[Task],
    config: &Config,
    readiness: &Readiness,
) -> Result<Vec<String>, FilterError> {
    let filter = Filter::parse(&limit.filter)?;
    let mut ids = Vec::new();
    for task in tasks {
        if filter.matches_with(task, config, readiness)? {
            ids.push(task.id.clone());
        }
    }
    Ok(ids)
}

#[cfg(test)]
#[path = "config_limits_tests.rs"]
mod tests;
