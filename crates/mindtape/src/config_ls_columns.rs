//! `[ls.columns.<name>]` — one `mt ls` column's display settings, and the
//! validation of the column-name vocabulary they are keyed by.
//!
//! Split out of [`crate::config_ls`] so that file stays under the repo's Rust
//! line cap; the `[ls]` block itself, the [`crate::config_ls::LS_COLUMNS`]
//! vocabulary and the `[[ls.rules]]` listing cascade all remain there.
//!
//! # Why a column carries `rules`
//!
//! A column's `visibility` answers "should this column render", but the honest
//! answer sometimes depends on who is reading. The `file` column is the case
//! that motivated this: on a terminal the `#` handle is the ergonomic address
//! and a full path is noise, while in captured output the handle is a position
//! in a view recomputed on every command and the path is the only stable thing
//! to quote. That is a condition, not a visibility — so it is written as one,
//! next to the column it governs, rather than folded into `auto`'s meaning
//! where no reader would look for it.
//!
//! ```toml
//! [[ls.columns.file.rules]]
//! condition = '!tty() && format() == "table"'
//! visibility = "visible"
//! ```

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::condition::Condition;
use crate::config_ls::{LS_COLUMNS, LsConfig};

/// `[ls.columns.<name>]` — one column's display settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ColumnConfig {
    /// Whether the column renders when no `rules` entry matches. `Auto` hides
    /// it when it carries nothing significant; `Visible` always renders it;
    /// `Hidden` never does.
    pub visibility: Visibility,
    /// Conditional overrides, in declaration order. The first entry whose
    /// `condition` holds supplies this column's visibility for the run,
    /// beating the `visibility` key above — rules ARE the override mechanism,
    /// so a matching one winning is what makes them worth writing.
    ///
    /// A project opts out of an inherited rule (the embedded floor ships one
    /// for `file`) by setting `rules = []`, which is explicit and greppable,
    /// rather than by setting `visibility` and wondering why it lost.
    pub rules: Vec<ColumnRule>,
}

/// `[[ls.columns.<name>.rules]]` — one conditional visibility override.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ColumnRule {
    /// Condition expression — the same grammar `[[ls.rules]]` uses (`tty()`,
    /// `format()`, `env()`, `on_branch()`, `now()`, `weekday()`), plus the
    /// four listing predicates only a column resolution can answer:
    /// `nonempty()`, `contested()`, `named()`, `blocked()`. Absent or empty
    /// means the rule always matches.
    pub condition: Option<String>,
    /// The visibility this rule imposes when its condition holds.
    pub visibility: Visibility,
}

/// A column's visibility policy.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
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

/// Validate `[ls.columns]` keys, their `rules` conditions, and `[ls]`
/// `columns-order` against [`LS_COLUMNS`]: unknown column names, duplicates
/// within `columns-order`, and unparseable rule conditions are reported.
///
/// A rule's condition is parsed here so a typo (`ttty()`, a task field name
/// carried over from filter syntax) fails at config load naming the column and
/// the rule index, rather than silently never matching at render time — where
/// a column that does not appear looks exactly like a column that was not
/// meant to.
#[must_use]
pub fn validate_ls_columns(ls: &LsConfig) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, column) in &ls.columns {
        if LS_COLUMNS.contains(&name.as_str()) {
            validate_column_rules(name, column, &mut problems);
        } else {
            problems.push(format!("ls.columns: {}", unknown_column(name)));
        }
    }
    let mut seen = BTreeSet::new();
    for name in &ls.columns_order {
        if !LS_COLUMNS.contains(&name.as_str()) {
            problems.push(format!("ls.columns-order: {}", unknown_column(name)));
        } else if !seen.insert(name.as_str()) {
            problems.push(format!("ls.columns-order: duplicate column `{name}`"));
        }
    }
    problems
}

/// Parse every `[[ls.columns.<name>.rules]]` condition, reporting each failure
/// under the column and index that owns it.
fn validate_column_rules(name: &str, column: &ColumnConfig, problems: &mut Vec<String>) {
    for (index, rule) in column.rules.iter().enumerate() {
        let Some(expr) = &rule.condition else {
            continue;
        };
        if let Err(err) = Condition::parse(expr) {
            problems.push(format!("ls.columns.{name}.rules[{index}].condition: {err}"));
        }
    }
}

/// The `unknown column` message body (no field-path prefix): the fixed
/// [`LS_COLUMNS`] enumeration plus a nearest-match suggestion when one clears
/// the similarity floor. Shared by every `[ls]` column-name validation site
/// (`[[ls.rules]]` entries and `[ls.columns]` / `[ls] columns-order`) so the
/// wording and the suggestion wiring live once
/// (`truth/tasks/adopt-nearest-match-suggestions-at-the-remaining-unknown-name-errors.typ`).
#[must_use]
pub(crate) fn unknown_column(name: &str) -> String {
    format!(
        "unknown column `{name}`, expected one of {}{}",
        LS_COLUMNS.join(", "),
        crate::suggest::suggestion(name, &LS_COLUMNS)
    )
}

#[cfg(test)]
#[path = "config_ls_columns_tests.rs"]
mod tests;
