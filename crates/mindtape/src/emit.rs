//! Serialize a [`Task`] back to `#show: task.with(...)` Typst source — the
//! one new primitive `mt import` needs (`truth/tasks/task-to-typ-emitter.typ`).
//! Pure: no IO, no config policy — the caller has already mapped values onto
//! the project's config before calling in. Field order is fixed (see
//! [`emit_task`]), so two calls on an unchanged `Task` agree byte-for-byte.
//!
//! `recur` emits `every(...)` for a period rule and `rrule(...)` for the
//! escape hatch, mirroring the prelude's two constructors — see
//! `truth/tasks/carry-recur-through-the-canonical-task-json-round-trip.typ`.
//!
//! Split into two helper submodules to stay under the file-size gate:
//! `format` (Typst literal rendering: strings, numbers, arrays, `HVal`) and
//! `links` (the `links:` field's per-kind/per-note grouping).

pub mod format;
mod links;

use std::collections::BTreeMap;

use crate::date::parse_ymd;
use crate::model::{Status, Task};
use crate::numeric::{InputValue, NumValue, format_num};
use crate::recur::{Recur, Rule};
use crate::status_value::status_value;
use format::{hval_to_typ, typ_array, typ_str};

/// Serialize `task` to a full task-file source: the `@local/mindtape`
/// import, `#show: task.with(...)` with every set field, and the prose body
/// appended verbatim after the marker (nothing appended when the body is
/// empty — the emitter never synthesizes a `Summary`).
#[must_use]
pub fn emit_task(task: &Task) -> String {
    let fields = emit_fields(task);
    let mut out = String::from("#import \"@local/mindtape:0.2.0\": *\n\n#show: task.with(\n");
    for field in &fields {
        out.push_str("  ");
        out.push_str(field);
        out.push_str(",\n");
    }
    out.push_str(")\n");
    if !task.body.is_empty() {
        out.push('\n');
        out.push_str(&task.body);
        out.push('\n');
    }
    out
}

/// Every set field, in the fixed emission order: title, status, priority,
/// difficulty, kind, assignee, estimate, links, tags, due/start/reviewed,
/// recur, then the `x-<ns>` extras in their `BTreeMap` key order.
fn emit_fields(task: &Task) -> Vec<String> {
    let mut fields = vec![format!("title: {}", typ_str(&task.title))];
    if let Some(status) = task.status.as_ref().and_then(emit_status) {
        fields.push(format!("status: {status}"));
    }
    if let Some(priority) = &task.priority {
        fields.push(emit_num_field("priority", priority));
    }
    if let Some(difficulty) = &task.difficulty {
        fields.push(emit_num_field("difficulty", difficulty));
    }
    if let Some(kind) = &task.kind {
        fields.push(format!("kind: {}", typ_str(kind)));
    }
    if let Some(assignee) = &task.assignee {
        fields.push(format!("assignee: {}", typ_str(assignee)));
    }
    if let Some(estimate) = emit_estimate(&task.estimate) {
        fields.push(estimate);
    }
    if let Some(links) = links::emit_links(&task.links) {
        fields.push(links);
    }
    if !task.tags.is_empty() {
        let tags: Vec<String> = task.tags.iter().map(|tag| typ_str(tag)).collect();
        fields.push(format!("tags: {}", typ_array(&tags)));
    }
    for (name, value) in [
        ("due", task.due.as_deref()),
        ("start", task.start.as_deref()),
        ("reviewed", task.reviewed.as_deref()),
    ] {
        if let Some(field) = emit_date_field(name, value) {
            fields.push(field);
        }
    }
    if let Some(recur) = task.recur.as_ref().map(emit_recur) {
        fields.push(recur);
    }
    for (key, value) in &task.extras {
        fields.push(format!("{key}: {}", hval_to_typ(value)));
    }
    fields
}

/// `status: name(y, m, d)` / `name(y, m, d)[note]`, or `None` when the date
/// is missing or unparseable — a status with no date has nothing to render
/// (guaranteeing one is the caller's job, per the module's pure-serializer
/// contract).
fn emit_status(status: &Status) -> Option<String> {
    let ymd = parse_ymd(status.date.as_deref()?)?;
    Some(status_value(&status.name, ymd, status.note.as_deref()))
}

