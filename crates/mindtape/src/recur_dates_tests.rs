#![allow(clippy::unwrap_used, clippy::many_single_char_names)]

use chrono::Weekday;

use super::*;
use crate::recur::{PeriodUnit, Recur, Rule};

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn period(unit: PeriodUnit, value: u32) -> Rule {
    Rule::Period {
        unit,
        value,
        weekdays: vec![],
    }
}

fn recur(rule: Rule, anchor: Option<&str>, until: Option<&str>, count: Option<u32>) -> Recur {
    Recur {
        rule,
        anchor: anchor.map(ToOwned::to_owned),
        until: until.map(ToOwned::to_owned),
        count,
        materialize: None,
    }
}

#[test]
fn weekly_period_advances_seven_days() {
    let next = advance_once(&period(PeriodUnit::Weeks, 1), date(2026, 6, 1)).unwrap();
    assert_eq!(next, date(2026, 6, 8));
}

#[test]
fn monthly_period_advances_one_month() {
    let next = advance_once(&period(PeriodUnit::Months, 1), date(2026, 6, 15)).unwrap();
    assert_eq!(next, date(2026, 7, 15));
}

#[test]
fn monthly_period_clamps_end_of_month() {
    // Jan 31 + 1 month has no Feb 31 — clamp to the last valid day.
    let next = advance_once(&period(PeriodUnit::Months, 1), date(2026, 1, 31)).unwrap();
    assert_eq!(next, date(2026, 2, 28));
}

#[test]
fn yearly_period_clamps_leap_day() {
    // 2024-02-29 + 1 year: 2025 is not a leap year.
    let next = advance_once(&period(PeriodUnit::Years, 1), date(2024, 2, 29)).unwrap();
    assert_eq!(next, date(2025, 2, 28));
}

#[test]
fn daily_period_advances_by_value() {
    let next = advance_once(&period(PeriodUnit::Days, 3), date(2026, 6, 1)).unwrap();
    assert_eq!(next, date(2026, 6, 4));
}

#[test]
fn weekday_set_finds_next_matching_day() {
    let rule = Rule::Period {
        unit: PeriodUnit::Weeks,
        value: 1,
        weekdays: vec![Weekday::Mon, Weekday::Wed, Weekday::Fri],
    };
    // 2026-06-01 is a Monday; next in the set strictly after it is Wednesday.
    assert_eq!(date(2026, 6, 1).weekday(), Weekday::Mon);
    let next = advance_once(&rule, date(2026, 6, 1)).unwrap();
    assert_eq!(next, date(2026, 6, 3));
    assert_eq!(next.weekday(), Weekday::Wed);
}

#[test]
fn weekday_set_wraps_into_next_week() {
    let rule = Rule::Period {
        unit: PeriodUnit::Weeks,
        value: 1,
        weekdays: vec![Weekday::Mon],
    };
    // Friday -> next Monday, three days later.
    assert_eq!(date(2026, 6, 5).weekday(), Weekday::Fri);
    let next = advance_once(&rule, date(2026, 6, 5)).unwrap();
    assert_eq!(next, date(2026, 6, 8));
}

#[cfg(not(feature = "rrule"))]
#[test]
fn rrule_rule_is_not_implemented_without_the_cargo_feature() {
    let rule = Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    };
    assert_eq!(
        advance_once(&rule, date(2026, 6, 1)),
        Err(RecurError::RruleNotImplemented)
    );
}

#[cfg(feature = "rrule")]
#[test]
fn rrule_rule_advances_via_the_expansion_engine() {
    let rule = Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    };
    let next = advance_once(&rule, date(2026, 6, 1)).unwrap();
    assert_eq!(next, date(2026, 6, 2));
}

#[cfg(feature = "rrule")]
#[test]
fn rrule_effective_anchor_is_always_due_even_with_completion_default() {
    let rule = Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    };
    let r = recur(rule, None, None, None);
    // The project default is `"completion"`; `rrule(...)` still resolves to
    // `"due"` — RFC 5545 has no completion-relative primitive.
    assert_eq!(effective_anchor(&r, &RecurConfig::default()), "due");
}

#[cfg(feature = "rrule")]
#[test]
fn rrule_first_monday_of_the_month() {
    let rule = Rule::Rrule {
        raw: "FREQ=MONTHLY;BYDAY=1MO".to_owned(),
    };
    let next = advance_once(&rule, date(2026, 6, 15)).unwrap();
    assert_eq!(next, date(2026, 7, 6));
}

#[cfg(feature = "rrule")]
#[test]
fn rrule_last_day_of_the_month() {
    let rule = Rule::Rrule {
        raw: "FREQ=MONTHLY;BYMONTHDAY=-1".to_owned(),
    };
    let next = advance_once(&rule, date(2026, 1, 31)).unwrap();
    assert_eq!(next, date(2026, 2, 28));
}

