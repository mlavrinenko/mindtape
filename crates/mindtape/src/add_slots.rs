//! Synthetic `slot(...)` calls for config-required core fields.
//!
//! `mt add` files tasks from a template, but a project's config may require
//! core fields (priority, difficulty, kind, assignee, estimate) or `x-<ns>`
//! fields the chosen template never declares. This module derives the
//! slot-bearing expressions those fields need so a CLI can splice them into
//! `task.with(…)` — surfacing them in the fill scaffold and the `--describe`
//! schema, and writing them into the final task. A slot the template does
//! declare for such a field inherits the field's config `values` as its
//! choices ([`inherit_field_choices`]). Consumer-blind: pure over [`Config`]
//! and the template source, no IO.

use typst_edit::{Call, Edit, EditError, apply, find_calls, find_method_calls};

use crate::config::{Config, Requirement};
use crate::framework::{Framework, Input, InputKind};
use crate::model::field_descriptor::{self as field_descriptor, Descriptor};
use crate::model::fields;
use crate::model::numeric::{NumericField, ScaleKind, format_num};

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
            call: scale_slot("difficulty", &tasks.difficulty),
            env: None,
            default: None,
        });
    }
    let described = field_descriptor::descriptors(config);
    let slots = |name: &str| {
        described
            .iter()
            .filter(|field| field.name == name && may_apply(field.required))
            .map(field_slot)
            .collect::<Vec<_>>()
    };
    out.extend(slots("kind"));
    out.extend(slots("assignee"));
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
        described
            .iter()
            .filter(|field| fields::is_extra(field.name) && may_apply(field.required))
            .map(field_slot),
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
    Some(scale_slot("priority", &priority.field))
}

/// The `framework("name", input: slot(...), …)` call for a strict-framework
/// priority: the generic spelling every framework shares (the named helpers
/// like `ice(...)` are sugar over it), with one leaf slot per declared input.
/// The leaves carry no defaults — a strict priority must be a conscious fill,
/// and an unfilled leaf fails loudly at fill time instead of writing a
/// misprioritised task. Each carries an example inside its range, which only
/// `mt check` fills.
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
        InputKind::Number => {
            let example = match (spec.min, spec.max) {
                (Some(lo), Some(hi)) => (lo + hi) / 2.0,
                (Some(bound), None) | (None, Some(bound)) => bound,
                (None, None) => 1.0,
            };
            format!(
                r#"slot("{input}", kind: "float", prompt: "{prompt}", example: {})"#,
                format_num(example)
            )
        }
        InputKind::Bool => {
            format!(r#"slot("{input}", kind: "bool", prompt: "{prompt}", example: false)"#)
        }
        InputKind::Enum => {
            let choices = choice_list(&spec.values);
            let example = spec.values.first().map_or("", String::as_str);
            format!(
                r#"slot("{input}", kind: "choice", choices: ({choices}), prompt: "{prompt}", example: {example:?})"#
            )
        }
    }
}

/// The slot for a numeric scale: its kind and, when the config gives one,
/// its example.
fn scale_slot(name: &str, field: &NumericField) -> String {
    let example = field.example.map_or_else(String::new, |value| {
        format!(", example: {}", format_num(value))
    });
    format!(
        r#"slot("{name}", kind: "{}", prompt: "{name}"{example})"#,
        scale_kind_str(field.kind)
    )
}

