//! Config-driven validation of loaded tasks.
//!
//! Enforces the declared status lifecycle, enum field constraints, the estimate
//! unit allow-list, the `required` rules (a filter expression per field, see
//! [`crate::config::required`]), the tag allow-list, the priority framework
//! scoring/manual-override policy (see [`crate::validate::priority`]), and the
//! typed `links:` edges (resolution, cardinality, notes, cycles — see
//! [`crate::validate::links`]), and that each `waits-on` entry names a declared
//! checker with a shell-safe id.
//!
//! # Files
//!
//! - `candidate.rs`: validates a rendered task before it is written.
//! - `links.rs`: the typed `links:` edge checks.
//! - `priority.rs`: priority framework and manual-override checks.
//! - `recur/mod.rs`: the task-level `recur` policy check.
//! - `severity.rs`: how loudly a config-declared breach reports.
//! - `violation.rs`: what a check reports and where in the source it points.
//! - `waits.rs`: the `waits-on` entry check.

pub mod candidate;
pub(crate) mod links;
pub(crate) mod priority;
pub(crate) mod recur;
pub mod severity;
pub(crate) mod violation;
pub(crate) mod waits;

use std::collections::HashMap;

use uuid::Uuid;

use crate::config::{Config, IdFormat, Legend, RequiredRules};
use crate::model::field_descriptor::Descriptor;
use crate::model::fields;
use crate::model::numeric::{NumValue, NumericField, format_num};
use crate::model::{Status, Task};
use crate::validate::severity::Severity;
pub use crate::validate::violation::{Location, Violation, ViolationKind};

/// Validate every task against `config`, returning all violations.
#[must_use]
pub fn validate(tasks: &[Task], config: &Config) -> Vec<Violation> {
    let mut out = Vec::new();
    check_id_uniqueness(tasks, &mut out);
    let required = RequiredRules::compile(config);
    let described = crate::model::field_descriptor::descriptors(config);
    for task in tasks {
        check_shape(task, &mut out);
        check_id_format(task, config, &mut out);
        check_status(task, config, &mut out);
        for field in &described {
            check_field(task, config, field, &mut out);
        }
        out.extend(crate::validate::priority::check_priority(
            task,
            &config.tasks.priority,
            still_required(required.priority(task, config), task, "priority"),
        ));
        out.extend(check_numeric(
            task,
            "difficulty",
            &task.difficulty,
            &config.tasks.difficulty,
            still_required(required.difficulty(task, config), task, "difficulty"),
        ));
        let must_estimate = still_required(required.estimate(task, config), task, "estimate");
        check_estimate(task, config, must_estimate, &mut out);
        crate::validate::recur::check_all(task, config, &mut out);
        crate::validate::waits::check_all(task, config, &mut out);
    }
    crate::validate::links::check_all(tasks, config, &mut out);
    crate::rules::check_all(tasks, config, &mut out);
    out
}

/// A field the marker held with the wrong Typst shape (coerced to
/// empty/absent by [`Task::from_marker`]) always fails `check`, regardless of
/// project config — a type mismatch is never a matter of policy.
fn check_shape(task: &Task, out: &mut Vec<Violation>) {
    for err in &task.shape_errors {
        push(out, task, &err.field, err.message.clone());
    }
}

/// Whether the projection reported `name` ITSELF unreadable — a FIELD-level
/// [`crate::model::marker::ShapeError`], whose path is the field name and nothing more.
/// Exact equality is the whole test, and it is exact by that module's path
/// convention rather than by luck: every element path EXTENDS the field name
/// (`status.date`, `tags[1]`, `priority.inputs.ease`), so the one path equal to
/// the name is the one case where the projection came away with nothing.
fn shape_reported(task: &Task, name: &str) -> bool {
    task.shape_errors.iter().any(|err| err.field == name)
}

