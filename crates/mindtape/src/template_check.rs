//! What `mt check` makes of a template
//! (`truth/tasks/check-templates-file-healthy-tasks.typ`).
//!
//! A template is checked as the task it files when every slot holds its
//! `example`, else its `default` ([`crate::fill_examples`]), and every breach
//! of that task is the template's. A slot with neither cannot be filled, so it
//! is a breach of its own ([`unfillable_slots`]) and the template goes
//! unchecked until it has one.
//!
//! The candidate validation itself is [`crate::check_candidate`]; the CLI
//! splices the config-required slots and the add-owned header in between,
//! exactly as `mt add` does. This module places what the validation says onto
//! the template's own file and lines.

use std::path::Path;

use typst_edit::{Call, find_method_calls, line_of};

use crate::severity::Severity;
use crate::template::discover_slots;
use crate::validate::{Location, Violation};

/// The field a template's own breach reports under.
const TEMPLATE: &str = "template";

/// One template, at each stage the check sees it.
#[derive(Debug, Clone, Copy)]
pub struct TemplateSources<'a> {
    /// The template's path relative to the project root, as a breach names it.
    pub file: &'a Path,
    /// The template as authored on disk.
    pub authored: &'a str,
    /// The task source that was validated: slots filled, header stamped.
    pub checked: &'a str,
}

/// One breach per slot of `spliced` — the template with `mt add`'s
/// config-required slots spliced in, each carrying an example from config —
/// that has neither an `example` nor a `default`, placed at the slot in
/// `authored`.
#[must_use]
pub fn unfillable_slots(file: &Path, authored: &str, spliced: &str) -> Vec<Violation> {
    let authored_slots = discover_slots(authored);
    let mut found: Vec<Violation> = discover_slots(spliced)
        .into_iter()
        .filter(|spec| spec.example.is_none() && spec.default.is_none())
        .map(|spec| {
            let line = (authored_slots.iter())
                .find(|slot| slot.name == spec.name)
                .map_or_else(
                    || call_line(authored),
                    |slot| line_of(authored, slot.range.start),
                );
            let message = format!(
                "slot `{}` has no example or default; give it `example:` so `mt check` can fill it",
                spec.name
            );
            breach(file, line, message)
        })
        .collect();
    found.sort_by_key(|violation| violation.location.line);
    found
}

/// `violations` of a template's filled task, each moved onto the template's
/// file and line.
#[must_use]
pub fn template_breaches(
    sources: TemplateSources<'_>,
    violations: Vec<Violation>,
) -> Vec<Violation> {
    violations
        .into_iter()
        .map(|violation| relocate(sources, violation))
        .collect()
}

/// A breach of a template that never became a task: it did not evaluate, or
/// `mt add` could not prepare it. Placed at the template's `task.with(…)`.
#[must_use]
pub fn template_failure(file: &Path, authored: &str, message: String) -> Violation {
    breach(file, call_line(authored), message)
}

fn breach(file: &Path, line: usize, message: String) -> Violation {
    Violation {
        task_id: file.to_string_lossy().into_owned(),
        field: TEMPLATE.to_owned(),
        message,
        location: Location {
            file: file.to_path_buf(),
            line,
        },
        severity: Severity::Error,
        reads: Some(std::collections::BTreeSet::new()),
    }
}

/// The first segment of a field path: `estimate` of `estimate.days`.
fn root_of(name: &str) -> &str {
    name.split(['.', '[']).next().unwrap_or(name)
}

/// Point `violation` at the template: the line of the argument it is about,
/// else the authored line reading as the checked one, else `task.with(…)`.
fn relocate(sources: TemplateSources<'_>, mut violation: Violation) -> Violation {
    let authored = sources.authored;
    let by_field = task_call(authored).and_then(|call| {
        call.named(root_of(&violation.field))
            .map(|arg| line_of(authored, arg.value_range.start))
    });
    let by_text = || {
        let wanted = sources
            .checked
            .lines()
            .nth(violation.location.line.saturating_sub(1))?
            .trim();
        (wanted.len() > 1)
            .then(|| authored.lines().position(|line| line.trim() == wanted))
            .flatten()
            .map(|index| index + 1)
    };
    violation.location.line = by_field
        .or_else(by_text)
        .unwrap_or_else(|| call_line(authored));
    violation.location.file = sources.file.to_path_buf();
    violation
}

fn task_call(source: &str) -> Option<Call> {
    find_method_calls(source, "task", "with").into_iter().next()
}

/// The line `task.with(…)` opens on, or the first line when there is none.
fn call_line(source: &str) -> usize {
    task_call(source).map_or(1, |call| line_of(source, call.range.start))
}

#[cfg(test)]
#[path = "template_check_tests.rs"]
mod tests;
