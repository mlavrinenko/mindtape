//! `required` — whether a core field must be set, expressed as a filter.
//!
//! `[tasks.priority] required` (and its `difficulty`, `estimate` and
//! `[tasks.fields.<name>]` siblings) takes `true`, `false`, or a filter
//! expression naming exactly the tasks that must carry the field. The shipped
//! idiom is `required = "!terminal"` — required until the task reaches a
//! terminal status. Anything the filter language can say is now sayable here,
//! e.g. `required = "kind == \"bug\""`.
//!
//! Expressions are checked once, at config load: [`validate_requirements`]
//! (called from [`crate::config::Config::validate`]) parses each one and
//! dry-runs it against a probe task, so a malformed rule surfaces as a single
//! config error rather than one violation per artifact. A validation pass then
//! compiles the whole set once ([`RequiredRules::compile`]) and asks it task by
//! task.

use serde::{Deserialize, Deserializer};

use crate::config::Config;
use crate::filter::Filter;
use crate::model::{Status, Task};

/// Whether a field is required: `false`, `true`, or a filter expression.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Requirement {
    /// Never required.
    #[default]
    No,
    /// Always required.
    Always,
    /// Required exactly for the tasks this filter expression matches.
    Expr(String),
}

impl Requirement {
    /// Whether the field can be required at all — `false` only for
    /// [`Requirement::No`]. An expression counts as possible here because the
    /// answer needs a task, and callers that have none (`mt add` scaffolding a
    /// task that does not exist yet) must surface the field rather than drop it.
    #[must_use]
    pub fn may_apply(&self) -> bool {
        !matches!(self, Self::No)
    }
}

impl Requirement {
    /// [`may_apply`](Self::may_apply) for a task about to be filed at
    /// `status`. An expression that reads nothing but `status` and `terminal`
    /// is settled by the status alone, so it is evaluated; any other one still
    /// needs fields the task does not have yet, and may apply.
    #[must_use]
    pub fn may_apply_at(&self, status: &str, config: &Config) -> bool {
        let Self::Expr(source) = self else {
            return self.may_apply();
        };
        let Ok(filter) = Filter::parse(source) else {
            return true;
        };
        if !filter
            .names()
            .iter()
            .all(|name| name == "status" || name == "terminal")
        {
            return true;
        }
        let probe = Task {
            status: Some(Status {
                name: status.to_owned(),
                date: None,
                note: None,
            }),
            ..Task::default()
        };
        filter.matches(&probe, config).unwrap_or(true)
    }
}

impl<'de> Deserialize<'de> for Requirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ReqVisitor;
        impl serde::de::Visitor<'_> for ReqVisitor {
            type Value = Requirement;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a boolean or a filter expression")
            }
            fn visit_bool<E>(self, value: bool) -> Result<Requirement, E> {
                Ok(if value {
                    Requirement::Always
                } else {
                    Requirement::No
                })
            }
            fn visit_str<E>(self, value: &str) -> Result<Requirement, E> {
                Ok(match value {
                    "true" => Requirement::Always,
                    "false" => Requirement::No,
                    expr => Requirement::Expr(expr.to_owned()),
                })
            }
        }
        deserializer.deserialize_any(ReqVisitor)
    }
}

/// A compiled [`Requirement`], ready to test task by task.
#[derive(Debug, Clone)]
pub enum RequirementRule {
    /// A constant answer, from `required = true` / `required = false`.
    Fixed(bool),
    /// The parsed filter expression: required exactly for the tasks it matches.
    Expr(Box<Filter>),
}

impl RequirementRule {
    /// Compile one requirement.
    ///
    /// A malformed expression compiles to `Fixed(false)`: [`validate_requirements`]
    /// rejects those at config load, so this only covers a hand-built config that
    /// never went through `load_config`, and a rule that cannot be evaluated
    /// constrains nothing — the same reading `[[tasks.body.headers.rules]] when`
    /// gives a filter it cannot evaluate.
    #[must_use]
    pub fn compile(requirement: &Requirement) -> Self {
        match requirement {
            Requirement::No => Self::Fixed(false),
            Requirement::Always => Self::Fixed(true),
            Requirement::Expr(source) => Filter::parse(source)
                .map_or(Self::Fixed(false), |filter| Self::Expr(Box::new(filter))),
        }
    }

    /// Whether the field is required for `task`. An expression that fails to
    /// evaluate reads as "not required", for the reason [`Self::compile`] gives.
    #[must_use]
    pub fn applies(&self, task: &Task, config: &Config) -> bool {
        match self {
            Self::Fixed(value) => *value,
            Self::Expr(filter) => filter.matches(task, config).unwrap_or(false),
        }
    }
}

