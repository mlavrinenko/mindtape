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
    // 2026-06-18 = day 20622 since the Unix epoch (shared with date/mod.rs's
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

// --- The process-global cache --------------------------------------------
//
// `init_pinned_today` sets a `OnceLock` once per process, so neither its
// caching nor `pinned_epoch_days`'s read of the cache can be asserted from a
// test that shares this binary's process with the rest of the suite. The two
// tests below split the two halves: the first asserts the uninitialised
// reading in the parent process (which never calls `init_pinned_today`), the
// second re-execs the binary for its own name so the child owns a fresh lock.
//
// `crates/mindtape-cli/tests/it/date_pin_e2e.rs` covers the same ground over the
// real `mt` binary, but it cannot stand in for this one: cargo-mutants tests
// only the package holding the mutant (no `--test-workspace`), so a mutant in
// `date/pin.rs` is never run against the CLI crate's e2e suite.

#[test]
fn pinned_epoch_days_is_none_before_init_runs() {
    assert_eq!(pinned_epoch_days(), None);
}

#[test]
fn init_pinned_today_caches_env_value() {
    let Ok(pin) = std::env::var("MINDTAPE_NOW") else {
        let out = crate::test_support::run_self_test(
            &crate::test_support::current_test_name(),
            "MINDTAPE_NOW",
            |child| {
                child.env("MINDTAPE_NOW", "2026-06-18");
            },
        );
        crate::test_support::assert_child_ok(&out);
        return;
    };
    assert_eq!(pin, "2026-06-18");
    init_pinned_today().unwrap();
    assert_eq!(pinned_epoch_days(), Some(20_622));
}
