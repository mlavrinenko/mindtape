#![allow(clippy::unwrap_used, clippy::many_single_char_names)]

use chrono::Datelike;

use super::*;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn validate_rule_accepts_well_formed_rrule() {
    assert!(validate_rule("FREQ=MONTHLY;BYDAY=1MO").is_ok());
}

#[test]
fn validate_rule_rejects_unknown_frequency_quoting_the_rule() {
    let err = validate_rule("FREQ=NOPE").unwrap_err();
    assert!(err.contains("invalid rrule"));
    assert!(err.contains("\"FREQ=NOPE\""));
}

#[test]
fn first_monday_of_the_month() {
    // 2026-06-01 is itself a Monday but not the *first* Monday's due date in
    // this test — seed from a mid-month due date and expect July's first
    // Monday.
    let due = date(2026, 6, 15);
    let next = next_after("FREQ=MONTHLY;BYDAY=1MO", due).unwrap();
    assert_eq!(next, date(2026, 7, 6));
    assert_eq!(next.weekday(), chrono::Weekday::Mon);
}

#[test]
fn last_day_of_the_month() {
    let due = date(2026, 1, 31);
    let next = next_after("FREQ=MONTHLY;BYMONTHDAY=-1", due).unwrap();
    assert_eq!(next, date(2026, 2, 28));
}

#[test]
fn every_weekday() {
    // Friday -> next weekday occurrence is Monday, skipping the weekend.
    let due = date(2026, 6, 5);
    assert_eq!(due.weekday(), chrono::Weekday::Fri);
    let next = next_after("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR", due).unwrap();
    assert_eq!(next, date(2026, 6, 8));
    assert_eq!(next.weekday(), chrono::Weekday::Mon);
}

#[test]
fn strictly_after_due_even_when_due_is_itself_an_occurrence() {
    // DTSTART (due) is a first-Monday occurrence; the next one must be a
    // full month later, not the same day.
    let due = date(2026, 6, 1);
    assert_eq!(due.weekday(), chrono::Weekday::Mon);
    let next = next_after("FREQ=MONTHLY;BYDAY=1MO", due).unwrap();
    assert_eq!(next, date(2026, 7, 6));
}

#[test]
fn exhausted_embedded_count_reports_a_clear_error() {
    // COUNT=1 means DTSTART is the only occurrence; asking for the next one
    // strictly after it must error, not hang or silently misbehave.
    let due = date(2026, 6, 1);
    let err = next_after("FREQ=DAILY;COUNT=1", due).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("FREQ=DAILY;COUNT=1"));
}

#[test]
fn embedded_until_past_due_reports_a_clear_error() {
    let due = date(2026, 6, 1);
    let err = next_after("FREQ=DAILY;UNTIL=20260601T000000Z", due).unwrap_err();
    assert!(err.to_string().contains("FREQ=DAILY"));
}
