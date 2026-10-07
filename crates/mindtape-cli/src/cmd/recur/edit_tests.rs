#![allow(clippy::unwrap_used, clippy::many_single_char_names)]

use chrono::NaiveDate;

use super::{
    OccurrenceDates, apply_edits, decrement_count_edit, due_start_edits, on_call,
    set_recurred_from, shift_start, status_due_start_edits,
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

/// An occurrence that is itself a spawn: it already carries the
/// `recurred-from` edge pointing at the hop before it.
const SOURCE_ALREADY_SPAWNED: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Water plants",
  status: done(2026, 6, 1),
  priority: 3,
  links: depends-on("other.typ") + recurred-from("chore.typ"),
  recur: every(weeks: 1),
)
"#;

/// The same, with a noted edge — the case a re-render would flatten.
const SOURCE_NOTED_LINKS: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Water plants",
  status: done(2026, 6, 1),
  links: related("other.typ")[same _room_] + recurred-from("chore.typ"),
  recur: every(weeks: 1),
)
"#;

/// The already-spawned links argument plus an edge helper call elsewhere in
/// the document (a body `related`), i.e. outside the `links:` value range.
const SOURCE_BODY_EDGE_CALL: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Water plants",
  status: done(2026, 6, 1),
  priority: 3,
  links: depends-on("other.typ") + recurred-from("chore.typ"),
  recur: every(weeks: 1),
)

== Body
#related("body.typ")
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
fn set_recurred_from_creates_links_when_absent() {
    let edit = set_recurred_from(SOURCE_NO_RECUR_LINKS, "chore").unwrap();
    let result = apply_edits(SOURCE_NO_RECUR_LINKS, vec![edit]).unwrap();
    assert!(result.contains(r#"links: recurred-from("chore.typ"),"#));
}

#[test]
fn set_recurred_from_preserves_existing_edges() {
    let edit = set_recurred_from(SOURCE_WITH_LINKS, "chore").unwrap();
    let result = apply_edits(SOURCE_WITH_LINKS, vec![edit]).unwrap();
    assert!(result.contains(r#"depends-on("other.typ") + recurred-from("chore.typ")"#));
}

/// Spawning from an occurrence that is itself a spawn: its own
/// `recurred-from` is REPLACED, not added to. `recurred-from` allows one edge
/// per task, so appending wrote an occurrence `mt check` refuses — and that is
/// every second hop of the ordinary recurrence loop, not a corner.
#[test]
fn set_recurred_from_replaces_the_predecessors_own_edge() {
    let edit = set_recurred_from(SOURCE_ALREADY_SPAWNED, "chore-2026-06-08").unwrap();
    let result = apply_edits(SOURCE_ALREADY_SPAWNED, vec![edit]).unwrap();
    assert!(
        result
            .contains(r#"links: depends-on("other.typ") + recurred-from("chore-2026-06-08.typ")"#),
        "{result}"
    );
    assert!(
        !result.contains(r#"recurred-from("chore.typ")"#),
        "the older hop must not survive: {result}"
    );
}

/// A note's markup passes through verbatim rather than being re-rendered: the
/// surviving calls are copied out of the source by span, not rebuilt.
#[test]
fn set_recurred_from_keeps_a_noted_edge_verbatim() {
    let edit = set_recurred_from(SOURCE_NOTED_LINKS, "chore-2026-06-08").unwrap();
    let result = apply_edits(SOURCE_NOTED_LINKS, vec![edit]).unwrap();
    assert!(
        result.contains(r#"related("other.typ")[same _room_]"#),
        "{result}"
    );
}

/// An edge helper call outside the `links:` value's byte range is not part of
/// the argument, so it must not enter the replacement scan: including it makes
/// the gap check fail, the rewrite bail, and the predecessor's own
/// `recurred-from` survive as a duplicate.
#[test]
fn set_recurred_from_ignores_edges_outside_the_links_value() {
    let edit = set_recurred_from(SOURCE_BODY_EDGE_CALL, "chore-2026-06-08").unwrap();
    let result = apply_edits(SOURCE_BODY_EDGE_CALL, vec![edit]).unwrap();
    assert!(
        !result.contains(r#"recurred-from("chore.typ")"#),
        "the older hop must not survive: {result}"
    );
    assert!(
        result.contains(r#"recurred-from("chore-2026-06-08.typ")"#),
        "{result}"
    );
    assert!(
        result.contains(r#"related("body.typ")"#),
        "the body edge must be left untouched: {result}"
    );
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
