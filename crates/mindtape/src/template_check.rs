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

use typst_edit::line_of;

use crate::add_slots::RequiredSlot;
use crate::severity::Severity;
use crate::task_spans::TaskCallSpans;
use crate::template::discover_slots;
use crate::validate::{Location, Violation};

/// The field a template's own breach reports under.
const TEMPLATE: &str = "template";

/// Where a template lives, as a breach names it.
#[derive(Debug, Clone, Copy)]
pub enum TemplateHome<'a> {
    /// A template file, by its path relative to the project root.
    File(&'a Path),
    /// A built-in template, by name. It has no file.
    Builtin(&'a str),
}

impl TemplateHome<'_> {
    /// A line of this template's source.
    fn at(self, line: usize) -> Location {
        match self {
            Self::File(file) => Location::at(file, line),
            Self::Builtin(template) => Location::Builtin {
                template: template.to_owned(),
                line,
            },
        }
    }

    /// What identifies the template in a breach's `task_id`.
    fn label(self) -> String {
        match self {
            Self::File(file) => file.to_string_lossy().into_owned(),
            Self::Builtin(template) => template.to_owned(),
        }
    }
}

/// One template, at each stage the check sees it.
#[derive(Debug, Clone, Copy)]
pub struct TemplateSources<'a> {
    /// Where the template lives, as a breach names it.
    pub home: TemplateHome<'a>,
    /// The template as authored on disk.
    pub authored: &'a str,
    /// The task source that was validated: slots filled, header stamped.
    pub checked: &'a str,
}

/// One breach per slot of `spliced` — the template with `mt add`'s
/// config-required slots spliced in, each carrying an example from config —
/// that has neither an `example` nor a `default`, placed at the slot in
/// `authored`. A slot `mt add` spliced in (`injected`) names the config table
/// that should carry its example.
#[must_use]
pub fn unfillable_slots(
    home: TemplateHome<'_>,
    authored: &str,
    spliced: &str,
    injected: &[RequiredSlot],
) -> Vec<Violation> {
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
            let message = match injected.iter().find(|slot| slot.field == spec.name) {
                Some(slot) => format!(
                    "slot `{}` has no example; set `example` in {} so `mt check` can fill it",
                    spec.name, slot.config_path
                ),
                None => format!(
                    "slot `{}` has no example or default; give it `example:` so `mt check` can fill it",
                    spec.name
                ),
            };
            breach(home, line, message)
        })
        .collect();
    found.sort_by_key(|violation| violation.location.line());
    found
}

/// `violations` of a template's filled task, each moved onto the template's
/// file and line.
#[must_use]
pub fn template_breaches(
    sources: TemplateSources<'_>,
    violations: Vec<Violation>,
) -> Vec<Violation> {
    let spans = TaskCallSpans::parse(sources.authored);
    violations
        .into_iter()
        .map(|violation| relocate(sources, &spans, violation))
        .collect()
}

/// A breach of a template that never became a task: it did not evaluate, or
/// `mt add` could not prepare it. Placed at the template's `task.with(…)`.
#[must_use]
pub fn template_failure(home: TemplateHome<'_>, authored: &str, message: String) -> Violation {
    breach(home, call_line(authored), message)
}

fn breach(home: TemplateHome<'_>, line: usize, message: String) -> Violation {
    Violation {
        task_id: home.label(),
        field: TEMPLATE.to_owned(),
        message,
        location: home.at(line),
        severity: Severity::Error,
        reads: Some(std::collections::BTreeSet::new()),
    }
}

/// Point `violation` at the template: where the shared spans place it in the
/// authored file ([`TaskCallSpans::relocate`]), else its `task.with(…)`.
fn relocate(
    sources: TemplateSources<'_>,
    spans: &TaskCallSpans,
    mut violation: Violation,
) -> Violation {
    let line = spans
        .relocate(
            sources.authored,
            sources.checked,
            &violation.field,
            violation.location.line(),
        )
        .unwrap_or_else(|| call_line(sources.authored));
    violation.location = sources.home.at(line);
    violation
}

/// The line `task.with(…)` opens on, or the first line when there is none.
fn call_line(source: &str) -> usize {
    TaskCallSpans::parse(source).call_line().unwrap_or(1)
}

#[cfg(test)]
#[path = "template_check_tests.rs"]
mod tests;
