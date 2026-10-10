use chrono::TimeZone as _;

use super::DATE_TOKENS;

use super::{
    days_ago_iso, days_from_civil, epoch_days_to_ymd, format_iso, local_epoch_days,
    today_epoch_days, today_iso, today_ymd,
};

/// 2026-07-28T22:00:00Z, the instant the deciding task was filed at: still
/// the 28th in UTC, already the 29th three hours east.
fn late_july_evening_utc() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc
        .with_ymd_and_hms(2026, 7, 28, 22, 0, 0)
        .single()
        .expect("22:00 UTC on 2026-07-28 is unambiguous")
}

/// The offset decides the calendar date, not the underlying instant: one
/// moment, read three hours east and five hours west, is two different
/// days. This is the assertion that pins "local, not UTC" — under the old
/// UTC conversion both sides would answer the 28th.
#[test]
fn local_epoch_days_reads_the_offsets_own_calendar_date() {
    let instant = late_july_evening_utc();
    let east = chrono::FixedOffset::east_opt(3 * 3600).expect("+03:00 is a valid offset");
    let west = chrono::FixedOffset::west_opt(5 * 3600).expect("-05:00 is a valid offset");

    assert_eq!(
        local_epoch_days(&instant.with_timezone(&east)),
        u64::try_from(days_from_civil(2026, 7, 29)).expect("after the epoch")
    );
    assert_eq!(
        local_epoch_days(&instant.with_timezone(&west)),
        u64::try_from(days_from_civil(2026, 7, 28)).expect("after the epoch")
    );
}

/// UTC is just the zero-offset case of the same conversion.
#[test]
fn local_epoch_days_at_utc_is_the_utc_date() {
    assert_eq!(
        local_epoch_days(&late_july_evening_utc()),
        u64::try_from(days_from_civil(2026, 7, 28)).expect("after the epoch")
    );
}

/// The clock fallback: no `MINDTAPE_NOW` pin is set in this process (the
/// pin is exercised out-of-process — see `date/pin_tests.rs` and
/// `crates/mindtape-cli/tests/date_pin_e2e.rs`), so `today_epoch_days`
/// must report the system clock. Bracketing it between two samples taken
/// around the call is rollover-safe: if midnight falls inside the window,
/// the answer is still one of the two ends.
#[test]
fn today_epoch_days_falls_back_to_the_system_clock() {
    let before = local_epoch_days(&chrono::Local::now());
    let days = today_epoch_days();
    let after = local_epoch_days(&chrono::Local::now());
    assert!(
        (before..=after).contains(&days),
        "today_epoch_days() = {days} outside the sampled window {before}..={after}"
    );
}

#[test]
fn epoch_zero_is_1970_01_01() {
    assert_eq!(epoch_days_to_ymd(0), (1970, 1, 1));
}

#[test]
fn known_date_2026_06_18() {
    // 2026-06-18 = day 20622 since Unix epoch
    assert_eq!(epoch_days_to_ymd(20_622), (2026, 6, 18));
}

#[test]
fn days_from_civil_is_the_inverse_of_epoch_days_to_ymd() {
    assert_eq!(days_from_civil(1970, 1, 1), 0);
    assert_eq!(days_from_civil(2026, 6, 18), 20_622);
}

#[test]
fn days_from_civil_before_epoch_is_negative() {
    assert!(days_from_civil(1969, 12, 31) < 0);
}

#[test]
fn today_iso_looks_like_a_date() {
    let date = today_iso();
    let parts: Vec<&str> = date.split('-').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(date.len(), 10);
}

#[test]
fn today_ymd_roundtrips_to_iso() {
    let (year, month, day) = today_ymd();
    let iso = format!("{year:04}-{month:02}-{day:02}");
    assert_eq!(iso, today_iso());
}

#[test]
fn days_ago_iso_is_before_today() {
    let ago = days_ago_iso(1);
    let today = today_iso();
    assert!(ago < today, "{ago} should be before {today}");
}

#[test]
fn days_ago_iso_zero_equals_today() {
    assert_eq!(days_ago_iso(0), today_iso());
}

#[test]
fn days_ago_saturates_at_epoch() {
    // Far in the past: should not panic
    let _ = days_ago_iso(u64::MAX);
}

#[test]
fn every_token_in_the_table_is_read_by_format_iso() {
    for t in DATE_TOKENS {
        let out = format_iso("2026-03-07", t.token);
        assert_ne!(out, t.token, "{} passed through literally", t.token);
        assert_eq!(out, t.render(2026, 3, 7));
    }
}

#[test]
fn tokens_are_ordered_longest_first() {
    for (i, a) in DATE_TOKENS.iter().enumerate() {
        for b in DATE_TOKENS.iter().skip(i + 1) {
            assert!(
                !b.token.starts_with(a.token) || a.token == b.token,
                "{} shadows {}",
                a.token,
                b.token
            );
        }
    }
}

#[test]
fn format_iso_default_reproduces_iso() {
    assert_eq!(format_iso("2026-06-27", "YYYY-MM-DD"), "2026-06-27");
}

#[test]
fn format_iso_reordered_dd_mm_yyyy() {
    assert_eq!(format_iso("2026-06-27", "DD.MM.YYYY"), "27.06.2026");
}

#[test]
fn format_iso_abbreviated_month() {
    assert_eq!(format_iso("2026-06-27", "DD MMM YYYY"), "27 Jun 2026");
}

#[test]
fn format_iso_full_month_name() {
    assert_eq!(format_iso("2026-06-27", "MMMM DD, YYYY"), "June 27, 2026");
}

#[test]
fn format_iso_two_digit_year() {
    assert_eq!(format_iso("2026-06-27", "DD/MM/YY"), "27/06/26");
}

#[test]
fn format_iso_bare_month_and_day() {
    assert_eq!(format_iso("2026-06-07", "M/D/YYYY"), "6/7/2026");
}

#[test]
fn format_iso_invalid_input_passes_through() {
    assert_eq!(format_iso("not-a-date", "DD.MM.YYYY"), "not-a-date");
}

#[test]
fn format_iso_mmmm_not_eaten_by_mm() {
    // MMMM must match before MM
    assert_eq!(format_iso("2026-01-15", "MMMM"), "January");
}

#[test]
fn epoch_days_to_ymd_last_day_of_era() {
    // day_in_era = 146096 exercises -day_in_era/146096 (col 62 -/+)
    // and month_idx = 11 (col 173 - vs /).
    assert_eq!(epoch_days_to_ymd(11_016), (2000, 2, 29));
}

#[test]
fn epoch_days_to_ymd_era_boundary_operators() {
    // day_in_era = 36524 exercises +day_in_era/36524 (col 41 +/- and +/*)
    // and -day_in_era/146096 rewritten as / (col 62 - vs /) — the
    // mutated operator changes year_in_era from 100 to 99.
    assert_eq!(epoch_days_to_ymd(47_541), (2100, 3, 1));
}

#[test]
fn days_from_civil_before_year_1() {
    // Exercises the y < 0 branch in era calculation (col 191 - vs +, - vs /).
    assert_eq!(days_from_civil(0, 1, 1), -719_528);
}

#[test]
fn days_from_civil_february() {
    // Month = 2 exercises the m > 2 vs m >= 2 boundary (col 194).
    // The >= mutant misroutes month=2 into the m - 3 arm, producing
    // u64 underflow (panic or wrapped garbage).
    assert_eq!(days_from_civil(1970, 2, 1), 31);
}
