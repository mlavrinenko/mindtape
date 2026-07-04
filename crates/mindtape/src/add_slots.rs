//! Synthetic `slot(...)` calls for config-required core fields.
//!
//! `mt add` files tasks from a template, but a project's config may require
//! core fields (priority, difficulty, kind, estimate) the chosen template never
//! declares. This module derives the `slot(...)` calls those fields need so a CLI
//! can splice them into `task.with(…)` — surfacing them in the fill scaffold and
//! the `--describe` schema, and writing them into the final task. Consumer-blind:
//! pure over [`Config`], no IO, no source editing.

use crate::config::{Config, FieldConfig, Requirement};
use crate::numeric::ScaleKind;

/// A core field the config requires, expressed as a `slot(...)` call to set as a
/// `task.with(…)` argument when the chosen template omits that field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredSlot {
    /// The `task.with` argument name (`priority`, `kind`, …).
    pub field: &'static str,
    /// The `slot(...)` call source to set as that argument's value.
    pub call: String,
}

/// The required core-field slots a config demands at task-creation time, in a
/// stable order (`priority`, `difficulty`, `kind`, `estimate`).
///
/// A field counts as required when its requirement is `Always` or `NonTerminal`
/// (a freshly added task is non-terminal). Priority driven by a strict framework
/// (`framework` set, `allow-manual = false`) has no scalar form and is omitted —
/// such a project must declare the framework call in its template.
#[must_use]
pub fn required_core_slots(config: &Config) -> Vec<RequiredSlot> {
    let tasks = &config.tasks;
    let mut out = Vec::new();
    if required(tasks.priority.field.required)
        && let Some(call) = priority_slot(config)
    {
        out.push(RequiredSlot {
            field: "priority",
            call,
        });
    }
    if required(tasks.difficulty.required) {
        out.push(RequiredSlot {
            field: "difficulty",
            call: format!(
                r#"slot("difficulty", kind: "{}", prompt: "difficulty")"#,
                scale_kind_str(tasks.difficulty.kind)
            ),
        });
    }
    if let Some(field) = tasks.fields.get("kind")
        && required(field.required)
    {
        out.push(RequiredSlot {
            field: "kind",
            call: kind_slot(field),
        });
    }
    if required(tasks.estimate.required) {
        out.push(RequiredSlot {
            field: "estimate",
            call: estimate_slot(config),
        });
    }
    out
}

fn required(requirement: Requirement) -> bool {
    !matches!(requirement, Requirement::No)
}

fn priority_slot(config: &Config) -> Option<String> {
    let priority = &config.tasks.priority;
    if priority.framework.is_some() && !priority.allow_manual {
        return None;
    }
    Some(format!(
        r#"slot("priority", kind: "{}", prompt: "priority")"#,
        scale_kind_str(priority.field.kind)
    ))
}

/// The DSL spelling of a numeric field's scale, mirroring `ScaleKind`'s own
/// config spelling (`int`/`real`) so a slot's kind faithfully reports the
/// field's declared scale, not a generic template value kind.
fn scale_kind_str(kind: ScaleKind) -> &'static str {
    match kind {
        ScaleKind::Int => "int",
        ScaleKind::Real => "real",
    }
}

fn kind_slot(field: &FieldConfig) -> String {
    if field.values.is_empty() {
        return r#"slot("kind", prompt: "kind")"#.to_owned();
    }
    let choices: String = field
        .values
        .iter()
        .map(|value| format!("{value:?}, "))
        .collect();
    let default = field
        .default
        .as_ref()
        .map_or_else(String::new, |value| format!(", default: {value:?}"));
    format!(r#"slot("kind", kind: "choice", choices: ({choices}){default})"#)
}

fn estimate_slot(config: &Config) -> String {
    let hint = config
        .tasks
        .estimate
        .units
        .first()
        .map_or("days", String::as_str);
    format!(r#"slot("estimate", kind: "text", prompt: "estimate dict, e.g. ({hint}: 1)")"#)
}

#[cfg(test)]
#[path = "add_slots_tests.rs"]
mod tests;
