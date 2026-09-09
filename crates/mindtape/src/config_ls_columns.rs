//! `[ls.columns.<name>]` — one `mt ls` column's display settings, and the
//! validation of the column-name vocabulary they are keyed by.
//!
//! Split out of [`crate::config_ls`] so that file stays under the repo's Rust
//! line cap; the `[ls]` block itself, the [`crate::config_ls::LS_COLUMNS`]
//! vocabulary and the `[[ls.rules]]` listing cascade all remain there.
//!
//! # Why a column carries one `when`
//!
//! Whether a column renders is one question, so it takes one answer: a boolean
//! expression in the condition language, evaluated against the rows it would
//! render.
//!
//! ```toml
//! [ls.columns.file]
//! when = 'named() || (!tty() && format() == "table")'
//! ```
//!
//! The predicates are what make that expressible. `nonempty()`, `contested()`,
//! `named()` and `blocked()` describe the listing, so the built-in policy each
//! column used to get from a Rust match is now a line of the embedded floor:
//! `#` and `title` are `true`, `status` / `priority` / `due` are `nonempty()`,
//! `assignee` is `contested()`, `ready` is `blocked()`, and `file` is the
//! expression above.
//!
//! A scalar replaces wholesale across config layers, so a project overrides an
//! inherited `when` by writing its own on one line — `when = 'false'` turns a
//! floored column off — and there is no precedence to state, no ordering, and
//! no opt-out convention: `||` does what a rule list's first-match-wins did.
//!
//! An absent `when` renders the column never. Nothing falls back to Rust: the
//! floor carries one per column, so the floor is the complete statement of what
//! `mt ls` shows, and a column nobody described is a column nobody asked for.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::condition::Condition;
use crate::config_ls::{LS_COLUMNS, LsConfig};

/// `[ls.columns.<name>]` — one column's display settings.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ColumnConfig {
    /// Whether this column renders, as a condition expression — the same
    /// grammar `[[ls.rules]]` uses (`tty()`, `format()`, `env()`,
    /// `on_branch()`, `now()`, `weekday()`, `true` / `false`), plus the four
    /// listing predicates only a column resolution can answer: `nonempty()`,
    /// `contested()`, `named()`, `blocked()`.
    ///
    /// Absent means the column never renders. The embedded floor carries one
    /// per column, so an absent `when` is a column the project deleted from
    /// the floor rather than a column awaiting a built-in default.
    pub when: Option<String>,
}

/// Validate `[ls.columns]` keys, their `when` expressions, and `[ls]`
/// `columns-order` against [`LS_COLUMNS`]: unknown column names, duplicates
/// within `columns-order`, and unparseable `when` expressions are reported.
///
/// A `when` is parsed here so a typo (`ttty()`, a task field name carried over
/// from filter syntax) fails at config load naming the column, rather than
/// silently never matching at render time — where a column that does not
/// appear looks exactly like a column that was not meant to.
#[must_use]
pub fn validate_ls_columns(ls: &LsConfig) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, column) in &ls.columns {
        if LS_COLUMNS.contains(&name.as_str()) {
            validate_column_when(name, column, &mut problems);
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

/// Parse one column's `when`, reporting a failure under the column that owns
/// it.
///
/// A blank expression is rejected rather than parsed: [`Condition`] reads an
/// empty expression as the always-matching one, so `when = ''` would render the
/// column always while an absent `when` renders it never — two spellings of
/// "nothing written here" with opposite answers. Writing the answer out
/// (`true` / `false`) is what the key is for.
fn validate_column_when(name: &str, column: &ColumnConfig, problems: &mut Vec<String>) {
    let Some(expr) = &column.when else {
        return;
    };
    if expr.trim().is_empty() {
        problems.push(format!(
            "ls.columns.{name}.when: empty expression; write `true` or `false`"
        ));
    } else if let Err(err) = Condition::parse(expr) {
        problems.push(format!("ls.columns.{name}.when: {err}"));
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
