//! Config-driven validation of loaded tasks.
//!
//! Enforces the declared status lifecycle, enum field constraints, the estimate
//! unit allow-list, required-unless-terminal, the tag allow-list, and link
//! resolution (a `depends-on`/`relates-to` target must name a loaded task).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::config::{Config, EstimateConfig, FieldConfig, IdFormat, Requirement};
use crate::model::{Task, resolve_link_path};
use crate::numeric::{NumValue, NumericField, format_num};

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
    let files: HashSet<&Path> = tasks.iter().map(|task| task.file.as_path()).collect();
    let mut out = Vec::new();
    check_id_uniqueness(tasks, &mut out);
    let terminal_values = config.tasks.status.terminal();
    for task in tasks {
        let terminal = task.is_terminal(&terminal_values);
        check_id_format(task, config, &mut out);
        check_status(task, config, &mut out);
        for (name, field) in &config.tasks.fields {
            check_enum_field(task, name, field, terminal, &mut out);
        }
        out.extend(check_numeric(
            task,
            "priority",
            &task.priority,
            &config.tasks.priority,
            terminal,
        ));
        out.extend(check_numeric(
            task,
            "difficulty",
            &task.difficulty,
            &config.tasks.difficulty,
            terminal,
        ));
        check_estimate(task, &config.tasks.estimate, terminal, &mut out);
        check_tags(task, config, &mut out);
        check_links(task, &files, &mut out);
    }
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

fn push(out: &mut Vec<Violation>, task: &Task, field: &str, message: String) {
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
        Some(_) => {}
    }
}

fn check_enum_field(
    task: &Task,
    name: &str,
    field: &FieldConfig,
    terminal: bool,
    out: &mut Vec<Violation>,
) {
    let value = match name {
        "kind" => task.kind.as_deref(),
        _ => return,
    };
    match value {
        None if is_required(field.required, terminal) => {
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

/// Validate a numeric-field value: required-unless-terminal, a known label, and
/// in-scale bounds. Returns at most one violation.
fn check_numeric(
    task: &Task,
    name: &str,
    value: &Option<NumValue>,
    config: &NumericField,
    terminal: bool,
) -> Option<Violation> {
    let Some(value) = value else {
        return is_required(config.required, terminal)
            .then(|| violation(task, name, "required field is missing".to_owned()));
    };
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

fn check_estimate(task: &Task, config: &EstimateConfig, terminal: bool, out: &mut Vec<Violation>) {
    if task.estimate.is_empty() {
        if is_required(config.required, terminal) {
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
}

fn check_links(task: &Task, files: &HashSet<&Path>, out: &mut Vec<Violation>) {
    for (field, links) in [
        ("depends-on", &task.depends_on),
        ("relates-to", &task.relates_to),
        ("supersedes", &task.supersedes),
        ("duplicates", &task.duplicates),
    ] {
        for target in links {
            let resolved = resolve_link_path(&task.file, target);
            let found = resolved.as_deref().is_some_and(|p| files.contains(p));
            if !found {
                let display = resolved
                    .as_deref()
                    .unwrap_or(Path::new(target))
                    .display()
                    .to_string();
                push(
                    out,
                    task,
                    field,
                    format!("link target `{display}` not found"),
                );
            }
        }
    }
}

fn is_required(requirement: Requirement, terminal: bool) -> bool {
    match requirement {
        Requirement::No => false,
        Requirement::Always => true,
        Requirement::NonTerminal => !terminal,
    }
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
