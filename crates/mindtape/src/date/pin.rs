//! Env-driven override for "today", resolved once per process (see
//! `truth/tasks/2026-07/pinnable-clock-for-reproducible-output.typ`).
//!
//! `$MINDTAPE_NOW` pins "today": a `YYYY-MM-DD` date or an RFC 3339 timestamp.
//! The code takes the date part verbatim, split on `'T'`. A trailing offset
//! shifts nothing, since the pinned value *is* the calendar date to stamp. An
//! unset variable falls through to the system clock, read in the machine's
//! local timezone (`crate::date`). [`init_pinned_today`] resolves this once
//! from the real environment and caches it in a process-wide [`OnceLock`], so
//! every date read in [`crate::date`] agrees. Call it once, early, from the
//! binary's `main`. A malformed override is a hard error that reports the
//! variable and the value. It never silently falls through to the system
//! clock. An empty value counts as unset, same as `$MINDTAPE_WORKTREE`.
//!
//! `$SOURCE_DATE_EPOCH` is deliberately not honoured: `mt`'s "today" is an
//! operational query concept (what is ready / stale / overdue), not a
//! build-artifact timestamp. Nix dev/CI shells export
//! `SOURCE_DATE_EPOCH=315532800` (1980-01-01), and honouring it would silently
//! time-travel every `mt ls` there. Reproducible output pins `$MINDTAPE_NOW`.

use std::sync::OnceLock;

use thiserror::Error;

use crate::date::{days_from_civil, parse_ymd};

/// Why the code could not resolve the env-driven "today" override.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DateOverrideError {
    /// `MINDTAPE_NOW` holds a value that is neither `YYYY-MM-DD` nor an RFC
    /// 3339 timestamp with a `YYYY-MM-DD` date part.
    #[error("MINDTAPE_NOW={0:?} is not a valid YYYY-MM-DD date or RFC 3339 timestamp")]
    InvalidNow(String),
}

/// Process-global pinned "today", as days since the Unix epoch. `None` means
/// "no override", the same behaviour as before [`init_pinned_today`] ever
/// runs (in library-only consumers, or unit tests that never call it). An
/// uninitialized cache and an explicitly-resolved "no override" agree.
static PINNED_EPOCH_DAYS: OnceLock<Option<u64>> = OnceLock::new();

/// Resolve and cache the pinned "today" for this process from `$MINDTAPE_NOW`.
/// Call once, early, from `main`, before any command reads a date. Later calls
/// do nothing, because the cache already won and the env stays fixed for the
/// process anyway.
///
/// # Errors
/// Returns [`DateOverrideError`] if `MINDTAPE_NOW` holds a non-empty value that
/// does not parse.
pub fn init_pinned_today() -> Result<(), DateOverrideError> {
    let now_env = std::env::var("MINDTAPE_NOW").ok();
    let resolved = resolve(now_env.as_deref())?;
    let _ = PINNED_EPOCH_DAYS.set(resolved);
    Ok(())
}

/// The cached override, or `None` if unset or [`init_pinned_today`] never ran.
pub(crate) fn pinned_epoch_days() -> Option<u64> {
    PINNED_EPOCH_DAYS.get().copied().flatten()
}

/// Pure resolution of `$MINDTAPE_NOW`'s raw value. It never reads the env,
/// which makes it testable without touching real process state (env mutation
/// in tests is not parallel-safe, and this sidesteps it entirely). An empty
/// string counts as unset.
///
/// # Errors
/// Returns [`DateOverrideError::InvalidNow`] when set (non-empty) but
/// unparseable.
pub(crate) fn resolve(now_env: Option<&str>) -> Result<Option<u64>, DateOverrideError> {
    match non_empty(now_env) {
        Some(value) => parse_mindtape_now(value).map(Some),
        None => Ok(None),
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.is_empty())
}

/// `YYYY-MM-DD`, or an RFC 3339 timestamp truncated at `'T'` to its date part.
/// The domain is date-only. The code accepts a time component and drops it
/// rather than rejecting it.
fn parse_mindtape_now(value: &str) -> Result<u64, DateOverrideError> {
    let date_part = value.split('T').next().unwrap_or(value);
    let (year, month, day) =
        parse_ymd(date_part).ok_or_else(|| DateOverrideError::InvalidNow(value.to_owned()))?;
    Ok(clamp_to_epoch(days_from_civil(year, month, day)))
}

/// A pinned date before 1970-01-01 clamps to epoch 0 rather than wrapping or
/// panicking. Every date in this domain comes after it.
fn clamp_to_epoch(days: i64) -> u64 {
    u64::try_from(days).unwrap_or(0)
}

#[cfg(test)]
#[path = "pin_tests.rs"]
mod tests;
