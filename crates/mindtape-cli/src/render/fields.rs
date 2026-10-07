//! `Field` implementations for the displayable task fields.
//!
//! Legend-backed discrete fields ([`Status`]) resolve through the status
//! legend; numeric fields ([`Priority`], [`Difficulty`]) resolve through their
//! [`NumericField`] classification. `kind` and `tags` adopt the legend pattern
//! when they gain display columns.

use std::borrow::Cow;

use mindtape::Task;
use mindtape::config::{Config, Legend};
use mindtape::model::numeric::{NumValue, NumericField};

use super::color::parse_color;
use super::field::{CellSpec, Field};

/// The `status` field, displayed through its `[[tasks.status]]` legend.
pub struct Status;

impl Field for Status {
    const NAME: &'static str = "STATUS";

    fn values(task: &Task) -> Vec<Cow<'_, str>> {
        task.status
            .as_ref()
            .map(|status| Cow::Borrowed(status.name.as_str()))
            .into_iter()
            .collect()
    }

    fn cell_spec(value: &str, config: &Config) -> CellSpec {
        legend_cell(value, &config.tasks.status)
    }
}

/// The `priority` field, displayed through its `[tasks.priority]` band
/// classification.
pub struct Priority;

impl Field for Priority {
    const NAME: &'static str = "PRIORITY";

    fn values(task: &Task) -> Vec<Cow<'_, str>> {
        numeric_values(task.priority.as_ref())
    }

    fn cell_spec(value: &str, config: &Config) -> CellSpec {
        numeric_cell(value, &config.tasks.priority.field)
    }
}

/// Resolve a task's priority value to a display cell, honouring an active
/// framework (scalar score → bands, or a categorical cell). Bypasses the
/// string round-trip so framework inputs survive.
#[must_use]
pub fn priority_cell_spec(value: &NumValue, config: &Config) -> CellSpec {
    match config.tasks.priority.cell(value) {
        Some(cell) => CellSpec {
            text: cell.label,
            glyph: cell.glyph,
            color: cell.color.as_deref().and_then(parse_color),
        },
        None => CellSpec {
            text: value.as_display(),
            glyph: None,
            color: None,
        },
    }
}

/// The `difficulty` field, displayed through its `[tasks.difficulty]` anchor
/// classification.
pub struct Difficulty;

impl Field for Difficulty {
    const NAME: &'static str = "DIFFICULTY";

    fn values(task: &Task) -> Vec<Cow<'_, str>> {
        numeric_values(task.difficulty.as_ref())
    }

    fn cell_spec(value: &str, config: &Config) -> CellSpec {
        numeric_cell(value, &config.tasks.difficulty)
    }
}

fn numeric_values(value: Option<&NumValue>) -> Vec<Cow<'_, str>> {
    value
        .map(|value| Cow::Owned(value.as_display()))
        .into_iter()
        .collect()
}

/// Resolve and classify `value` against a numeric `field` into a display cell.
fn numeric_cell(value: &str, field: &NumericField) -> CellSpec {
    let parsed = NumValue::parse(value);
    match field.cell(&parsed) {
        Some(cell) => CellSpec {
            text: cell.label,
            glyph: cell.glyph,
            color: cell.color.as_deref().and_then(parse_color),
        },
        None => CellSpec {
            text: parsed.as_display(),
            glyph: None,
            color: None,
        },
    }
}

/// Resolve `value` against `legend` into a display cell (label, glyph, colour).
fn legend_cell(value: &str, legend: &Legend) -> CellSpec {
    match legend.entry(value) {
        Some(entry) => CellSpec {
            text: entry.label.clone().unwrap_or_else(|| value.to_owned()),
            glyph: entry.glyph.clone(),
            color: entry.color.as_deref().and_then(parse_color),
        },
        None => CellSpec {
            text: value.to_owned(),
            glyph: None,
            color: None,
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use mindtape::config::parse_config;
    use mindtape::model::numeric::NumValue;
    use mindtape::test_support::bare_task;

    use super::{Difficulty, Priority, Status};
    use crate::render::field::Field;

    const PRIORITY_CONFIG: &str = "[tasks.priority]\nmin = 0\nmax = 9\n[[tasks.priority.bands]]\nlabel = \"low\"\nmin = 0\nmax = 4\nglyph = \"🟢\"\ncolor = \"green\"\n[[tasks.priority.bands]]\nlabel = \"high\"\nmin = 5\nmax = 9\nglyph = \"🔴\"\ncolor = \"red\"\n";

    const DIFFICULTY_CONFIG: &str = "[tasks.difficulty]\nmin = 0\nmax = 9\n[[tasks.difficulty.anchors]]\nvalue = 0\nlabel = \"junior\"\nglyph = \"🟢\"\ncolor = \"green\"\n[[tasks.difficulty.anchors]]\nvalue = 6\nlabel = \"senior\"\nglyph = \"🟡\"\ncolor = \"yellow\"\n";

    #[test]
    fn priority_cell_uses_band_glyph_and_label() {
        let config = parse_config(PRIORITY_CONFIG).unwrap();
        let cell = Priority::cell_spec("8", &config);
        assert_eq!(cell.text, "high");
        assert_eq!(cell.glyph.as_deref(), Some("🔴"));
        assert!(cell.color.is_some());
    }

    #[test]
    fn difficulty_cell_derives_between_anchors() {
        let config = parse_config(DIFFICULTY_CONFIG).unwrap();
        let cell = Difficulty::cell_spec("7", &config);
        assert_eq!(cell.text, "senior+");
        assert_eq!(cell.glyph.as_deref(), Some("🟡"));
    }

    #[test]
    fn status_cell_uses_legend_glyph_and_label() {
        let config = parse_config(
            "[[tasks.status]]\nvalue = \"wip\"\nglyph = \"🔨\"\ncolor = \"yellow\"\nlabel = \"in progress\"\n",
        )
        .unwrap();
        let cell = Status::cell_spec("wip", &config);
        assert_eq!(cell.glyph.as_deref(), Some("🔨"));
        assert_eq!(cell.text, "in progress");
        assert!(cell.color.is_some());
    }

    #[test]
    fn unknown_status_is_plain() {
        // The embedded floor's legend doesn't recognise this value (unlike
        // "wip", now floored), so it renders plain.
        let config = parse_config("").unwrap();
        let cell = Status::cell_spec("not-a-real-status", &config);
        assert_eq!(cell.text, "not-a-real-status");
        assert!(cell.glyph.is_none());
        assert!(cell.color.is_none());
    }

    #[test]
    fn priority_values_returns_numeric_display_for_set_priority() {
        let mut task = bare_task("t");
        task.priority = Some(NumValue::Num(8.0));
        let vals = Priority::values(&task);
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0].as_ref(), "8");
    }

    #[test]
    fn priority_values_empty_when_unset() {
        let task = bare_task("t");
        let vals = Priority::values(&task);
        assert!(vals.is_empty());
    }

    #[test]
    fn difficulty_values_returns_numeric_display_for_set_difficulty() {
        let mut task = bare_task("t");
        task.difficulty = Some(NumValue::Num(6.0));
        let vals = Difficulty::values(&task);
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0].as_ref(), "6");
    }

    #[test]
    fn difficulty_values_empty_when_unset() {
        let task = bare_task("t");
        let vals = Difficulty::values(&task);
        assert!(vals.is_empty());
    }
}
