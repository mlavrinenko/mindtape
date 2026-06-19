//! `Field` implementations for legend-backed discrete fields.
//!
//! Provides [`Status`]; `kind` and `tags` adopt the same `legend_cell` pattern
//! when they gain display columns.

use std::borrow::Cow;

use mindtape::Task;
use mindtape::config::{Config, Legend};

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
#[allow(clippy::unwrap_used)]
mod tests {
    use mindtape::config::parse_config;

    use super::Status;
    use crate::render::field::Field;

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
        let config = parse_config("").unwrap();
        let cell = Status::cell_spec("wip", &config);
        assert_eq!(cell.text, "wip");
        assert!(cell.glyph.is_none());
        assert!(cell.color.is_none());
    }
}