/// `name: on(y, m, d)`, or `None` when `value` is absent or unparseable.
fn emit_date_field(name: &str, value: Option<&str>) -> Option<String> {
    let (year, month, day) = parse_ymd(value?)?;
    Some(format!("{name}: on({year}, {month}, {day})"))
}

/// A numeric-field value (`priority`/`difficulty`): a raw scalar, a label
/// string, or a framework invocation.
fn emit_num_field(name: &str, value: &NumValue) -> String {
    match value {
        NumValue::Num(num) => format!("{name}: {}", format_num(*num)),
        NumValue::Label(label) => format!("{name}: {}", typ_str(label)),
        NumValue::Framework {
            name: framework,
            inputs,
        } => format!("{name}: {}", emit_framework(framework, inputs)),
    }
}

/// Priority frameworks the prelude ships a named helper for; any other name
/// falls back to the generic `framework("name", ...)` escape hatch.
const NAMED_FRAMEWORKS: &[&str] = &["ice", "rice", "wsjf", "eisenhower", "moscow"];

fn emit_framework(name: &str, inputs: &BTreeMap<String, InputValue>) -> String {
    if name == "moscow" {
        // `moscow(level)` takes one positional argument, not a named one.
        let level = inputs.get("level").map_or_else(String::new, emit_input);
        return format!("moscow({level})");
    }
    let named = emit_named_inputs(inputs);
    if NAMED_FRAMEWORKS.contains(&name) {
        format!("{name}({named})")
    } else if named.is_empty() {
        format!("framework({})", typ_str(name))
    } else {
        format!("framework({}, {named})", typ_str(name))
    }
}

fn emit_named_inputs(inputs: &BTreeMap<String, InputValue>) -> String {
    inputs
        .iter()
        .map(|(key, value)| format!("{key}: {}", emit_input(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn emit_input(value: &InputValue) -> String {
    match value {
        InputValue::Num(num) => format_num(*num),
        InputValue::Bool(flag) => flag.to_string(),
        InputValue::Text(text) => typ_str(text),
    }
}

/// `estimate: (unit: n, ...)`, or `None` when the map is empty.
fn emit_estimate(estimate: &BTreeMap<String, f64>) -> Option<String> {
    if estimate.is_empty() {
        return None;
    }
    let parts: Vec<String> = estimate
        .iter()
        .map(|(unit, value)| format!("{unit}: {}", format_num(*value)))
        .collect();
    Some(format!("estimate: ({})", parts.join(", ")))
}

/// `recur: every(...)` for a period rule, `recur: rrule(...)` for the escape
/// hatch. An unparseable `until` is simply omitted, matching every other
/// unparseable date field this module emits.
fn emit_recur(recur: &Recur) -> String {
    let (helper, mut args) = match &recur.rule {
        Rule::Period {
            unit,
            value,
            weekdays,
        } => {
            let mut args = vec![format!("{}: {value}", unit.as_str())];
            if !weekdays.is_empty() {
                let days: Vec<String> = weekdays
                    .iter()
                    .map(|day| typ_str(crate::recur::weekday_name(*day)))
                    .collect();
                args.push(format!("on: {}", typ_array(&days)));
            }
            ("every", args)
        }
        Rule::Rrule { raw } => ("rrule", vec![typ_str(raw)]),
    };
    if let Some(anchor) = &recur.anchor {
        args.push(format!("anchor: {}", typ_str(anchor)));
    }
    if let Some(until) = recur.until.as_deref().and_then(parse_ymd) {
        args.push(format!("until: on({}, {}, {})", until.0, until.1, until.2));
    }
    if let Some(count) = recur.count {
        args.push(format!("count: {count}"));
    }
    if let Some(materialize) = &recur.materialize {
        args.push(format!("materialize: {}", typ_str(materialize)));
    }
    format!("recur: {helper}({})", args.join(", "))
}

#[cfg(test)]
#[path = "emit_tests.rs"]
mod tests;
