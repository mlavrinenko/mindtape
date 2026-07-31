//! Synthetic `slot(...)` calls for config-required core fields.
//!
//! `mt add` files tasks from a template, but a project's config may require
//! core fields (priority, difficulty, kind, estimate) the chosen template never
//! declares. This module derives the slot-bearing expressions those fields need
//! so a CLI can splice them into `task.with(…)` — surfacing them in the fill
//! scaffold and the `--describe` schema, and writing them into the final task.
//! Consumer-blind: pure over [`Config`], no IO, no source editing.

use crate::config::{Config, FieldConfig};
use crate::framework::{Framework, Input, InputKind};
use crate::numeric::{ScaleKind, format_num};

/// A core field the config requires, expressed as the source to set as a
/// `task.with(…)` argument when the chosen template omits that field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredSlot {
    /// The `task.with` argument name (`priority`, `kind`, …).
    pub field: &'static str,
    /// The source to set as that argument's value: a bare `slot(...)` call, or
    /// — for a strict-framework priority — a `framework(...)` call wrapping one
    /// `slot(...)` per input. Either way slot discovery walks it and every slot
    /// it holds is filled before the task is written.
    pub call: String,
}

/// The required core-field slots a config demands at task-creation time, in a
/// stable order (`priority`, `difficulty`, `kind`, `estimate`).
///
/// A field counts as required unless its requirement is `false`: a filter
/// expression (`"!terminal"`, `"kind == \"bug\""`) is taken as possibly required,
/// since the task being scaffolded does not exist yet — better to surface the
/// slot and let `mt check` settle it. Priority driven by a strict framework
/// (`framework` set, `allow-manual = false`) has no scalar form, so the
/// framework call itself is spliced with one leaf slot per declared input.
#[must_use]
pub fn required_core_slots(config: &Config) -> Vec<RequiredSlot> {
    let tasks = &config.tasks;
    let mut out = Vec::new();
    if tasks.priority.field.required.may_apply()
        && let Some(call) = priority_slot(config)
    {
        out.push(RequiredSlot {
            field: "priority",
            call,
        });
    }
    if tasks.difficulty.required.may_apply() {
        out.push(RequiredSlot {
            field: "difficulty",
            call: format!(
                r#"slot("difficulty", kind: "{}", prompt: "difficulty")"#,
                scale_kind_str(tasks.difficulty.kind)
            ),
        });
    }
    if let Some(field) = tasks.fields.get("kind")
        && field.required.may_apply()
    {
        out.push(RequiredSlot {
            field: "kind",
            call: kind_slot(field),
        });
    }
    if tasks.estimate.required.may_apply() {
        out.push(RequiredSlot {
            field: "estimate",
            call: estimate_slot(config),
        });
    }
    out
}

fn priority_slot(config: &Config) -> Option<String> {
    let priority = &config.tasks.priority;
    if priority.framework.is_some() && !priority.allow_manual {
        // A strict framework has no scalar form, so splice the framework call
        // itself with one leaf slot per input. Config loading rejects an
        // unknown framework name, so both lookups succeed for a loaded project.
        let name = priority.framework.as_deref()?;
        return Some(framework_call(name, priority.active_framework()?));
    }
    Some(format!(
        r#"slot("priority", kind: "{}", prompt: "priority")"#,
        scale_kind_str(priority.field.kind)
    ))
}

/// The `framework("name", input: slot(...), …)` call for a strict-framework
/// priority: the generic spelling every framework shares (the named helpers
/// like `ice(...)` are sugar over it), with one leaf slot per declared input.
/// The leaves carry no defaults — a strict priority must be a conscious fill,
/// and an unfilled leaf fails loudly at fill time instead of writing a
/// misprioritised task.
fn framework_call(name: &str, framework: &Framework) -> String {
    if framework.inputs.is_empty() {
        return format!(r#"framework("{name}")"#);
    }
    let leaves: String = framework
        .inputs
        .iter()
        .map(|(input, spec)| format!("{input}: {}, ", leaf_slot(name, input, spec)))
        .collect();
    format!(r#"framework("{name}", {leaves})"#)
}

/// One framework input as a fill slot: number → `float` (bounds in the
/// prompt), bool → `bool`, enum → `choice` over the declared values.
fn leaf_slot(framework: &str, input: &str, spec: &Input) -> String {
    let prompt = match (spec.kind, spec.min, spec.max) {
        (InputKind::Number, Some(lo), Some(hi)) => {
            format!(
                "{framework} {input} ({}..{})",
                format_num(lo),
                format_num(hi)
            )
        }
        _ => format!("{framework} {input}"),
    };
    match spec.kind {
        InputKind::Number => format!(r#"slot("{input}", kind: "float", prompt: "{prompt}")"#),
        InputKind::Bool => format!(r#"slot("{input}", kind: "bool", prompt: "{prompt}")"#),
        InputKind::Enum => {
            let choices: String = spec
                .values
                .iter()
                .map(|value| format!("{value:?}, "))
                .collect();
            format!(r#"slot("{input}", kind: "choice", choices: ({choices}), prompt: "{prompt}")"#)
        }
    }
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
