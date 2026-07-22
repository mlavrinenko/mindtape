//! Coverage for the error and fall-through arms of the `function` dispatcher
//! and its helpers — the paths a happy-path filter never walks.
//!
//! `function` is a flat match over filter-function names, so its cyclomatic
//! complexity is dispatch breadth. Leaving the rare arms unexercised is what
//! pushed it over the CRAP threshold (`just crap`); these tests walk them.

#![allow(clippy::unwrap_used)]

use super::{hit, sample};
use crate::config::Config;
use crate::filter::{Filter, FilterError};
use crate::model::Task;
use crate::parse_config;

fn err(expr: &str, task: &Task) -> FilterError {
    Filter::parse(expr)
        .unwrap()
        .matches(task, &Config::default())
        .unwrap_err()
}

#[test]
fn miss_propagates_an_unknown_field_error() {
    // `miss(...)` negates `present`, so its error path is separate from `has`.
    let got = err("miss(bogus)", &sample());
    assert!(matches!(got, FilterError::UnknownField(_)), "{got:?}");
}

#[test]
fn search_matches_the_id_when_the_title_does_not() {
    // Title match short-circuits `||`; only a title miss reaches the id side.
    let mut task = sample();
    task.id = "mod-001".to_owned();
    task.title = "Nothing To See".to_owned();
    assert!(hit("search(\"mod-0\")", &task));
    assert!(!hit("search(\"absent\")", &task));
}

#[test]
fn stale_with_a_non_integer_literal_errors() {
    // A quoted non-number reaches the parse, unlike the bare identifier that
    // `single_literal` rejects first.
    let got = err("stale(\"soon\")", &sample());
    assert!(
        matches!(&got, FilterError::Invalid(msg) if msg.contains("non-negative integer")),
        "{got:?}"
    );
}

#[test]
fn unknown_function_errors() {
    let got = err("bogus()", &sample());
    assert!(
        matches!(&got, FilterError::Invalid(msg) if msg.contains("unknown function: bogus")),
        "{got:?}"
    );
}

#[test]
fn numeric_field_falls_back_to_string_compare() {
    // A non-numeric literal cannot be parsed as a number, so the numeric field
    // is formatted and compared as text ("8" sorts before "eight").
    let task = sample(); // priority 8
    assert!(!hit("priority == \"eight\"", &task));
    assert!(hit("priority != \"eight\"", &task));
    assert!(hit("priority < \"eight\"", &task));
}

#[test]
fn in_view_with_an_empty_filter_matches_everything() {
    let cfg = parse_config("[views.all]\nfilter = ''\n").unwrap();
    let task = sample();
    assert!(Filter::parse("@all").unwrap().matches(&task, &cfg).unwrap());
}

#[test]
fn task_without_reviewed_or_status_date_is_stale() {
    // No date to measure against at all — treated as stale.
    let mut task = sample();
    task.reviewed = None;
    task.status = None;
    assert!(hit("stale(30)", &task));
}

#[test]
fn contains_reads_numeric_fields_and_rejects_missing_ones() {
    let task = sample(); // priority 8, start None
    assert!(hit("contains(priority, \"8\")", &task));
    assert!(!hit("contains(priority, \"9\")", &task));
    assert!(!hit("contains(start, \"2026\")", &task));
}
