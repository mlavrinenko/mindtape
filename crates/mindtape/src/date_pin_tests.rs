#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn unset_falls_through_to_none() {
    assert_eq!(resolve(None), Ok(None));
}

#[test]
fn empty_string_counts_as_unset() {
    assert_eq!(resolve(Some("")), Ok(None));
}

#[test]
fn mindtape_now_plain_date_resolves_to_that_day() {
    // 2026-06-18 = day 20622 since the Unix epoch (shared with date.rs's
    // `known_date_2026_06_18`).
    assert_eq!(resolve(Some("2026-06-18")), Ok(Some(20_622)));
}

#[test]
fn mindtape_now_rfc3339_truncates_time_part() {
    assert_eq!(resolve(Some("2026-06-18T23:59:59Z")), Ok(Some(20_622)));
}

#[test]
fn mindtape_now_rfc3339_with_offset_truncates_time_part() {
    assert_eq!(resolve(Some("2026-06-18T08:00:00+02:00")), Ok(Some(20_622)));
}

#[test]
fn mindtape_now_malformed_is_a_hard_error_naming_variable_and_value() {
    let err = resolve(Some("not-a-date")).unwrap_err();
    assert_eq!(err, DateOverrideError::InvalidNow("not-a-date".to_owned()));
    let msg = err.to_string();
    assert!(msg.contains("MINDTAPE_NOW"), "{msg}");
    assert!(msg.contains("not-a-date"), "{msg}");
}

#[test]
fn mindtape_now_extra_date_segments_are_malformed() {
    assert!(resolve(Some("2026-06-18-01")).is_err());
}

#[test]
fn pinned_epoch_days_is_none_before_init_runs() {
    // `init_pinned_today` is process-global and single-set (real tests exercise
    // it out-of-process, over the compiled `mt` binary — see
    // crates/mindtape-cli/tests/date_pin_e2e.rs — precisely to avoid a shared
    // OnceLock racing across this binary's test threads). Absent that call,
    // the cache reads as "no override", matching the pre-pin behaviour.
    assert_eq!(pinned_epoch_days(), None);
}
