//! Config-driven validation of loaded tasks.
//!
//! Enforces the declared status lifecycle, enum field constraints, the estimate
//! unit allow-list, the `required` rules (a filter expression per field, see
//! [`crate::config_required`]), the tag allow-list, and the typed `links:`
//! edges (resolution, cardinality, notes, cycles — see
//! [`crate::validate_links`]).

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use uuid::Uuid;

use crate::config::{
    Config, EstimateConfig, FieldConfig, IdFormat, Legend, PriorityConfig, RequiredRules,
};
use crate::model::{Status, Task};
use crate::numeric::{InputValue, NumValue, NumericField, format_num};

/// Source location for a violation that points to a specific line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// File path relative to the project root.
    pub file: PathBuf,
    /// One-based line number.
    pub line: usize,
}

/// A single validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Offending task id.
    pub task_id: String,
    /// Field or aspect at fault (`status`, `priority`, `tags`, `depends-on`, …).
    pub field: String,
    /// Human-readable message.
    pub message: String,
    /// Source location when the violation points to a specific heading line.
    pub location: Option<Location>,
}

/// Validate every task against `config`, returning all violations.
#[must_use]
pub fn validate(tasks: &[Task], config: &Config) -> Vec<Violation> {
    let mut out = Vec::new();
    check_id_uniqueness(tasks, &mut out);
    let required = RequiredRules::compile(config);
    for task in tasks {
        check_id_format(task, config, &mut out);
        check_status(task, config, &mut out);
        for (name, field) in &config.tasks.fields {
            check_enum_field(
                task,
                name,
                field,
                required.field(name, task, config),
                &mut out,
            );
        }
        out.extend(check_priority(
            task,
            &config.tasks.priority,
            required.priority(task, config),
        ));
        out.extend(check_numeric(
            task,
            "difficulty",
            &task.difficulty,
            &config.tasks.difficulty,
            required.difficulty(task, config),
        ));
        check_estimate(
            task,
            &config.tasks.estimate,
            required.estimate(task, config),
            &mut out,
        );
        check_tags(task, config, &mut out);
        crate::validate_recur::check_all(task, config, &mut out);
    }
    crate::validate_links::check_all(tasks, config, &mut out);
    out
}

fn check_id_uniqueness(tasks: &[Task], out: &mut Vec<Violation>) {
    let mut seen: HashMap<&str, &Task> = HashMap::new();
    for task in tasks {
        if let Some(first) = seen.get(task.id.as_str()) {
            out.push(Violation {
                task_id: task.id.clone(),
                field: "id".to_owned(),
                message: format!(
                    "duplicate id `{}`; first seen in `{}`",
                    task.id,
                    first.file.display()
                ),
                location: None,
            });
        } else {
            seen.insert(&task.id, task);
        }
    }
}

fn check_id_format(task: &Task, config: &Config, out: &mut Vec<Violation>) {
    let format = config.tasks.id.format;
    if format == IdFormat::String {
        return;
    }
    match Uuid::parse_str(&task.id) {
        Err(_) => push(
            out,
            task,
            "id",
            format!("id `{}` is not a valid UUID", task.id),
        ),
        Ok(uuid) => match format {
            IdFormat::UuidV4 if uuid.get_version_num() != 4 => push(
                out,
                task,
                "id",
                format!("id `{}` must be a UUID v4", task.id),
            ),
            IdFormat::UuidV7 if uuid.get_version_num() != 7 => push(
                out,
                task,
                "id",
                format!("id `{}` must be a UUID v7", task.id),
            ),
            _ => {}
        },
    }
}

pub(crate) fn push(out: &mut Vec<Violation>, task: &Task, field: &str, message: String) {
    out.push(violation(task, field, message));
}

fn violation(task: &Task, field: &str, message: String) -> Violation {
    Violation {
        task_id: task.id.clone(),
        field: field.to_owned(),
        message,
        location: None,
    }
}

fn check_status(task: &Task, config: &Config, out: &mut Vec<Violation>) {
    let legend = &config.tasks.status;
    if legend.is_empty() {
        return;
    }
    match &task.status {
        None => push(out, task, "status", "missing required status".to_owned()),
        Some(status) if !legend.contains(&status.name) => push(
            out,
            task,
            "status",
            format!("`{}` is not a configured status", status.name),
        ),
        Some(status) => {
            check_status_reachable(task, legend, &status.name, out);
            check_status_note(task, legend, status, out);
        }
    }
}

/// Flag a task sitting in a status whose legend entry declares `note =
/// "required"` (see [`crate::config::LegendEntry::note_required`], the same
/// [`crate::link::NoteRule`] primitive typed `links:` edges use) but whose
/// status carries no note, or only a blank one.
fn check_status_note(task: &Task, legend: &Legend, status: &Status, out: &mut Vec<Violation>) {
    let Some(entry) = legend.entry(&status.name) else {
        return;
    };
    let blank = status.note.as_deref().unwrap_or("").trim().is_empty();
    if entry.note_required() && blank {
        push(
            out,
            task,
            "status",
            format!("`{}` requires a note", status.name),
        );
    }
}

/// Flag a task sitting in a status no legal path reaches from the initial state.
/// Only a reachability check — the snapshot carries no history, so this cannot
/// prove the task walked a legal path, only that its current state is not a
/// structurally dead one (a hand-edit into an orphan, or a since-changed graph).
/// No-op when the graph is absent (any-to-any) or has no initial state.
fn check_status_reachable(
    task: &Task,
    legend: &crate::config::Legend,
    name: &str,
    out: &mut Vec<Violation>,
) {
    if !legend.transitions_declared() {
        return;
    }
    let reachable = legend.reachable();
    if !reachable.is_empty() && !reachable.contains(name) {
        push(
            out,
            task,
            "status",
            format!("`{name}` is unreachable under the declared status transitions"),
        );
    }
}

