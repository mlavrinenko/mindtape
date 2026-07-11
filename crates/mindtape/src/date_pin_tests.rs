#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn both_unset_falls_through_to_none() {
    assert_eq!(resolve(None, None), Ok(None));
}

#[test]
fn empty_strings_count_as_unset() {
    assert_eq!(resolve(Some(""), Some("")), Ok(None));
}

#[test]
fn empty_mindtape_now_falls_back_to_source_date_epoch() {
    assert_eq!(resolve(Some(""), Some("0")), Ok(Some(0)));
}

#[test]
fn mindtape_now_plain_date_resolves_to_that_day() {
    // 2026-06-18 = day 20622 since the Unix epoch (shared with date.rs's
    // `known_date_2026_06_18`).
    assert_eq!(resolve(Some("2026-06-18"), None), Ok(Some(20_622)));
}

#[test]
fn mindtape_now_rfc3339_truncates_time_part() {
    assert_eq!(
        resolve(Some("2026-06-18T23:59:59Z"), None),
        Ok(Some(20_622))
    );
}

#[test]
fn mindtape_now_rfc3339_with_offset_truncates_time_part() {
    assert_eq!(
        resolve(Some("2026-06-18T08:00:00+02:00"), None),
        Ok(Some(20_622))
    );
}

#[test]
fn mindtape_now_malformed_is_a_hard_error_naming_variable_and_value() {
    let err = resolve(Some("not-a-date"), None).unwrap_err();
    assert_eq!(err, DateOverrideError::InvalidNow("not-a-date".to_owned()));
    let msg = err.to_string();
    assert!(msg.contains("MINDTAPE_NOW"), "{msg}");
    assert!(msg.contains("not-a-date"), "{msg}");
}

#[test]
fn mindtape_now_extra_date_segments_are_malformed() {
    assert!(resolve(Some("2026-06-18-01"), None).is_err());
}

#[test]
fn mindtape_now_wins_over_source_date_epoch() {
    assert_eq!(resolve(Some("2026-06-18"), Some("0")), Ok(Some(20_622)));
}

#[test]
fn mindtape_now_precedence_skips_validating_source_date_epoch() {
    // SOURCE_DATE_EPOCH is malformed but never inspected once MINDTAPE_NOW wins.
    assert_eq!(
        resolve(Some("2026-06-18"), Some("not-a-number")),
        Ok(Some(20_622))
    );
}

#[test]
fn source_date_epoch_used_when_mindtape_now_unset() {
    assert_eq!(
        resolve(None, Some(&(20_622_u64 * 86_400).to_string())),
        Ok(Some(20_622))
    );
}

#[test]
fn source_date_epoch_zero_is_the_unix_epoch() {
    assert_eq!(resolve(None, Some("0")), Ok(Some(0)));
}

#[test]
fn source_date_epoch_truncates_a_partial_day() {
    // A few seconds into day 20622 still floors to that day.
    assert_eq!(
        resolve(None, Some(&(20_622_u64 * 86_400 + 30).to_string())),
        Ok(Some(20_622))
    );
}

#[test]
fn source_date_epoch_malformed_is_a_hard_error_naming_variable_and_value() {
    let err = resolve(None, Some("not-a-number")).unwrap_err();
    assert_eq!(
        err,
        DateOverrideError::InvalidEpoch("not-a-number".to_owned())
    );
    let msg = err.to_string();
    assert!(msg.contains("SOURCE_DATE_EPOCH"), "{msg}");
    assert!(msg.contains("not-a-number"), "{msg}");
}

#[test]
fn source_date_epoch_negative_is_malformed() {
    assert!(resolve(None, Some("-1")).is_err());
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
