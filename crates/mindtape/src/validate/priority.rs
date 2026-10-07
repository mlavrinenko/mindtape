//! Priority validation, split out of `validate/mod.rs` (file-size gate): the
//! framework-vs-manual dispatch, the active/known/valid-inputs checks a
//! framework invocation must clear, and the strict-mode policy a manual value
//! must clear instead.
//!
//! Both paths bottom out on [`crate::validate::check_in_scale`], the same scale
//! machinery difficulty (and any other plain numeric field) also uses — that
//! helper stays in `validate/mod.rs`, shared rather than duplicated.

use std::collections::BTreeMap;

use crate::config::PriorityConfig;
use crate::model::Task;
use crate::model::numeric::{InputValue, NumValue, format_num};
use crate::validate::{Violation, check_in_scale, violation};

/// Validate a task's priority: presence when `required`, then either a framework
/// invocation (active, known, valid inputs, in-scale score) or a manual value
/// (allowed only when no framework is strict). Returns at most one violation.
pub(crate) fn check_priority(
    task: &Task,
    config: &PriorityConfig,
    required: bool,
) -> Option<Violation> {
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

#[cfg(test)]
#[path = "priority_tests.rs"]
mod tests;
