//! Collapse `[[ls.columns.<name>.rules]]` into a concrete visibility.
//!
//! A column's conditional rules are resolved once per command, here, and
//! written back onto the [`Config`] the renderers read. That keeps the render
//! layer ([`crate::render::columns`]) infallible and unaware of conditions —
//! it asks a column for its `visibility` and gets an answer — and it puts the
//! evaluation where a failure can still be reported, next to the other
//! resolution `ls.rs` does before handing a config to the renderer (the
//! `columns-order` a view or `--columns` supplies is folded in the same way).
//!
//! First match wins, mirroring `[[ls.rules]]`, and a matching rule beats the
//! column's own `visibility` key — see [`mindtape::ColumnRule`] for why that
//! direction and not the other.

use anyhow::{Context, Result};
use mindtape::{Condition, ConditionEnv, Config};

/// Resolve every column's `rules` against `env`, writing the winning
/// visibility onto the column. Columns with no rules are untouched.
///
/// # Errors
/// Returns an error if a rule's condition fails to parse or evaluate. Config
/// load already validates every one of them, so this only recurs for a config
/// built in memory rather than parsed.
pub(crate) fn apply_column_rules(config: &mut Config, env: &ConditionEnv) -> Result<()> {
    for (name, column) in &mut config.ls.columns {
        for (index, rule) in column.rules.iter().enumerate() {
            let holds = match &rule.condition {
                None => true,
                Some(expr) => Condition::parse(expr)
                    .and_then(|condition| condition.matches(env))
                    .with_context(|| {
                        format!("ls.columns.{name}.rules[{index}].condition {expr:?}")
                    })?,
            };
            if holds {
                column.visibility = rule.visibility;
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "ls_columns_tests.rs"]
mod tests;
