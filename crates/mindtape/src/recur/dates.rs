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
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecurError {
    /// `rrule(...)` expansion requires the crate to be built with the
    /// `rrule` cargo feature; this build was not (the config/feature gate in
    /// [`crate::recur::check_policy`] should have already rejected the task
    /// before this is ever reached).
    #[error("rrule(...) expansion requires mindtape to be built with the `rrule` cargo feature")]
    RruleNotImplemented,
    /// Chrono arithmetic ran out of range (a date past `NaiveDate::MAX`).
    #[error("date arithmetic overflowed")]
    DateOverflow,
    /// No weekday in the `on:` set was found within a full period — unreachable
    /// for a non-empty set, but the scan is bounded rather than infinite.
    #[error("no weekday in the `on:` set was reachable within a period")]
    NoMatchingWeekday,
    /// [`next_date`]'s due-anchor catch-up hit its step ceiling without
    /// reaching a date after today. Carries the `due` it started from. Means
    /// either a rule that does not move forward (`every(days: 0)`) or a `due`
    /// implausibly far in the past — the loop is bounded rather than infinite.
    #[error("recurrence did not reach today within {MAX_CATCH_UP_STEPS} steps from {0}")]
    CatchUpExhausted(String),
    /// The `rrule` crate could not produce a next occurrence: a malformed
    /// rule (should already be caught by `mt check`'s parse-time gate), a
    /// pathological rule that hits the crate's iteration bound, or the
    /// rule's own embedded `COUNT=`/`UNTIL=` ending the series. Carries the
    /// crate's own message; only constructed behind the `rrule` feature.
    #[error("{0}")]
    RruleExpansion(String),
}

/// The anchor `recur` resolves to: its own explicit anchor, or the project's
/// `[tasks.recur] default-anchor` when omitted.
///
/// `rrule(...)` is date-anchored only (RFC 5545 has no completion-relative
/// primitive — see `truth/tasks/rrule-escape-hatch.typ`): an omitted anchor
/// always resolves to `"due"` for it, even when the project's
/// `default-anchor` is `"completion"`. [`crate::recur::check_policy`] rejects
/// an *explicit* `anchor: "completion"` on an `rrule(...)` task outright, so
/// the only values reaching this function for one are `None` or `"due"`.
#[must_use]
pub fn effective_anchor<'a>(recur: &'a Recur, config: &'a RecurConfig) -> &'a str {
    if matches!(recur.rule, Rule::Rrule { .. }) {
        return "due";
    }
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

/// Ceiling on [`next_date`]'s due-anchor catch-up. Bounds the loop against a
/// rule that never advances; `10_000` steps is a daily task some 27 years stale,
/// far past any plausible backlog.
const MAX_CATCH_UP_STEPS: usize = 10_000;

/// Compute the single next occurrence date.
///
/// `anchor: "due"` catches up: advance by the rule from `due` (falling back
/// to `today` when the task has none) until strictly after `today`, so a late
/// completion never yields an already-overdue occurrence (Org `++`). Any
/// other anchor (`"completion"`, the default) advances once from `today` —
/// always in the future, so it needs no catch-up loop.
///
/// The catch-up is bounded by [`MAX_CATCH_UP_STEPS`]: a rule that stands still
/// (`every(days: 0)`) is [`RecurError::CatchUpExhausted`], not a hang.
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
        let mut candidate = due.unwrap_or(today);
        for _ in 0..MAX_CATCH_UP_STEPS {
            candidate = advance_once(&recur.rule, candidate)?;
            if candidate > today {
                return Ok(candidate);
            }
        }
        Err(RecurError::CatchUpExhausted(format_iso(
            due.unwrap_or(today),
        )))
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
        Rule::Period {
            value,
            weekdays,
            start,
            ..
        } if !weekdays.is_empty() => next_weekday_after(from, weekdays, *value, *start),
        Rule::Period { unit, value, .. } => add_period(from, *unit, *value),
        Rule::Rrule { raw } => advance_rrule(raw, from),
    }
}

/// The `rrule(...)` step: the rule's first occurrence strictly after `from`.
/// `crate::recur::rrule` is the real engine with the `rrule` feature on and a
/// fixed-error stub without it (`recur/rrule_off.rs`), so this stays one
/// always-compiled function either way.
fn advance_rrule(raw: &str, from: NaiveDate) -> Result<NaiveDate, RecurError> {
    crate::recur::rrule::next_after(raw, from)
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

/// The Monday that fixes the weeks of every `every(weeks: N, on: …)` series:
/// a week is an on-week when its Monday is a multiple of `N` weeks after this
/// date.
const SERIES_EPOCH: NaiveDate = match NaiveDate::from_ymd_opt(2001, 1, 1) {
    Some(date) => date,
    None => NaiveDate::MIN,
};

/// Whether the Monday-to-Sunday week holding `date` is an on-week for a
/// `weeks`-week period, counted from the Monday of `start`'s week, else from
/// [`SERIES_EPOCH`].
fn is_on_week(date: NaiveDate, weeks: u32, start: Option<NaiveDate>) -> bool {
    let monday = date.week(Weekday::Mon).first_day();
    let origin = start.map_or(SERIES_EPOCH, |start| start.week(Weekday::Mon).first_day());
    let elapsed = monday.signed_duration_since(origin).num_weeks();
    elapsed.rem_euclid(i64::from(weeks.max(1))) == 0
}

/// The next date strictly after `from` whose weekday is in `weekdays` and
/// whose Monday-to-Sunday week is an on-week: every `weeks`-th week counted
/// from the series origin (see [`is_on_week`]), whichever week `from` itself
/// falls in. Bounded to
/// `weeks` weeks of scan: any non-empty set matches within one period.
fn next_weekday_after(
    from: NaiveDate,
    weekdays: &[Weekday],
    weeks: u32,
    start: Option<NaiveDate>,
) -> Result<NaiveDate, RecurError> {
    let mut candidate = from;
    for _ in 0..u64::from(weeks.max(1)) * 7 {
        candidate = candidate.succ_opt().ok_or(RecurError::DateOverflow)?;
        if weekdays.contains(&candidate.weekday()) && is_on_week(candidate, weeks, start) {
            return Ok(candidate);
        }
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
/// [`mindtape::model::naming::MintContext`] and `on(y, m, d)` Typst edits take.
#[must_use]
pub fn date_to_ymd(date: NaiveDate) -> (i32, u8, u8) {
    // `month()` is 1..=12 and `day()` 1..=31 (chrono invariants for any valid
    // `NaiveDate`); both fit `u8` exactly.
    #[allow(clippy::cast_possible_truncation)]
    let ymd = (date.year(), date.month() as u8, date.day() as u8);
    ymd
}

#[cfg(test)]
#[path = "dates_tests.rs"]
mod tests;
