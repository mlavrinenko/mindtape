//! Config-driven validation of loaded tasks.
//!
//! Enforces the declared status lifecycle, enum/number field constraints,
//! required-unless-terminal, the tag allow-list, and link resolution (a
//! `depends-on`/`relates-to` target must name a loaded task).

use std::collections::HashSet;

use crate::config::{Config, FieldConfig, Requirement};
use crate::model::Task;

/// A single validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Offending task id.
    pub task_id: String,
    /// Field or aspect at fault (`status`, `priority`, `tags`, `depends-on`, …).
    pub field: String,
    /// Human-readable message.
    pub message: String,
}

/// Validate every task against `config`, returning all violations.
#[must_use]
pub fn validate(tasks: &[Task], config: &Config) -> Vec<Violation> {
    let ids: HashSet<&str> = tasks.iter().map(|task| task.id.as_str()).collect();
    let mut out = Vec::new();
    for task in tasks {
        let terminal = task.is_terminal(&config.tasks.status.terminal);
        check_status(task, config, &mut out);
        for (name, field) in &config.tasks.fields {
            check_field(task, name, field, terminal, &mut out);
        }
        check_tags(task, config, &mut out);
        check_links(task, &ids, &mut out);
    }
    out
}

fn push(out: &mut Vec<Violation>, task: &Task, field: &str, message: String) {
    out.push(Violation {
        task_id: task.id.clone(),
        field: field.to_owned(),
        message,
    });
}

fn check_status(task: &Task, config: &Config, out: &mut Vec<Violation>) {
    let values = &config.tasks.status.values;
    if values.is_empty() {
        return;
    }
    match &task.status {
        None => push(out, task, "status", "missing required status".to_owned()),
        Some(status) if !values.contains(&status.name) => push(
            out,
            task,
            "status",
            format!("`{}` is not a configured status", status.name),
        ),
        Some(_) => {}
    }
}

fn check_field(
    task: &Task,
    name: &str,
    field: &FieldConfig,
    terminal: bool,
    out: &mut Vec<Violation>,
) {
    if name == "estimate-hours" {
        check_number_field(task, name, field, terminal, out);
    } else {
        check_enum_field(task, name, field, terminal, out);
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
        "priority" => task.priority.as_deref(),
        "grade" => task.grade.as_deref(),
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

fn check_number_field(
    task: &Task,
    name: &str,
    field: &FieldConfig,
    terminal: bool,
    out: &mut Vec<Violation>,
) {
    let Some(value) = task.estimate_hours else {
        if is_required(field.required, terminal) {
            push(out, task, name, "required field is missing".to_owned());
        }
        return;
    };
    if let Some(min) = field.min
        && value < min
    {
        push(out, task, name, format!("{value} is below minimum {min}"));
    }
    if let Some(max) = field.max
        && value > max
    {
        push(out, task, name, format!("{value} is above maximum {max}"));
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

fn check_links(task: &Task, ids: &HashSet<&str>, out: &mut Vec<Violation>) {
    for (field, links) in [
        ("depends-on", &task.depends_on),
        ("relates-to", &task.relates_to),
    ] {
        for target in links {
            if !ids.contains(target.as_str()) {
                push(
                    out,
                    task,
                    field,
                    format!("link target `{target}` not found"),
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
