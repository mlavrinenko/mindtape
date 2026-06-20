//! Shared in-place edits for the `task.with(…)` header.
//!
//! Locating the call and upserting a named argument (`reviewed:`, `status:`, …)
//! is common to every command that stamps a field onto an existing task.

use anyhow::{Context, Result};
use typst_edit::{Call, Edit, find_method_calls};

/// The first `task.with(…)` call in `source`.
///
/// # Errors
/// Returns an error when the source holds no `task.with(…)` call.
pub fn task_with(source: &str) -> Result<Call> {
    find_method_calls(source, "task", "with")
        .into_iter()
        .next()
        .context("no task.with(…) call found in source")
}

/// Build an edit setting `name: value_text` in `call`: replace an existing
/// argument's value, or insert a new line before the closing paren.
///
/// # Errors
/// Returns an error when the call range is empty (malformed source).
pub fn upsert_named_arg(source: &str, call: &Call, name: &str, value_text: &str) -> Result<Edit> {
    if let Some(arg) = call.named(name) {
        return Ok(Edit::new(arg.value_range.clone(), value_text.to_owned()));
    }
    let indent = detect_indent(source, call);
    let insertion = format!("{indent}{name}: {value_text},\n");
    let close_paren = call
        .range
        .end
        .checked_sub(1)
        .context("call range is empty")?;
    Ok(Edit::new(close_paren..close_paren, insertion))
}

fn detect_indent(source: &str, call: &Call) -> String {
    call.args.first().map_or_else(
        || "  ".to_owned(),
        |arg| {
            let line_begin = source[..arg.value_range.start]
                .rfind('\n')
                .map_or(0, |pos| pos + 1);
            source[line_begin..]
                .chars()
                .take_while(|ch| *ch == ' ' || *ch == '\t')
                .collect()
        },
    )
}
