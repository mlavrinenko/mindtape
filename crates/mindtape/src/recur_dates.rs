//! Pure next-occurrence date arithmetic for [`crate::recur::Recur`].
//!
//! No clock reads: every function takes its reference date as an argument —
//! `mt recur` (`crates/mindtape-cli`) passes today's date and the task's
//! current `due`. `chrono` gives calendar-correct period arithmetic
//! (month/year-end clamping via [`Months`]); the weekday-set path is a plain
//! forward scan. See `truth/tasks/recurring-tasks.typ`'s "Anchor" section.

use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use thiserror::Error;

use crate::recur::{PeriodUnit, Recur, RecurConfig, Rule};

/// Why next-date computation could not produce a date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RecurError {
    /// `rrule(...)` expansion is a later wave; this wave parses and gates the
    /// rule but cannot compute its next date.
    #[error("rrule(...) expansion is not implemented yet (planned for a later wave)")]
    RruleNotImplemented,
    /// Chrono arithmetic ran out of range (a date past `NaiveDate::MAX`).
    #[error("date arithmetic overflowed")]
    DateOverflow,
    /// No weekday in the `on:` set was found within a full week — unreachable
    /// for a non-empty set, but the scan is bounded rather than infinite.
    #[error("no weekday in the `on:` set was reachable within a week")]
    NoMatchingWeekday,
}

/// The anchor `recur` resolves to: its own explicit anchor, or the project's
/// `[tasks.recur] default-anchor` when omitted.
#[must_use]
pub fn effective_anchor<'a>(recur: &'a Recur, config: &'a RecurConfig) -> &'a str {
    recur
        .anchor
        .as_deref()
        .unwrap_or(config.default_anchor.as_str())
}

/// The materialize model `recur` resolves to: its own explicit value, or the
/// project's `[tasks.recur] materialize` when omitted.
#[must_use]
pub fn effective_materialize<'a>(recur: &'a Recur, config: &'a RecurConfig) -> &'a str {
    recur
        .materialize
        .as_deref()
        .unwrap_or(config.materialize.as_str())
}

/// Compute the single next occurrence date.
///
/// `anchor: "due"` catches up: advance by the rule from `due` (falling back
/// to `today` when the task has none) until strictly after `today`, so a late
/// completion never yields an already-overdue occurrence (Org `++`). Any
/// other anchor (`"completion"`, the default) advances once from `today` —
/// always in the future, so it needs no catch-up loop.
///
/// # Errors
/// See [`RecurError`].
pub fn next_date(
    recur: &Recur,
    config: &RecurConfig,
    today: NaiveDate,
    due: Option<NaiveDate>,
) -> Result<NaiveDate, RecurError> {
    if effective_anchor(recur, config) == "due" {
        let start = due.unwrap_or(today);
        let mut candidate = advance_once(&recur.rule, start)?;
        while candidate <= today {
            candidate = advance_once(&recur.rule, candidate)?;
        }
        Ok(candidate)
    } else {
        advance_once(&recur.rule, today)
    }
}

/// Advance `from` by exactly one occurrence of `rule`. The shared single step
/// both [`next_date`]'s due-anchor catch-up loop and `mt recur --advance N`'s
/// forward chain compose from.
///
/// # Errors
/// See [`RecurError`].
pub fn advance_once(rule: &Rule, from: NaiveDate) -> Result<NaiveDate, RecurError> {
    match rule {
        Rule::Period { weekdays, .. } if !weekdays.is_empty() => next_weekday_after(from, weekdays),
        Rule::Period { unit, value, .. } => add_period(from, *unit, *value),
        Rule::Rrule { .. } => Err(RecurError::RruleNotImplemented),
    }
}

/// Add one `unit`-`value` period to `from`, clamping month/year-end overflow
/// (Jan 31 + 1 month = Feb 28/29) via chrono's [`Months`] arithmetic.
fn add_period(from: NaiveDate, unit: PeriodUnit, value: u32) -> Result<NaiveDate, RecurError> {
    match unit {
        PeriodUnit::Days => from.checked_add_days(Days::new(u64::from(value))),
        PeriodUnit::Weeks => from.checked_add_days(Days::new(u64::from(value) * 7)),
        PeriodUnit::Months => from.checked_add_months(Months::new(value)),
        PeriodUnit::Years => value
            .checked_mul(12)
            .and_then(|months| from.checked_add_months(Months::new(months))),
    }
    .ok_or(RecurError::DateOverflow)
}

/// The next date strictly after `from` whose weekday is in `weekdays`.
/// Bounded to a 7-day scan: any non-empty set matches within one week.
fn next_weekday_after(from: NaiveDate, weekdays: &[Weekday]) -> Result<NaiveDate, RecurError> {
    let mut candidate = from.succ_opt().ok_or(RecurError::DateOverflow)?;
    for _ in 0..7 {
        if weekdays.contains(&candidate.weekday()) {
            return Ok(candidate);
        }
        candidate = candidate.succ_opt().ok_or(RecurError::DateOverflow)?;
    }
    Err(RecurError::NoMatchingWeekday)
}

/// Whether `next` overshoots the series' `until` bound — series end, not to
/// be confused with an instance's own `due` (see the spec's "Stop
/// conditions"). `until` unset or unparsable means unbounded.
#[must_use]
pub fn past_until(recur: &Recur, next: NaiveDate) -> bool {
    recur
        .until
        .as_deref()
        .and_then(parse_iso)
        .is_some_and(|until| next > until)
}

/// Whether the series has no budget left (`count` reached zero).
#[must_use]
pub fn exhausted(recur: &Recur) -> bool {
    recur.count == Some(0)
}

/// The carried-forward `count` after materializing one occurrence:
/// decremented when set; `None` (unbounded) stays `None`.
#[must_use]
pub fn carry_count(recur: &Recur) -> Option<u32> {
    recur.count.map(|count| count.saturating_sub(1))
}

/// Parse a `YYYY-MM-DD` string into a [`NaiveDate`]; `None` on any malformed
/// input.
#[must_use]
pub fn parse_iso(text: &str) -> Option<NaiveDate> {
    let (year, month, day) = crate::date::parse_ymd(text)?;
    NaiveDate::from_ymd_opt(year, u32::from(month), u32::from(day))
}

/// Render a [`NaiveDate`] as `YYYY-MM-DD`.
#[must_use]
pub fn format_iso(date: NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

/// A `(year, month, day)` tuple (as returned by `mindtape::today_ymd`) as a
/// [`NaiveDate`]; `None` for an out-of-range tuple.
#[must_use]
pub fn ymd_to_date(ymd: (i32, u8, u8)) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(ymd.0, u32::from(ymd.1), u32::from(ymd.2))
}

/// A [`NaiveDate`] as a `(year, month, day)` tuple — the shape
/// [`mindtape::naming::MintContext`] and `on(y, m, d)` Typst edits take.
#[must_use]
pub fn date_to_ymd(date: NaiveDate) -> (i32, u8, u8) {
    // `month()` is 1..=12 and `day()` 1..=31 (chrono invariants for any valid
    // `NaiveDate`); both fit `u8` exactly.
    #[allow(clippy::cast_possible_truncation)]
    let ymd = (date.year(), date.month() as u8, date.day() as u8);
    ymd
}

#[cfg(test)]
#[path = "recur_dates_tests.rs"]
mod tests;
