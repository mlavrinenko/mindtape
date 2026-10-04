//! Where a task file's `task.with(…)` call and its arguments sit, read once.
//!
//! Every place that points a breach at a line, or slices the prose body after
//! the call, asks this one parse: the loader, [`crate::Task::field_location`]
//! and the template check. Parse-plane only (no eval):
//! [`typst_edit::find_method_calls`] locates the call.

use std::collections::BTreeMap;

use typst_edit::{Call, find_method_calls, line_of};

/// The object whose `.with(…)` call opens a task file: `#show: task.with(…)`.
const TASK: &str = "task";

/// The position of one source's `task.with(…)` call.
///
/// The default is a source with no such call — a task authored as a bare
/// `#metadata((…))`, or built without a source — which has no line to point at
/// and no body to slice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskCallSpans {
    /// One-based line the call opens on.
    call_line: Option<usize>,
    /// Byte offset just past the call's closing `)`, or past its trailing
    /// content block.
    call_end: Option<usize>,
    /// One-based line of each named argument's value.
    fields: BTreeMap<String, usize>,
}

impl TaskCallSpans {
    /// The spans of `source`'s first `task.with(…)` call.
    #[must_use]
    pub fn parse(source: &str) -> Self {
        Self::parse_as(source, TASK)
    }

    /// The spans of `source`'s first `<callee>.with(…)` call, for a task file
    /// that applies the show rule through a helper of its own.
    #[must_use]
    pub fn parse_as(source: &str, callee: &str) -> Self {
        find_method_calls(source, callee, "with")
            .into_iter()
            .next()
            .map_or_else(Self::default, |call| Self::of_call(source, &call))
    }

    fn of_call(source: &str, call: &Call) -> Self {
        let fields = call
            .args
            .iter()
            .filter_map(|arg| {
                let name = arg.name.clone()?;
                Some((name, line_of(source, arg.value_range.start)))
            })
            .collect();
        Self {
            call_line: Some(line_of(source, call.range.start)),
            call_end: Some(call.range.end),
            fields,
        }
    }

    /// Spans built by hand, for a test that needs a task with known lines.
    #[must_use]
    pub fn from_lines(call_line: usize, fields: &[(&str, usize)]) -> Self {
        Self {
            call_line: (call_line > 0).then_some(call_line),
            call_end: None,
            fields: fields
                .iter()
                .map(|(name, line)| ((*name).to_owned(), *line))
                .collect(),
        }
    }

    /// The line the call opens on, or `None` when there is no call.
    #[must_use]
    pub const fn call_line(&self) -> Option<usize> {
        self.call_line
    }

    /// The line of the argument `field` names (a sub-path such as
    /// `estimate.days` or `tags[1]` resolves to its root), else the call's own
    /// line.
    #[must_use]
    pub fn field_line(&self, field: &str) -> Option<usize> {
        let root = field.split(['.', '[']).next().unwrap_or(field);
        self.fields.get(root).copied().or(self.call_line)
    }

    /// Whether the call has an argument named `field`'s root.
    #[must_use]
    pub fn has_field(&self, field: &str) -> bool {
        let root = field.split(['.', '[']).next().unwrap_or(field);
        self.fields.contains_key(root)
    }

    /// The prose after the call in `source`, trimmed; empty when there is no
    /// call. Trimming drops the blank line the emitter re-adds between the
    /// call and the body, keeping the round-trip stable.
    #[must_use]
    pub fn body(&self, source: &str) -> String {
        self.call_end
            .and_then(|end| source.get(end..))
            .unwrap_or_default()
            .trim()
            .to_owned()
    }

    /// Where a breach that `checked` (a rendering of `authored`) places at
    /// `checked_line` of `field` sits in `authored`, which these are the spans
    /// of: the line of the argument itself, else the authored line reading as
    /// the checked one, else the call. `None` when `authored` has no call.
    #[must_use]
    pub fn relocate(
        &self,
        authored: &str,
        checked: &str,
        field: &str,
        checked_line: usize,
    ) -> Option<usize> {
        let call = self.call_line?;
        if self.has_field(field) {
            return self.field_line(field);
        }
        let wanted = checked.lines().nth(checked_line.saturating_sub(1))?.trim();
        let by_text = (wanted.len() > 1)
            .then(|| authored.lines().position(|line| line.trim() == wanted))
            .flatten()
            .map(|index| index + 1);
        Some(by_text.unwrap_or(call))
    }
}

#[cfg(test)]
#[path = "task_spans_tests.rs"]
mod tests;
