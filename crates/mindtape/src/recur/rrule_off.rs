//! Stand-in for [`crate::recur::rrule`] when the `rrule` cargo feature is off:
//! the same surface, no engine.
//!
//! `[tasks.recur] rrule` plus the feature gate in [`crate::recur::check_policy`]
//! reject an `rrule(...)` task long before an expansion is attempted, so these
//! are the "never reached in a well-formed project" arms. They live in their
//! own file rather than as `#[cfg(not(...))]` twins beside the real functions
//! so that every function in `recur/mod.rs` and `recur/dates.rs` is compiled under
//! any feature set — a mutation sweep can only exercise code the build
//! configuration actually kept (see `.cargo/mutants.toml`).

use chrono::NaiveDate;

use crate::recur::dates::RecurError;

/// Always `Ok`: with no engine there is no grammar to check the rule against.
/// The rule is rejected by the feature gate instead.
// The `Result` is not superfluous: it mirrors the real `recur::rrule` so the
// caller stays one function under either feature setting.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn validate_rule(_raw: &str) -> Result<(), String> {
    Ok(())
}

/// Always [`RecurError::RruleNotImplemented`].
pub(crate) fn next_after(_raw: &str, _from: NaiveDate) -> Result<NaiveDate, RecurError> {
    Err(RecurError::RruleNotImplemented)
}