fn check_enum_field(
    task: &Task,
    name: &str,
    field: &FieldConfig,
    required: bool,
    out: &mut Vec<Violation>,
) {
    let value = match name {
        "kind" => task.kind.as_deref(),
        _ => return,
    };
    match value {
        None if required => {
            push(out, task, name, "required field is missing".to_owned());
        }
        Some(value) if !field.values.is_empty() && !field.values.iter().any(|v| v == value) => {
            push(
                out,
                task,
                name,
                format!("`{value}` is not an allowed value"),
            );
        }
        _ => {}
    }
}

/// Validate a numeric-field value: presence when `required`, a known label, and
/// in-scale bounds. Returns at most one violation.
fn check_numeric(
    task: &Task,
    name: &str,
    value: &Option<NumValue>,
    config: &NumericField,
    required: bool,
) -> Option<Violation> {
    let Some(value) = value else {
        return required.then(|| violation(task, name, "required field is missing".to_owned()));
    };
    check_in_scale(task, name, value, config)
}

/// Resolve a value and check it lands on the scale. Returns at most one violation.
fn check_in_scale(
    task: &Task,
    name: &str,
    value: &NumValue,
    config: &NumericField,
) -> Option<Violation> {
    match config.resolve(value) {
        None => Some(violation(
            task,
            name,
            format!("`{}` is not a known {name} value", value.as_display()),
        )),
        Some(point) if point < config.min || point > config.max => Some(violation(
            task,
            name,
            format!(
                "{} is outside the {name} scale {}..{}",
                format_num(point),
                format_num(config.min),
                format_num(config.max),
            ),
        )),
        Some(_) => None,
    }
}

/// Validate a task's priority: presence when `required`, then either a framework
/// invocation (active, known, valid inputs, in-scale score) or a manual value
/// (allowed only when no framework is strict). Returns at most one violation.
fn check_priority(task: &Task, config: &PriorityConfig, required: bool) -> Option<Violation> {
    let Some(value) = &task.priority else {
        return required
            .then(|| violation(task, "priority", "required field is missing".to_owned()));
    };
    match value {
        NumValue::Framework { name, inputs } => check_framework(task, config, name, inputs),
        NumValue::Num(_) | NumValue::Label(_) => check_manual_priority(task, config, value),
    }
}

fn check_framework(
    task: &Task,
    config: &PriorityConfig,
    name: &str,
    inputs: &BTreeMap<String, InputValue>,
) -> Option<Violation> {
    match config.framework.as_deref() {
        None => {
            return Some(violation(
                task,
                "priority",
                format!("framework `{name}` used but no priority framework is configured"),
            ));
        }
        Some(active) if active != name => {
            return Some(violation(
                task,
                "priority",
                format!("framework `{name}` is not the active framework `{active}`"),
            ));
        }
        Some(_) => {}
    }
    let Some(framework) = config.frameworks.get(name) else {
        return Some(violation(
            task,
            "priority",
            format!("unknown framework `{name}`"),
        ));
    };
    if let Some(problem) = framework.check_input_values(inputs).into_iter().next() {
        return Some(violation(task, "priority", problem));
    }
    match framework.score_of(inputs) {
        Err(message) => Some(violation(task, "priority", message)),
        Ok(score) => score_in_scale(task, config, score),
    }
}

fn score_in_scale(task: &Task, config: &PriorityConfig, score: f64) -> Option<Violation> {
    (score < config.field.min || score > config.field.max).then(|| {
        violation(
            task,
            "priority",
            format!(
                "{} is outside the priority scale {}..{}",
                format_num(score),
                format_num(config.field.min),
                format_num(config.field.max),
            ),
        )
    })
}

fn check_manual_priority(
    task: &Task,
    config: &PriorityConfig,
    value: &NumValue,
) -> Option<Violation> {
    if let Some(active) = &config.framework
        && !config.allow_manual
    {
        return Some(violation(
            task,
            "priority",
            format!(
                "`{}` is a manual priority but framework `{active}` is strict (allow-manual = false)",
                value.as_display()
            ),
        ));
    }
    check_in_scale(task, "priority", value, &config.field)
}

fn check_estimate(task: &Task, config: &EstimateConfig, required: bool, out: &mut Vec<Violation>) {
    if task.estimate.is_empty() {
        if required {
            push(
                out,
                task,
                "estimate",
                "required field is missing".to_owned(),
            );
        }
        return;
    }
    if config.units.is_empty() {
        return;
    }
    for unit in task.estimate.keys() {
        if !config.units.contains(unit) {
            push(
                out,
                task,
                "estimate",
                format!("`{unit}` is not an allowed estimate unit"),
            );
        }
    }
}

fn check_tags(task: &Task, config: &Config, out: &mut Vec<Violation>) {
    let allowed = &config.tasks.tags.allowed;
    if allowed.is_empty() {
        return;
    }
    for tag in &task.tags {
        if !allowed.contains(tag) {
            push(out, task, "tags", format!("`{tag}` is not an allowed tag"));
        }
    }
    for tag in &task.derived_tags {
        if !allowed.contains(tag) {
            push(
                out,
                task,
                "tags",
                format!("`{tag}` derived from folder is not an allowed tag"),
            );
        }
    }
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "validate_tests_fields.rs"]
mod tests_fields;
