#![allow(clippy::unwrap_used, clippy::many_single_char_names)]

use chrono::NaiveDate;

use super::{
    OccurrenceDates, append_recurred_from, apply_edits, decrement_count_edit, due_start_edits,
    on_call, shift_start, status_due_start_edits,
};
use crate::task_edit::task_with;

const SOURCE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Water plants",
  status: done(2026, 6, 1),
  priority: 3,
  due: on(2026, 6, 8),
  start: on(2026, 6, 6),
  recur: every(weeks: 1, count: 5),
)

== Body
"#;

const SOURCE_NO_RECUR_LINKS: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Water plants",
  status: done(2026, 6, 1),
  priority: 3,
  recur: every(weeks: 1),
)
"#;

const SOURCE_WITH_LINKS: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Water plants",
  status: done(2026, 6, 1),
  priority: 3,
  links: depends-on("other.typ"),
  recur: every(weeks: 1),
)
"#;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn on_call_renders_typst_source() {
    assert_eq!(on_call(date(2026, 6, 18)), "on(2026, 6, 18)");
}

#[test]
fn shift_start_moves_by_the_due_delta() {
    let shifted = shift_start(
        Some(date(2026, 6, 8)),
        Some(date(2026, 6, 6)),
        date(2026, 6, 15),
    );
    assert_eq!(shifted, Some(date(2026, 6, 13)));
}

#[test]
fn shift_start_none_without_both_dates() {
    assert_eq!(
        shift_start(None, Some(date(2026, 6, 6)), date(2026, 6, 15)),
        None
    );
    assert_eq!(
        shift_start(Some(date(2026, 6, 8)), None, date(2026, 6, 15)),
        None
    );
}

#[test]
fn due_start_edits_sets_both_when_start_given() {
    let call = task_with(SOURCE).unwrap();
    let edits = due_start_edits(SOURCE, &call, date(2026, 6, 15), Some(date(2026, 6, 13))).unwrap();
    let result = apply_edits(SOURCE, edits).unwrap();
    assert!(result.contains("due: on(2026, 6, 15)"));
    assert!(result.contains("start: on(2026, 6, 13)"));
}

#[test]
fn due_start_edits_skips_start_when_no_shift() {
    let call = task_with(SOURCE).unwrap();
    let edits = due_start_edits(SOURCE, &call, date(2026, 6, 15), None).unwrap();
    let result = apply_edits(SOURCE, edits).unwrap();
    assert!(result.contains("due: on(2026, 6, 15)"));
    assert!(
        result.contains("start: on(2026, 6, 6)"),
        "start left untouched"
    );
}

#[test]
fn status_due_start_edits_stamps_the_initial_legend_value() {
    // `Config::default()` alone carries an empty legend (the deliberate
    // unconstrained shape); the embedded floor is what supplies the shipped
    // "proposed" first value, same as a real project sees.
    let config = mindtape::parse_config("").unwrap();
    let dates = OccurrenceDates {
        today: (2026, 6, 15),
        old_due: Some(date(2026, 6, 8)),
        old_start: Some(date(2026, 6, 6)),
        next_due: date(2026, 6, 15),
    };
    let edits = status_due_start_edits(SOURCE, &config, &dates).unwrap();
    let result = apply_edits(SOURCE, edits).unwrap();
    // The embedded floor's first non-terminal legend value is "proposed".
    assert!(result.contains("proposed(2026, 6, 15)"));
    assert!(result.contains("due: on(2026, 6, 15)"));
    assert!(result.contains("start: on(2026, 6, 13)"));
}

#[test]
fn append_recurred_from_creates_links_when_absent() {
    let edit = append_recurred_from(SOURCE_NO_RECUR_LINKS, "chore").unwrap();
    let result = apply_edits(SOURCE_NO_RECUR_LINKS, vec![edit]).unwrap();
    assert!(result.contains(r#"links: recurred-from("chore.typ"),"#));
}

#[test]
fn append_recurred_from_preserves_existing_edges() {
    let edit = append_recurred_from(SOURCE_WITH_LINKS, "chore").unwrap();
    let result = apply_edits(SOURCE_WITH_LINKS, vec![edit]).unwrap();
    assert!(result.contains(r#"depends-on("other.typ") + recurred-from("chore.typ")"#));
}

#[test]
fn decrement_count_edit_none_when_no_count_set() {
    assert!(
        decrement_count_edit(SOURCE_NO_RECUR_LINKS, None)
            .unwrap()
            .is_none()
    );
}

#[test]
fn decrement_count_edit_rewrites_the_nested_count() {
    let edit = decrement_count_edit(SOURCE, Some(5)).unwrap().unwrap();
    let result = apply_edits(SOURCE, vec![edit]).unwrap();
    assert!(result.contains("every(weeks: 1, count: 4)"));
}

#[test]
fn decrement_count_edit_saturates_at_zero() {
    let edit = decrement_count_edit(SOURCE, Some(0)).unwrap().unwrap();
    let result = apply_edits(SOURCE, vec![edit]).unwrap();
    assert!(result.contains("every(weeks: 1, count: 0)"));
}
