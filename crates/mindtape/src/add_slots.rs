//! Synthetic `slot(...)` calls for config-required core fields.
//!
//! `mt add` files tasks from a template, but a project's config may require
//! core fields (priority, difficulty, kind, assignee, estimate) or `x-<ns>`
//! fields the chosen template never declares. This module derives the
//! slot-bearing expressions those fields need so a CLI can splice them into
//! `task.with(…)` — surfacing them in the fill scaffold and the `--describe`
//! schema, and writing them into the final task. Consumer-blind: pure over
//! [`Config`], no IO, no source editing.

use crate::config::{AssigneeConfig, Config, FieldConfig, Requirement};
use crate::framework::{Framework, Input, InputKind};
use crate::numeric::{ScaleKind, format_num};

/// A core field the config requires, expressed as the source to set as a
/// `task.with(…)` argument when the chosen template omits that field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredSlot {
    /// The `task.with` argument name (`priority`, `kind`, …).
    pub field: String,
    /// The config path the requirement comes from, named in the fill note
    /// (`[tasks.assignee]`, `[tasks.fields.kind]`).
    pub config_path: String,
    /// The source to set as that argument's value: a bare `slot(...)` call, or
    /// — for a strict-framework priority — a `framework(...)` call wrapping one
    /// `slot(...)` per input. Either way slot discovery walks it and every slot
    /// it holds is filled before the task is written.
    pub call: String,
    /// The environment variable the slot reads, when it declares one. A CLI
    /// uses it to name the origin of a value the caller did not supply.
    pub env: Option<&'static str>,
    /// The config default the slot falls back to, when it has one, in the
    /// source form the slot carries (`"feature"`). A CLI compares the filled
    /// value against it to tell a default fill from a hand-supplied one.
    pub default: Option<String>,
}

/// The required core-field slots a config demands at task-creation time, in a
/// stable order (`priority`, `difficulty`, `kind`, `assignee`, `estimate`,
/// then each `x-<ns>` field by name).
///
/// A field counts as required unless its requirement is `false`, or `status`,
/// the status the task is filed at, settles it as not required (`"!terminal"`
/// at `done`). Any other filter expression (`"kind == \"bug\""`) is taken as
/// possibly required, since the task being scaffolded does not exist yet —
/// better to surface the slot and let `mt check` settle it. Priority driven by a strict framework
/// (`framework` set, `allow-manual = false`) has no scalar form, so the
/// framework call itself is spliced with one leaf slot per declared input.
#[must_use]
pub fn required_core_slots(config: &Config, status: Option<&str>) -> Vec<RequiredSlot> {
    let tasks = &config.tasks;
    let may_apply = |required: &Requirement| {
        status.map_or_else(
            || required.may_apply(),
            |status| required.may_apply_at(status, config),
        )
    };
    let mut out = Vec::new();
    if may_apply(&tasks.priority.field.required)
        && let Some(call) = priority_slot(config)
    {
        out.push(RequiredSlot {
            field: "priority".to_owned(),
            config_path: "[tasks.priority]".to_owned(),
            call,
            env: None,
            default: None,
        });
    }
    if may_apply(&tasks.difficulty.required) {
        out.push(RequiredSlot {
            field: "difficulty".to_owned(),
            config_path: "[tasks.difficulty]".to_owned(),
            call: format!(
                r#"slot("difficulty", kind: "{}", prompt: "difficulty")"#,
                scale_kind_str(tasks.difficulty.kind)
            ),
            env: None,
            default: None,
        });
    }
    if let Some(field) = tasks.fields.get("kind")
        && may_apply(&field.required)
    {
        out.push(field_slot("kind", field));
    }
    if may_apply(&tasks.assignee.required) {
        out.push(RequiredSlot {
            field: "assignee".to_owned(),
            config_path: "[tasks.assignee]".to_owned(),
            call: assignee_slot(&tasks.assignee),
            env: Some(PRINCIPAL_ENV),
            default: None,
        });
    }
    if may_apply(&tasks.estimate.required) {
        out.push(RequiredSlot {
            field: "estimate".to_owned(),
            config_path: "[tasks.estimate]".to_owned(),
            call: estimate_slot(config),
            env: None,
            default: None,
        });
    }
    out.extend(
        tasks
            .fields
            .iter()
            .filter(|(name, field)| name.starts_with("x-") && may_apply(&field.required))
            .map(|(name, field)| field_slot(name, field)),
    );
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

/// The slot for a required `[tasks.fields.<name>]` entry: a choice over
/// `values` when the entry declares any, else a plain string slot, falling
/// back to `default` either way.
fn field_slot(name: &str, field: &FieldConfig) -> RequiredSlot {
    let default = field
        .default
        .as_ref()
        .map_or_else(String::new, |value| format!(", default: {value:?}"));
    let call = if field.values.is_empty() {
        format!(r#"slot("{name}", prompt: "{name}"{default})"#)
    } else {
        let choices: String = field
            .values
            .iter()
            .map(|value| format!("{value:?}, "))
            .collect();
        format!(r#"slot("{name}", kind: "choice", choices: ({choices}){default})"#)
    };
    RequiredSlot {
        field: name.to_owned(),
        config_path: format!("[tasks.fields.{name}]"),
        call,
        env: None,
        default: field.default.as_ref().map(|value| format!("{value:?}")),
    }
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

/// Environment variable a required assignee slot reads by default: the actor
/// running the CLI. Unset or blank is an unknown actor — not a value — so the
/// fill falls through to the interactive scaffold.
const PRINCIPAL_ENV: &str = "MINDTAPE_PRINCIPAL";

fn assignee_slot(assignee: &AssigneeConfig) -> String {
    if assignee.allowed.is_empty() {
        return format!(r#"slot("assignee", prompt: "assignee", env: "{PRINCIPAL_ENV}")"#);
    }
    let choices: String = assignee
        .allowed
        .iter()
        .map(|value| format!("{value:?}, "))
        .collect();
    format!(r#"slot("assignee", kind: "choice", choices: ({choices}), env: "{PRINCIPAL_ENV}")"#)
}

#[cfg(test)]
#[path = "add_slots_tests.rs"]
mod tests;