/// `values` as the inside of a Typst array literal of strings.
fn choice_list(values: &[String]) -> String {
    values.iter().map(|value| format!("{value:?}, ")).collect()
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

/// The slot for a required described field: a choice over its allowed values
/// when it declares any, else a plain string slot, falling back to `default`
/// either way. Its example is the default, else the first allowed value, else
/// the descriptor's own.
fn field_slot(field: &Descriptor) -> RequiredSlot {
    let name = field.name;
    let list = field.cardinality.is_list();
    let literal = |value: &str| {
        if list {
            format!("({value:?},)")
        } else {
            format!("{value:?}")
        }
    };
    let default = field.default.map_or_else(String::new, |value| {
        format!(", default: {}", literal(value))
    });
    let env = field
        .env
        .map_or_else(String::new, |env| format!(", env: \"{env}\""));
    let example = field
        .example
        .as_ref()
        .map_or_else(String::new, |example| format!(", example: {example}"));
    let call = if field.allowed.is_empty() {
        let kind = if list { r#"kind: "list", "# } else { "" };
        format!(r#"slot("{name}", {kind}prompt: "{name}"{default}{env}{example})"#)
    } else {
        let kind = if list { "multichoice" } else { "choice" };
        let choices = choice_list(field.allowed);
        format!(r#"slot("{name}", kind: "{kind}", choices: ({choices}){default}{env}{example})"#)
    };
    RequiredSlot {
        field: name.to_owned(),
        config_path: field.path.clone(),
        call,
        env: field.env,
        default: field.default.map(literal),
    }
}

fn estimate_slot(config: &Config) -> String {
    let estimate = &config.tasks.estimate;
    let hint = estimate.units.first().map_or("unit", String::as_str);
    let example = estimate.example.as_ref().map_or_else(String::new, |pairs| {
        let inner: Vec<String> = pairs
            .iter()
            .map(|(unit, quantity)| format!("{unit}: {}", format_num(*quantity)))
            .collect();
        format!(r#", example: "({})""#, inner.join(", "))
    });
    format!(r#"slot("estimate", kind: "text", prompt: "estimate dict, e.g. ({hint}: 1)"{example})"#)
}

/// `source` with each `slot(...)` that is the whole value of a `task.with`
/// named argument made a choice over that field's `[tasks.fields]` values.
///
/// The field is the argument the slot sits in, never the slot's name, so
/// `kind: slot("type")` inherits `kind`'s values. A slot inherits only when it
/// declares no `choices` and its kind is absent, `string` or `choice`; any
/// other kind, and a slot nested inside a field's value, is left as written.
/// The choices live only in the template source the fill reads: filling
/// replaces the whole call, so none of them reach the filed task.
///
/// # Errors
/// Returns an [`EditError`] when the edits cannot be applied, which two
/// disjoint slot calls never cause.
pub fn inherit_field_choices(source: &str, config: &Config) -> Result<String, EditError> {
    let Some(call) = find_method_calls(source, "task", "with").into_iter().next() else {
        return Ok(source.to_owned());
    };
    let slots = find_calls(source, "slot");
    let described = field_descriptor::descriptors(config);
    let edits: Vec<Edit> = (call.args.iter())
        .filter_map(|arg| {
            let name = arg.name.as_deref()?;
            let field = described.iter().find(|field| field.name == name)?;
            let slot = slots.iter().find(|slot| slot.range == arg.value_range)?;
            choice_edits(slot, field.allowed, field.cardinality.is_list())
        })
        .flatten()
        .collect();
    if edits.is_empty() {
        return Ok(source.to_owned());
    }
    apply(source, edits)
}

/// The edits that make `slot` a choice over `values` (a multi-choice for a
/// list field), or `None` when it keeps its own spec.
fn choice_edits(slot: &Call, values: &[String], list: bool) -> Option<Vec<Edit>> {
    if values.is_empty() || slot.named("choices").is_some() {
        return None;
    }
    let (plain, kind) = if list {
        ("list", "multichoice")
    } else {
        ("string", "choice")
    };
    let after_name = slot.positional(0)?.value_range.end;
    let choices = format!(", choices: ({})", choice_list(values));
    match slot.named("kind").map(|arg| (arg, arg.value.as_str())) {
        None => Some(vec![Edit::new(
            after_name..after_name,
            format!(r#", kind: "{kind}"{choices}"#),
        )]),
        Some((arg, declared)) if declared == plain || declared == kind => Some(vec![
            Edit::new(after_name..after_name, choices),
            Edit::new(arg.value_range.clone(), format!("{kind:?}")),
        ]),
        Some(_) => None,
    }
}

#[cfg(test)]
#[path = "add_slots_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "add_slots_tests_inherit.rs"]
mod tests_inherit;