#[cfg(feature = "rrule")]
#[test]
fn rrule_every_weekday_skips_the_weekend() {
    let rule = Rule::Rrule {
        raw: "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR".to_owned(),
    };
    // 2026-06-05 is a Friday.
    let next = advance_once(&rule, date(2026, 6, 5)).unwrap();
    assert_eq!(next, date(2026, 6, 8));
}

#[cfg(feature = "rrule")]
#[test]
fn rrule_past_until_ends_the_series() {
    let rule = Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    };
    let r = recur(rule, None, Some("2026-06-05"), None);
    assert!(!past_until(&r, date(2026, 6, 5)));
    assert!(past_until(&r, date(2026, 6, 6)));
}

#[cfg(feature = "rrule")]
#[test]
fn rrule_count_reaches_exhaustion_like_any_other_rule() {
    let rule = Rule::Rrule {
        raw: "FREQ=DAILY".to_owned(),
    };
    let r = recur(rule, None, None, Some(1));
    assert!(!exhausted(&r));
    assert_eq!(carry_count(&r), Some(0));
}

#[test]
fn completion_anchor_advances_from_today_not_due() {
    // An overdue `due` (long past) must not matter: completion always steps
    // from today, landing in the future with no catch-up needed.
    let r = recur(period(PeriodUnit::Weeks, 1), None, None, None);
    let config = RecurConfig::default();
    let today = date(2026, 6, 18);
    let stale_due = date(2026, 1, 1);
    let next = next_date(&r, &config, today, Some(stale_due)).unwrap();
    assert_eq!(next, date(2026, 6, 25));
}

#[test]
fn due_anchor_catches_up_past_today() {
    // Weekly, due long overdue: catch-up must land strictly after today, not
    // immediately after the stale due date.
    let r = recur(period(PeriodUnit::Weeks, 1), Some("due"), None, None);
    let config = RecurConfig::default();
    let today = date(2026, 6, 18);
    let due = date(2026, 5, 1);
    let next = next_date(&r, &config, today, Some(due)).unwrap();
    assert!(next > today, "catch-up must land after today, got {next}");
    // 2026-05-01 + 7*n weeks, first value > 2026-06-18 is 2026-06-19.
    assert_eq!(next, date(2026, 6, 19));
}

#[test]
fn due_anchor_without_a_due_falls_back_to_today() {
    let r = recur(period(PeriodUnit::Weeks, 1), Some("due"), None, None);
    let config = RecurConfig::default();
    let today = date(2026, 6, 18);
    let next = next_date(&r, &config, today, None).unwrap();
    assert_eq!(next, date(2026, 6, 25));
}

#[test]
fn default_anchor_is_completion() {
    let r = recur(period(PeriodUnit::Weeks, 1), None, None, None);
    assert_eq!(effective_anchor(&r, &RecurConfig::default()), "completion");
}

#[test]
fn explicit_anchor_wins_over_config_default() {
    let r = recur(period(PeriodUnit::Weeks, 1), Some("due"), None, None);
    assert_eq!(effective_anchor(&r, &RecurConfig::default()), "due");
}

#[test]
fn effective_materialize_falls_back_to_config() {
    let r = recur(period(PeriodUnit::Weeks, 1), None, None, None);
    let config = RecurConfig {
        materialize: "mutate".to_owned(),
        ..RecurConfig::default()
    };
    assert_eq!(effective_materialize(&r, &config), "mutate");
}

#[test]
fn past_until_flags_a_next_date_beyond_the_bound() {
    let r = recur(period(PeriodUnit::Weeks, 1), None, Some("2026-06-20"), None);
    assert!(past_until(&r, date(2026, 6, 25)));
    assert!(!past_until(&r, date(2026, 6, 20)));
    assert!(!past_until(&r, date(2026, 6, 15)));
}

#[test]
fn past_until_unset_never_ends_the_series() {
    let r = recur(period(PeriodUnit::Weeks, 1), None, None, None);
    assert!(!past_until(&r, date(2099, 1, 1)));
}

#[test]
fn exhausted_only_at_zero() {
    let mut r = recur(period(PeriodUnit::Weeks, 1), None, None, Some(1));
    assert!(!exhausted(&r));
    r.count = Some(0);
    assert!(exhausted(&r));
}

#[test]
fn carry_count_decrements_and_saturates() {
    let r = recur(period(PeriodUnit::Weeks, 1), None, None, Some(1));
    assert_eq!(carry_count(&r), Some(0));
    let unbounded = recur(period(PeriodUnit::Weeks, 1), None, None, None);
    assert_eq!(carry_count(&unbounded), None);
}

#[test]
fn iso_roundtrip() {
    let d = date(2026, 6, 18);
    assert_eq!(parse_iso(&format_iso(d)), Some(d));
    assert_eq!(format_iso(d), "2026-06-18");
}

#[test]
fn ymd_to_date_roundtrips() {
    assert_eq!(ymd_to_date((2026, 6, 18)), Some(date(2026, 6, 18)));
}

#[test]
fn date_to_ymd_roundtrips() {
    assert_eq!(date_to_ymd(date(2026, 6, 18)), (2026, 6, 18));
}
