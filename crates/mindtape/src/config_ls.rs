//! `[ls]` config: `mindtape ls` rendering defaults.
//!
//! Filter / sort defaults, the global date format, and per-column display
//! settings (visibility, order). The column-name vocabulary ([`LS_COLUMNS`])
//! lives here as the single validated source shared by `[ls.columns]`,
//! `[ls] columns-order`, and any consumer renderer.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

/// Canonical `mindtape ls` column names, in built-in render order. The single
/// validated vocabulary shared by `[ls.columns]` and `[ls] columns-order` (and
/// any consumer): `#` is the handle column, `title` the flex column.
pub const LS_COLUMNS: [&str; 6] = ["#", "status", "ready", "priority", "due", "title"];

/// `[ls]` — `mindtape ls` defaults.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LsConfig {
    /// Filter applied by `mindtape ls` when the caller passes none.
    pub default_filter: Option<String>,
    /// Default sort chain for `mindtape ls` when the caller passes no `--sort`.
    /// Same spec grammar as `--sort`: `"field"` or `"field:asc"` / `"field:desc"`.
    /// Validated at config load. When absent the shipped built-in chain applies.
    pub default_sort: Vec<String>,
    /// Global date display format applied to every rendered date column.
    /// Tokens (longest-match-first): `YYYY` 4-digit year, `YY` 2-digit year,
    /// `MMMM` full month name, `MMM` abbreviated month, `MM` zero-padded month,
    /// `M` bare month, `DD` zero-padded day, `D` bare day.
    /// Non-token characters pass through literally.
    /// Default `None` renders ISO `YYYY-MM-DD`.
    pub date_format: Option<String>,
    /// Per-column display settings, keyed by a [`LS_COLUMNS`] name. A map value
    /// (not a scalar) so future column knobs extend [`ColumnConfig`] without a
    /// breaking shape change. Unlisted columns take the defaults.
    pub columns: BTreeMap<String, ColumnConfig>,
    /// Explicit column order. Empty means the built-in [`LS_COLUMNS`] order.
    /// When set, it both selects and orders: only the listed columns render, in
    /// the given sequence. Validated against [`LS_COLUMNS`] at config load.
    pub columns_order: Vec<String>,
}

/// `[ls.columns.<name>]` — one column's display settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ColumnConfig {
    /// Whether the column renders. `Auto` hides it when it carries nothing
    /// significant; `Visible` always renders it; `Hidden` never does.
    pub visibility: Visibility,
}

/// A column's visibility policy.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Render only when the column is significant (non-empty / contested).
    #[default]
    Auto,
    /// Always render, even when every cell is blank.
    Visible,
    /// Never render, even when populated.
    Hidden,
}

/// Validate `[ls.columns]` keys and `[ls] columns-order` against [`LS_COLUMNS`]:
/// unknown column names, and duplicates within `columns-order`, are reported.
#[must_use]
pub fn validate_ls_columns(ls: &LsConfig) -> Vec<String> {
    let legal = LS_COLUMNS.join(", ");
    let mut problems = Vec::new();
    for name in ls.columns.keys() {
        if !LS_COLUMNS.contains(&name.as_str()) {
            problems.push(format!(
                "ls.columns: unknown column `{name}`, expected one of {legal}"
            ));
        }
    }
    let mut seen = BTreeSet::new();
    for name in &ls.columns_order {
        if !LS_COLUMNS.contains(&name.as_str()) {
            problems.push(format!(
                "ls.columns-order: unknown column `{name}`, expected one of {legal}"
            ));
        } else if !seen.insert(name.as_str()) {
            problems.push(format!("ls.columns-order: duplicate column `{name}`"));
        }
    }
    problems
}
