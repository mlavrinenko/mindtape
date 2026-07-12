//! Env-driven override for "today", resolved once per process (see
//! `truth/tasks/pinnable-clock-for-reproducible-output.typ`).
//!
//! Precedence: `$MINDTAPE_NOW` (a `YYYY-MM-DD` date or an RFC 3339 timestamp —
//! the date part is used, split on `'T'`), then `$SOURCE_DATE_EPOCH` (epoch
//! seconds, the reproducible-builds standard), then the system clock (UTC).
//! [`init_pinned_today`] resolves this once from the real environment and
//! caches it in a process-wide [`OnceLock`] so every day-granular read in
//! [`crate::date`] agrees — call it once, early, from the binary's `main`.
//! A malformed override is a hard error naming the offending variable and
//! value; it never silently falls through to the system clock. An empty
//! value for either variable counts as unset, same as `$MINDTAPE_WORKTREE`.

use std::sync::OnceLock;

use thiserror::Error;

use crate::date::{days_from_civil, parse_ymd};

/// Why the env-driven "today" override could not be resolved.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DateOverrideError {
    /// `MINDTAPE_NOW` was set but is neither `YYYY-MM-DD` nor an RFC 3339
    /// timestamp with a `YYYY-MM-DD` date part.
    #[error("MINDTAPE_NOW={0:?} is not a valid YYYY-MM-DD date or RFC 3339 timestamp")]
    InvalidNow(String),
    /// `SOURCE_DATE_EPOCH` was set but is not a valid non-negative integer
    /// number of epoch seconds.
    #[error("SOURCE_DATE_EPOCH={0:?} is not a valid integer number of epoch seconds")]
    InvalidEpoch(String),
}

/// Process-global pinned "today", as days since the Unix epoch. `None` means
/// "no override" — the same behaviour as before [`init_pinned_today`] ever
/// runs (e.g. in library-only consumers, or unit tests that never call it),
/// so an uninitialized cache and an explicitly-resolved "no override" agree.
static PINNED_EPOCH_DAYS: OnceLock<Option<u64>> = OnceLock::new();

/// Resolve and cache the pinned "today" for this process from the real
/// environment (`$MINDTAPE_NOW`, `$SOURCE_DATE_EPOCH`). Call once, early, from
/// `main`, before any command reads a date; later calls are ignored — the
/// cache already won (env is fixed for the life of the process anyway).
///
/// # Errors
/// Returns [`DateOverrideError`] if either variable is set (non-empty) but
/// malformed.
pub fn init_pinned_today() -> Result<(), DateOverrideError> {
    let now_env = std::env::var("MINDTAPE_NOW").ok();
    let epoch_env = std::env::var("SOURCE_DATE_EPOCH").ok();
    let resolved = resolve(now_env.as_deref(), epoch_env.as_deref())?;
    let _ = PINNED_EPOCH_DAYS.set(resolved);
    Ok(())
}

/// The cached override, or `None` if unset or [`init_pinned_today`] never ran.
pub(crate) fn pinned_epoch_days() -> Option<u64> {
    PINNED_EPOCH_DAYS.get().copied().flatten()
}

/// Pure precedence resolution over the two variables' raw values: no env
/// reads, so this is testable without touching real process state (env
/// mutation in tests is not parallel-safe; this function sidesteps that
/// entirely). An empty string counts as unset for either variable.
///
/// `now_env` wins outright when set: `epoch_env` is not even inspected, valid
/// or not, matching `$MINDTAPE_WORKTREE`'s flag-wins-over-env precedence.
///
/// # Errors
/// Returns [`DateOverrideError`] naming the offending variable and value when
/// the higher-precedence variable that is actually set fails to parse.
pub(crate) fn resolve(
    now_env: Option<&str>,
    epoch_env: Option<&str>,
) -> Result<Option<u64>, DateOverrideError> {
    if let Some(value) = non_empty(now_env) {
        return parse_mindtape_now(value).map(Some);
    }
    if let Some(value) = non_empty(epoch_env) {
        return parse_source_date_epoch(value).map(Some);
    }
    Ok(None)
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.is_empty())
}

/// `YYYY-MM-DD`, or an RFC 3339 timestamp truncated at `'T'` to its date part
/// — the domain is date-only (day-granular), so a time component is accepted
/// and dropped rather than rejected.
fn parse_mindtape_now(value: &str) -> Result<u64, DateOverrideError> {
    let date_part = value.split('T').next().unwrap_or(value);
    let (year, month, day) =
        parse_ymd(date_part).ok_or_else(|| DateOverrideError::InvalidNow(value.to_owned()))?;
    Ok(clamp_to_epoch(days_from_civil(year, month, day)))
}

/// Epoch seconds (the reproducible-builds standard) to whole days since the
/// Unix epoch.
fn parse_source_date_epoch(value: &str) -> Result<u64, DateOverrideError> {
    value
        .parse::<u64>()
        .map(|secs| secs / 86_400)
        .map_err(|_| DateOverrideError::InvalidEpoch(value.to_owned()))
}

/// A pinned date before 1970-01-01 clamps to epoch 0 rather than wrapping or
/// panicking; nothing in this domain predates it.
fn clamp_to_epoch(days: i64) -> u64 {
    u64::try_from(days).unwrap_or(0)
}

#[cfg(test)]
#[path = "date_pin_tests.rs"]
mod tests;
