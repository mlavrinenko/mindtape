//! `starts_with("text", field)`'s required needle-then-field grammar: the
//! field vocabulary and text-only reader it shares with `contains()`, the case
//! fold both use, and the errors that name the call. Lives in the filter test
//! family so it shares `sample()` and `hit()`, and out of `tests.rs` for the
//! line cap.

#![allow(clippy::unwrap_used)]

use super::{hit, sample};
use crate::config::Config;
use crate::filter::{Filter, FilterError};

/// Parse `expr` and evaluate it on `sample()`, returning the error it must
/// raise.
fn failure(expr: &str) -> FilterError {
    Filter::parse(expr)
        .unwrap()
        .matches(&sample(), &Config::default())
        .unwrap_err()
}

/// Every `starts_with()` misuse names the predicate and shows the correct
/// call, so a reader does not have to guess the argument order.
fn assert_names_the_call(expr: &str) -> FilterError {
    let got = failure(expr);
    let message = got.to_string();
    assert!(message.contains("starts_with"), "{expr}: {got}");
    assert!(
        message.contains("starts_with(\"fix\", title)"),
        "{expr}: {got}"
    );
    got
}

#[test]
fn starts_with_matches_a_prefix_and_not_a_substring() {
    let task = sample(); // title "Add Auth"
    assert!(hit("starts_with(\"add\", title)", &task));
    assert!(!hit("starts_with(\"auth\", title)", &task));
}

#[test]
fn starts_with_folds_case_like_contains() {
    let task = sample(); // title "Add Auth"
    assert!(hit("starts_with(\"ADD\", title)", &task));
    assert!(hit("starts_with(\"aDd\", title)", &task));
}

#[test]
fn starts_with_reads_text_only_fields_as_contains_does() {
    let mut task = sample();
    task.body = "Run the migration first".to_owned();
    task.tags = vec!["backend".to_owned()];
    assert!(hit("starts_with(\"run\", body)", &task));
    assert!(hit("starts_with(\"BACK\", tags)", &task));
    assert!(hit("starts_with(\"cont\", note)", &task)); // sample's link note
}

#[test]
fn starts_with_reads_a_missing_value_as_no_match() {
    let task = sample(); // assignee is None
    assert!(!hit("starts_with(\"ada\", assignee)", &task));
}

#[test]
fn starts_with_refuses_a_field_outside_the_contains_vocabulary() {
    let got = assert_names_the_call("starts_with(\"fix\", bogus)");
    assert!(got.to_string().contains("bogus"), "{got}");
}

#[test]
fn starts_with_refuses_a_numeric_field() {
    let got = assert_names_the_call("starts_with(\"8\", priority)");
    assert!(got.to_string().contains("priority"), "{got}");
}

#[test]
fn starts_with_refuses_a_wrong_argument_count() {
    for expr in ["starts_with(\"fix\")", "starts_with(\"fix\", title, id)"] {
        assert_names_the_call(expr);
    }
}

#[test]
fn starts_with_refuses_a_non_literal_needle_or_field() {
    assert_names_the_call("starts_with(title, title)");
    assert_names_the_call("starts_with(\"fix\", \"title\")");
}