/// Every `required` rule a config declares, compiled once for a validation pass.
#[derive(Debug, Clone)]
pub struct RequiredRules {
    priority: RequirementRule,
    difficulty: RequirementRule,
    estimate: RequirementRule,
}

impl RequiredRules {
    /// Compile every `required` rule in `config`.
    #[must_use]
    pub fn compile(config: &Config) -> Self {
        let tasks = &config.tasks;
        Self {
            priority: RequirementRule::compile(&tasks.priority.field.required),
            difficulty: RequirementRule::compile(&tasks.difficulty.required),
            estimate: RequirementRule::compile(&tasks.estimate.required),
        }
    }

    /// Whether `task` must carry a priority.
    #[must_use]
    pub fn priority(&self, task: &Task, config: &Config) -> bool {
        self.priority.applies(task, config)
    }

    /// Whether `task` must carry a difficulty.
    #[must_use]
    pub fn difficulty(&self, task: &Task, config: &Config) -> bool {
        self.difficulty.applies(task, config)
    }

    /// Whether `task` must carry an estimate.
    #[must_use]
    pub fn estimate(&self, task: &Task, config: &Config) -> bool {
        self.estimate.applies(task, config)
    }
}

/// Load-time check of every `required` expression in `config`: one message per
/// unusable rule, prefixed by the rule's dotted TOML path.
///
/// Each expression is parsed and dry-run against a probe task, so an unknown
/// field, unknown view or unsupported operator is caught here — where it is one
/// config error — instead of at evaluation time, where it would repeat per task.
pub(crate) fn validate_requirements(config: &Config) -> Vec<String> {
    let probe = Task::default();
    requirements(config)
        .into_iter()
        .filter_map(|(path, requirement)| {
            problem(requirement, &probe, config).map(|problem| format!("{path}: {problem}"))
        })
        .collect()
}

/// Every `values-when` filter in `config`, paired with its dotted TOML path.
fn values_whens(config: &Config) -> Vec<(String, &str)> {
    let tasks = &config.tasks;
    let mut sources = vec![
        (
            "tasks.assignee.values-when".to_owned(),
            tasks.assignee.values_when.as_deref(),
        ),
        (
            "tasks.tags.values-when".to_owned(),
            tasks.tags.values_when.as_deref(),
        ),
    ];
    sources.extend(tasks.fields.iter().map(|(name, field)| {
        (
            format!("tasks.fields.{name}.values-when"),
            field.values_when.as_deref(),
        )
    }));
    sources
        .into_iter()
        .filter_map(|(path, source)| Some((path, source?)))
        .collect()
}

/// Load-time check of every `values-when` filter, like [`validate_requirements`].
pub(crate) fn validate_values_when(config: &Config) -> Vec<String> {
    let probe = Task::default();
    values_whens(config)
        .into_iter()
        .filter_map(|(path, source)| {
            let problem = crate::filter::expr_problem(source, &probe, config)?;
            Some(format!("{path}: {problem}"))
        })
        .collect()
}

/// Every filter the `[tasks]` field tables store, paired with its dotted TOML
/// path: the `required` expressions and the `values-when` filters.
pub(crate) fn field_filters(config: &Config) -> Vec<(String, String)> {
    let required = requirements(config)
        .into_iter()
        .filter_map(|(path, requirement)| match requirement {
            Requirement::Expr(source) => Some((path, source.clone())),
            _ => None,
        });
    let when = values_whens(config)
        .into_iter()
        .map(|(path, source)| (path, source.to_owned()));
    required.chain(when).collect()
}

/// Every `required` rule in `config`, paired with its dotted TOML path.
fn requirements(config: &Config) -> Vec<(String, &Requirement)> {
    let tasks = &config.tasks;
    let mut out = vec![
        (
            "tasks.priority.required".to_owned(),
            &tasks.priority.field.required,
        ),
        (
            "tasks.difficulty.required".to_owned(),
            &tasks.difficulty.required,
        ),
        (
            "tasks.estimate.required".to_owned(),
            &tasks.estimate.required,
        ),
        (
            "tasks.assignee.required".to_owned(),
            &tasks.assignee.required,
        ),
    ];
    out.extend(
        tasks
            .fields
            .iter()
            .map(|(name, field)| (format!("tasks.fields.{name}.required"), &field.required)),
    );
    out
}

/// What makes `requirement` unusable, if anything: a syntax error, or a
/// structural problem the probe dry-run surfaces.
fn problem(requirement: &Requirement, probe: &Task, config: &Config) -> Option<String> {
    let Requirement::Expr(source) = requirement else {
        return None;
    };
    crate::filter::expr_problem(source, probe, config)
}

#[cfg(test)]
#[path = "required_tests.rs"]
mod tests;
