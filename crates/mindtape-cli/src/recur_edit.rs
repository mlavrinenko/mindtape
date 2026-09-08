//! Typst-source rewriting helpers for `mt recur`.
//!
//! The status/due/start edits shared by spawn and mutate, the `links:`
//! `recurred-from` rewrite (spawn only), and the nested `every()`/`rrule()`
//! `count:` decrement. Same `task_edit::{task_with, upsert_named_arg}` +
//! `typst_edit::apply` shape as `flip.rs`'s `status_edit` and `review.rs`'s
//! `build_edit` — just more fields rewritten at once.

use std::ops::Range;

use anyhow::{Context, Result, anyhow};
use chrono::{Datelike, NaiveDate};
use mindtape::Config;
use typst_edit::{Call, Edit, apply, find_calls};

use crate::task_edit::{initial_status, task_with, upsert_named_arg};

/// Render a Typst `on(y, m, d)` call for `date` — the same literal shape
/// `review.rs`'s `build_edit` writes for `reviewed:`.
#[must_use]
pub fn on_call(date: NaiveDate) -> String {
    format!("on({}, {}, {})", date.year(), date.month(), date.day())
}

/// The date inputs one occurrence-advance needs: today (for the status
/// stamp), the source task's current `due`/`start`, and the freshly computed
/// `next_due`. Bundled so [`status_due_start_edits`] takes one argument
/// instead of four.
#[derive(Debug, Clone, Copy)]
pub struct OccurrenceDates {
    /// Today's date — the status stamp's date.
    pub today: (i32, u8, u8),
    /// The source task's current `due:`, if it set one.
    pub old_due: Option<NaiveDate>,
    /// The source task's current `start:`, if it set one.
    pub old_start: Option<NaiveDate>,
    /// The freshly computed next occurrence date.
    pub next_due: NaiveDate,
}

/// The shifted `start` for one occurrence advance: moves by the same delta as
/// `due` (a "N days before due" offset stays constant across occurrences).
/// `None` when the source task did not set both `due` and `start` — nothing
/// to shift.
#[must_use]
pub fn shift_start(
    old_due: Option<NaiveDate>,
    old_start: Option<NaiveDate>,
    next_due: NaiveDate,
) -> Option<NaiveDate> {
    let (due, start) = (old_due?, old_start?);
    let delta = next_due.signed_duration_since(due);
    start.checked_add_signed(delta)
}

/// The `due:`/`start:` edits for one occurrence advance: `due` set to
/// `next_due`; `start` set to `next_start` when the caller computed a shift
/// (see [`shift_start`]).
pub fn due_start_edits(
    source: &str,
    call: &Call,
    next_due: NaiveDate,
    next_start: Option<NaiveDate>,
) -> Result<Vec<Edit>> {
    let mut edits = vec![upsert_named_arg(source, call, "due", &on_call(next_due))?];
    if let Some(start) = next_start {
        edits.push(upsert_named_arg(source, call, "start", &on_call(start))?);
    }
    Ok(edits)
}

/// The full set of edits a fresh occurrence needs: status reset to the
/// initial legend value dated `today`, plus [`due_start_edits`]. Shared by
/// spawn (on the copied source) and mutate (in place); `mt recur --skip`
/// calls [`due_start_edits`] directly since it must not touch status.
///
/// # Errors
/// Returns an error when `source` holds no `task.with(…)` call, no status is
/// configured to stamp, or an edit cannot be built.
pub fn status_due_start_edits(
    source: &str,
    config: &Config,
    dates: &OccurrenceDates,
) -> Result<Vec<Edit>> {
    let call = task_with(source)?;
    let status = initial_status(config).context(
        "no initial status configured; cannot stamp a fresh occurrence ([tasks.status] is empty)",
    )?;
    let mut edits = vec![upsert_named_arg(
        source,
        &call,
        "status",
        &mindtape::status_value(&status, dates.today, None),
    )?];
    let next_start = shift_start(dates.old_due, dates.old_start, dates.next_due);
    edits.extend(due_start_edits(source, &call, dates.next_due, next_start)?);
    Ok(edits)
}

