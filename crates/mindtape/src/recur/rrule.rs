//! The `rrule(...)` escape hatch's expansion engine.
//!
//! Wraps the `rrule` crate (RFC 5545 recurrence rules): [`validate_rule`]
//! grammar-checks a raw rule string at `mt check`/`mt recur` parse time (see
//! [`crate::recur::check_policy`]), and [`next_after`] expands it for
//! [`crate::recur::dates::advance_once`]. `DTSTART` is always the reference date
//! the caller passes in (the task's `due`, or a catch-up cursor mid-loop). See
//! `truth/tasks/2026-07/rrule-escape-hatch.typ`'s "Seed `DTSTART` from the
//! task's `due`" decision. This module never reads the clock.

use chrono::{DateTime, Duration, NaiveDate, TimeZone};
use rrule::{RRule, RRuleSet, Tz, Unvalidated};

use crate::recur::dates::RecurError;

/// Grammar/semantic-check `raw` as an RFC 5545 `RRULE` value, independent of
/// any task's `due` (`DTSTART` is an arbitrary fixed date, and validity never
/// depends on which date seeds it).
///
/// # Errors
/// A message that quotes the offending rule, such as
/// `invalid rrule "FREQ=NOPE": RRule parsing error: ...`.
pub(crate) fn validate_rule(raw: &str) -> Result<(), String> {
    // Any fixed date works: BYDAY/BYMONTHDAY/BYSETPOS grammar validity does
    // not depend on it, only the *expansion* (see `next_after`) does.
    let epoch = NaiveDate::from_ymd_opt(2000, 1, 1).expect("2000-01-01 is a valid date");
    build_set(raw, midnight_utc(epoch))
        .map(|_| ())
        .map_err(|err| format!("invalid rrule {raw:?}: {err}"))
}

/// The rule's first occurrence strictly after `due` (`RRuleSet::after`-style:
/// `DTSTART` comes from `due`, and an occurrence landing exactly on `due`
/// itself does not count, because the caller always wants forward progress).
///
/// The `rrule` crate's own iteration cap bounds it (`RRuleSet::all`'s
/// `limited` mode). A pathological rule that never matches surfaces a clear
/// error here rather than hanging.
///
/// # Errors
/// See [`RecurError::RruleExpansion`].
pub(crate) fn next_after(raw: &str, due: NaiveDate) -> Result<NaiveDate, RecurError> {
    let dt_start = midnight_utc(due);
    let set = build_set(raw, dt_start)
        .map_err(|err| RecurError::RruleExpansion(format!("invalid rrule {raw:?}: {err}")))?;
    // Strictly after `due`: nudge the lower bound one second past DTSTART so
    // an occurrence landing exactly on it (the common case — DTSTART is
    // itself always a candidate unless BY* filters exclude it) is skipped.
    let after = dt_start + Duration::seconds(1);
    let result = set.after(after).all(1);
    if let Some(next) = result.dates.first() {
        return Ok(next.date_naive());
    }
    let reason = if result.limited {
        "hit the rrule crate's iteration bound before finding one (a pathological rule?)"
    } else {
        "the rule's own COUNT=/UNTIL= already ended the series"
    };
    Err(RecurError::RruleExpansion(format!(
        "rrule {raw:?} produced no occurrence after {due}: {reason}"
    )))
}

fn build_set(raw: &str, dt_start: DateTime<Tz>) -> Result<RRuleSet, rrule::RRuleError> {
    raw.parse::<RRule<Unvalidated>>()?.build(dt_start)
}

/// `date` at midnight UTC. The timezone is arbitrary, because every date this
/// module produces or consumes is calendar-only (see [`crate::recur::dates`]'s
/// own no-clock-reads discipline). UTC just avoids daylight-saving ambiguity in
/// the `rrule` crate's `chrono-tz`-backed clock arithmetic.
fn midnight_utc(date: NaiveDate) -> DateTime<Tz> {
    Tz::UTC.from_utc_datetime(&date.and_hms_opt(0, 0, 0).expect("midnight is always valid"))
}

#[cfg(test)]
#[path = "rrule_tests.rs"]
mod tests;
