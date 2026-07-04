//! `[[tasks.status]]` — the status legend: an ordered, displayable vocabulary.
//!
//! Split out of `config.rs` to keep that file under the size gate; re-exported
//! at `crate::config::{Legend, LegendEntry}`.

use serde::Deserialize;

/// The status legend: an ordered, displayable vocabulary.
///
/// Each entry binds a value to optional display attributes (glyph, colour,
/// label) plus the `terminal` flag. The legend defines the legal values, their
/// lifecycle order, and how each renders. Empty = unconstrained (though an
/// omitted `[[tasks.status]]` section inherits the embedded floor's six-value
/// legend through `parse_config` — see `crate::config`). The same primitive
/// will serve other discrete enum fields (kind, tags) as they gain display.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Legend {
    /// Ordered entries, in lifecycle / display order.
    pub entries: Vec<LegendEntry>,
}

/// One `[[tasks.status]]` entry: a value plus its display attributes.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct LegendEntry {
    /// The value this entry describes (e.g. a status name).
    pub value: String,
    /// Optional decorative glyph (emoji or ASCII).
    pub glyph: Option<String>,
    /// Optional colour: a named ANSI colour or `#RRGGBB`.
    pub color: Option<String>,
    /// Optional display label; falls back to `value`.
    pub label: Option<String>,
    /// Whether this value ends a task; required fields relax once terminal.
    pub terminal: bool,
}

impl Legend {
    /// Whether the legend constrains nothing (no entries).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether `value` is a legal entry.
    #[must_use]
    pub fn contains(&self, value: &str) -> bool {
        self.entries.iter().any(|entry| entry.value == value)
    }

    /// The entry describing `value`, if any.
    #[must_use]
    pub fn entry(&self, value: &str) -> Option<&LegendEntry> {
        self.entries.iter().find(|entry| entry.value == value)
    }

    /// The values flagged `terminal`.
    #[must_use]
    pub fn terminal(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|entry| entry.terminal)
            .map(|entry| entry.value.clone())
            .collect()
    }
}
