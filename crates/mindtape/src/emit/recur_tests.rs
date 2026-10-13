//! Round-trip tests for the `recur` field: [`super::emit_task`] on `every(...)`
//! and `rrule(...)` recurrences re-harvests to an equal [`Recur`].
#![allow(clippy::unwrap_used)]

use super::emit_task;
use super::tests::{CONFIG, assert_round_trips, setup, status};
use crate::model::Task;
use crate::recur::{PeriodUnit, Recur, Rule};
use crate::test_support::{bare_task, tempdir};

/// A `wip` task named `id` carrying `recur`.
fn recurring_task(id: &str, recur: Recur) -> Task {
    let mut task = bare_task(id);
    task.status = Some(status("wip", "2026-06-01"));
    task.recur = Some(recur);
    task
}

/// A recurrence of `rule` with every policy field left to the project default.
fn plain(rule: Rule) -> Recur {
    Recur {
        rule,
        anchor: None,
        until: None,
        count: None,
        materialize: None,
    }
}

fn period(unit: PeriodUnit, value: u32, weekdays: Vec<chrono::Weekday>) -> Rule {
    Rule::Period {
        unit,
        value,
        weekdays,
        start: None,
    }
}

#[test]
fn round_trips_recur_every() {
    let recur = Recur {
        anchor: Some("due".to_owned()),
        until: Some("2027-01-01".to_owned()),
        count: Some(10),
        materialize: Some("spawn".to_owned()),
        ..plain(period(
            PeriodUnit::Weeks,
            1,
            vec![chrono::Weekday::Mon, chrono::Weekday::Wed],
        ))
    };
    assert_round_trips(&recurring_task("recur-every", recur), CONFIG, &[]);
}

#[test]
fn round_trips_recur_start() {
    let rule = Rule::Period {
        unit: PeriodUnit::Weeks,
        value: 2,
        weekdays: vec![chrono::Weekday::Mon, chrono::Weekday::Thu],
        start: chrono::NaiveDate::from_ymd_opt(2026, 6, 8),
    };
    assert_round_trips(&recurring_task("recur-start", plain(rule)), CONFIG, &[]);
}

#[test]
fn emit_recur_rrule_escape_hatch_round_trips_through_reload() {
    // `check_policy` gates `rrule(...)` behind config and the `rrule` cargo
    // feature — orthogonal to emission, and this crate's own tests build
    // without the feature — so this reloads without calling `validate` (the
    // way `assert_round_trips` does), proving the marker-parse round trip on
    // its own rather than the policy check this build cannot honour.
    let dir = tempdir();
    let rule = Rule::Rrule {
        raw: "FREQ=MONTHLY;BYDAY=1MO".to_owned(),
    };
    let recur = Recur {
        anchor: Some("due".to_owned()),
        ..plain(rule)
    };
    let task = recurring_task("rrule-recur", recur);
    let source = emit_task(&task);
    assert!(
        source.contains(r#"recur: rrule("FREQ=MONTHLY;BYDAY=1MO", anchor: "due")"#),
        "{source}"
    );
    let files = vec![(format!("{}.typ", task.id), source.clone())];
    let outcome = setup(dir.path(), CONFIG, &files);
    assert!(
        outcome.errors.is_empty(),
        "{:?}\n--- source ---\n{source}",
        outcome.errors
    );
    let reloaded = outcome
        .tasks
        .into_iter()
        .find(|found| found.id == task.id)
        .unwrap_or_else(|| panic!("task `{}` did not load\n--- source ---\n{source}", task.id));
    assert_eq!(reloaded.recur, task.recur);
}

#[test]
fn round_trips_recur_without_specific_weekdays() {
    let recur = plain(period(PeriodUnit::Days, 3, Vec::new()));
    assert_round_trips(&recurring_task("recur-no-weekdays", recur), CONFIG, &[]);
}

#[test]
fn round_trips_recur_with_every_weekday() {
    use chrono::Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
    let recur = plain(period(
        PeriodUnit::Weeks,
        1,
        vec![Mon, Tue, Wed, Thu, Fri, Sat, Sun],
    ));
    assert_round_trips(&recurring_task("recur-all-weekdays", recur), CONFIG, &[]);
}