/// Point `links:` at `predecessor_stem` with a single
/// `recurred-from("<stem>.typ")` edge, creating the argument when absent and
/// preserving every edge of any other kind.
///
/// It REPLACES rather than appends, and the difference is the whole point. A
/// spawn's source is its predecessor's source, so an occurrence spawned from an
/// occurrence already carries a `recurred-from` — and that kind allows one edge
/// per task ([`mindtape::link::builtin_kinds`]). Appending wrote a second one,
/// which is a file `mt check` refuses: the chain is a linked list, and each
/// occurrence names only the one before it.
///
/// # Errors
/// Returns an error when `source` holds no `task.with(…)` call.
pub fn set_recurred_from(source: &str, predecessor_stem: &str) -> Result<Edit> {
    let call = task_with(source)?;
    let edge = format!("recurred-from(\"{predecessor_stem}.typ\")");
    let value_text = match call.named("links") {
        Some(existing) => without_recurred_from(source, &existing.value_range, &edge)
            .unwrap_or_else(|| format!("{} + {edge}", existing.value)),
        None => edge,
    };
    upsert_named_arg(source, &call, "links", &value_text)
}

/// The `links:` value in `range` rewritten as every edge call that is not
/// `recurred-from`, then `edge` — each surviving call's text taken verbatim, so
/// a note's markup passes through untouched rather than being re-rendered.
///
/// `None` when there is nothing to replace (no `recurred-from` in there, so the
/// plain append is already right and byte-identical), or when the edge calls do
/// not account for the whole value apart from `+` and whitespace. The second
/// case is the guard: this scanner only knows the edge helpers, and silently
/// dropping something it cannot see would be a worse bug than the duplicate
/// edge it is here to prevent.
fn without_recurred_from(source: &str, range: &Range<usize>, edge: &str) -> Option<String> {
    let calls: Vec<_> = mindtape::link_edges(source)
        .into_iter()
        .filter(|call| call.range.start >= range.start && call.range.end <= range.end)
        .collect();
    if !calls.iter().any(|call| call.name == "recurred-from") {
        return None;
    }
    let mut gaps = String::new();
    let mut cursor = range.start;
    for call in &calls {
        gaps.push_str(source.get(cursor..call.range.start)?);
        cursor = call.range.end;
    }
    gaps.push_str(source.get(cursor..range.end)?);
    if !gaps.chars().all(|ch| ch.is_whitespace() || ch == '+') {
        return None;
    }

    let mut kept: Vec<&str> = calls
        .iter()
        .filter(|call| call.name != "recurred-from")
        .map(|call| source.get(call.range.clone()))
        .collect::<Option<Vec<_>>>()?;
    kept.push(edge);
    Some(kept.join(" + "))
}

/// Decrement the recur rule's `count:` in place, when the task set one
/// (`None` stays unbounded — nothing to rewrite).
///
/// # Errors
/// Returns an error when `old_count` is set but no `every(...)`/`rrule(...)`
/// call can be located in `source`.
pub fn decrement_count_edit(source: &str, old_count: Option<u32>) -> Result<Option<Edit>> {
    let Some(old_count) = old_count else {
        return Ok(None);
    };
    let call = recur_call(source)?;
    let new_count = old_count.saturating_sub(1);
    Ok(Some(upsert_named_arg(
        source,
        &call,
        "count",
        &new_count.to_string(),
    )?))
}

/// Locate the task's `every(...)`/`rrule(...)` call (the `recur:` value).
fn recur_call(source: &str) -> Result<Call> {
    find_calls(source, "every")
        .into_iter()
        .chain(find_calls(source, "rrule"))
        .next()
        .context("could not locate the every()/rrule() call to update its `count:`")
}

/// Apply `edits` to `source` in one atomic splice.
///
/// # Errors
/// Returns an error when the edits overlap or fall out of bounds.
pub fn apply_edits(source: &str, edits: Vec<Edit>) -> Result<String> {
    apply(source, edits).map_err(|err| anyhow!(err.to_string()))
}

#[cfg(test)]
#[path = "recur_edit_tests.rs"]
mod tests;