/// A `required` answer with the fields the marker could not read at all masked
/// out. A FIELD-level shape error means the author DID write the field and
/// [`check_shape`] already reported it naming the type; "missing required" on
/// top of that sends them to add a field that is on the page.
///
/// The mask sits HERE, at the input to each check, rather than as a pass over
/// the finished violations: after the fact the only handle on the report to
/// drop is its rendered message — `required field is missing`, spelled
/// independently by four checks, and `missing required status` by a fifth — so
/// filtering would pin prose no gate holds still and would equally swallow a
/// genuinely different violation worded the same way. The `required` bool is
/// the real input, and the input is what is wrong.
///
/// An ELEMENT error is deliberately NOT masked: it leaves the field itself
/// readable, so the required check on what survives is correct and must fire.
/// `estimate: (days: "two")` is an estimate whose only unit did not parse, and
/// reporting both the element and the empty estimate is two true statements.
fn still_required(required: bool, task: &Task, name: &str) -> bool {
    required && !shape_reported(task, name)
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
                location: task.field_location("id"),
                severity: Severity::Error,
                kind: crate::validate::violation::ViolationKind::Other,
                reads: Some(crate::validate::violation::reads_of_field("id")),
                unknown: None,
                detail: None,
                dict_miss: None,
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

pub(crate) fn violation(task: &Task, field: &str, message: String) -> Violation {
    Violation {
        task_id: task.id.clone(),
        field: field.to_owned(),
        message,
        location: task.field_location(field),
        severity: Severity::Error,
        kind: crate::validate::violation::ViolationKind::Other,
        reads: Some(crate::validate::violation::reads_of_field(field)),
        unknown: None,
        detail: None,
        dict_miss: None,
    }
}

fn check_status(task: &Task, config: &Config, out: &mut Vec<Violation>) {
    let legend = &config.tasks.status;
    if legend.is_empty() {
        return;
    }
    match &task.status {
        // `status:` never passes through `RequiredRules` — a non-empty legend
        // requires it unconditionally — so it takes the same mask by hand.
        // See [`still_required`] for why the mask, and not a filter over the
        // finished violations.
        None if shape_reported(task, "status") => {}
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

/// One described field: a missing required value, and a held value outside
/// the allowed ones for the tasks the field's `values-when` selects.
fn check_field(task: &Task, config: &Config, field: &Descriptor, out: &mut Vec<Violation>) {
    let name = field.name;
    let Some(value) = fields::held(task, name) else {
        if still_required(field.must.applies(task, config), task, name) {
            push(out, task, name, "required field is missing".to_owned());
        }
        return;
    };
    if field.holds(task, config) {
        for problem in field.breaches(task, &value) {
            let mut breach = violation(task, name, problem.text);
            breach.unknown = problem.unknown;
            out.push(breach);
        }
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
///
/// Shared by [`check_numeric`] (difficulty and any other plain numeric field)
/// and [`crate::validate::priority`] (priority's manual-value path and its
/// framework score both land on the same [`NumericField`] scale machinery).
pub(crate) fn check_in_scale(
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

/// Validate a task's estimate: presence when `required`, allowed unit names,
/// and a `count(...)` unit that resolved. A quantity is never checked here — a cap is a `[[tasks.rules]]`
/// entry reading `estimate.<unit>`.
///
/// Units and counts are skipped once a task is terminal — it cannot be re-estimated any
/// more, so retiring a unit is one edit to `units` rather than a rewrite of
/// every closed task that ever carried it.
fn check_estimate(task: &Task, config: &Config, required: bool, out: &mut Vec<Violation>) {
    let estimate = &config.tasks.estimate;
    if task.estimate.is_empty() && task.estimate_counts.is_empty() {
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
    if task.is_terminal(&config.tasks.status.terminal()) {
        return;
    }
    for (unit, count) in &task.estimate_counts {
        if let Some(problem) = &count.problem {
            push(out, task, &format!("estimate.{unit}"), problem.clone());
        }
    }
    let units: std::collections::BTreeSet<&String> = task
        .estimate
        .keys()
        .chain(task.estimate_counts.keys())
        .collect();
    for unit in units {
        if !estimate.units.is_empty() && !estimate.units.contains(unit) {
            push(
                out,
                task,
                "estimate",
                format!("`{unit}` is not an allowed estimate unit"),
            );
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_fields;

#[cfg(test)]
mod tests_id;

#[cfg(test)]
mod tests_shape;

#[cfg(test)]
mod tests_extras;

#[cfg(test)]
mod tests_when;
