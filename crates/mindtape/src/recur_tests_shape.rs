//! The `recur` field's shape-error channel: what a value `every()`/`rrule()`
//! passed through must tell the author when the Rust layer cannot read it.
//! Split from `recur_tests.rs` (which pins the parse itself) for size; shares
//! its fixtures.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use chrono::Weekday;

use super::tests::{dict, every_val, parse, period_rule};
use super::{HVal, Rule, parse_recur};

/// The reports [`parse_recur`] pushes for `value`, discarding the recurrence
/// itself — the shape-error tests below care about what the author is told,
/// not what survived.
fn reports(value: &HVal) -> Vec<crate::marker::ShapeError> {
    let mut errors = Vec::new();
    drop(parse_recur(Some(value), &mut errors));
    errors
}

/// Assert `value` produces exactly one report, naming `path` and `message`.
fn assert_one_report(value: &HVal, path: &str, message: &str) {
    let errors = reports(value);
    assert_eq!(errors.len(), 1, "expected one report, got {errors:?}");
    assert_eq!(errors[0].field, path);
    assert_eq!(errors[0].message, message);
}

#[test]
fn parse_recur_non_dict_value_reports_the_field() {
    assert_one_report(
        &HVal::Str("weekly".to_owned()),
        "recur",
        "expected a recurrence like `every(weeks: 2)`, found a string",
    );
}

#[test]
fn parse_recur_missing_rule_reports_the_field() {
    assert_one_report(
        &dict(vec![("anchor", HVal::Str("due".to_owned()))]),
        "recur",
        "expected a recurrence like `every(weeks: 2)`, found one with no `rule`",
    );
}

#[test]
fn parse_recur_missing_rule_kind_reports_it() {
    let val = dict(vec![("rule", dict(vec![]))]);
    assert_one_report(&val, "recur.rule.kind", "expected a string, found none");
    assert_eq!(parse(Some(&val)), None);
}

#[test]
fn parse_recur_unknown_unit_reports_it() {
    let val = every_val(period_rule("fortnights", 1, None), vec![]);
    assert_one_report(
        &val,
        "recur.rule.unit",
        "expected one of days, weeks, months, years, found `fortnights`",
    );
    assert_eq!(parse(Some(&val)), None);
}

/// `every(weeks: "two")` — the period length is the one part of a period rule
/// an author passes in by hand, so its report names the argument they wrote
/// (`recur.weeks`), not the marker's `rule.value`.
#[test]
fn parse_recur_non_number_period_value_reports_the_authored_unit() {
    let mut rule = period_rule("weeks", 1, None);
    let HVal::Dict(fields) = &mut rule else {
        panic!("period_rule builds a dict");
    };
    fields.insert("value".to_owned(), HVal::Str("two".to_owned()));
    assert_one_report(
        &every_val(rule, vec![]),
        "recur.weeks",
        "expected a whole number, found a string",
    );
}

/// Parse a weekly period narrowed by `on`, returning the weekday set that
/// survived alongside every report — the `on:` tests differ only in what they
/// hand it, and both care about the pair.
fn weekdays_and_reports(on: HVal) -> (Vec<Weekday>, Vec<crate::marker::ShapeError>) {
    let val = every_val(period_rule("weeks", 1, Some(on)), vec![]);
    let mut errors = Vec::new();
    let recur = parse_recur(Some(&val), &mut errors).unwrap();
    let Rule::Period { weekdays, .. } = recur.rule else {
        panic!("expected a period rule");
    };
    (weekdays, errors)
}

#[test]
fn parse_recur_non_array_on_reports_shape_error_not_silent_empty() {
    let (weekdays, errors) = weekdays_and_reports(HVal::Str("mon".to_owned()));

    assert!(weekdays.is_empty());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "recur.on");
    assert_eq!(errors[0].message, "expected an array, found a string");
}

/// A well-shaped `on:` array with unreadable ENTRIES: a string that is no
/// weekday and a value that is no string. Each names its own index, and the
/// entries that did read survive — narrowing a period to fewer days than the
/// author wrote is exactly the silent change `mt recur` would then act on.
#[test]
fn parse_recur_reports_each_unreadable_on_entry_by_index() {
    let (weekdays, errors) = weekdays_and_reports(HVal::Array(vec![
        HVal::Str("mon".to_owned()),
        HVal::Str("funday".to_owned()),
        HVal::Int(7),
    ]));

    assert_eq!(weekdays, vec![Weekday::Mon]);
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].field, "recur.on[1]");
    assert_eq!(
        errors[0].message,
        "expected a weekday name (mon, tue, wed, thu, fri, sat, sun), found `funday`"
    );
    assert_eq!(errors[1].field, "recur.on[2]");
    assert_eq!(
        errors[1].message,
        "expected a weekday name (mon, tue, wed, thu, fri, sat, sun), found a number"
    );
}

/// The policy half of the dict reports the same way, each against the argument
/// that carried it.
#[test]
fn parse_recur_reports_a_wrong_typed_policy_argument() {
    let cases = [
        (
            "anchor",
            HVal::Int(5),
            "recur.anchor",
            "expected a string, found a number",
        ),
        (
            "count",
            HVal::Str("many".to_owned()),
            "recur.count",
            "expected a whole number, found a string",
        ),
    ];
    for (key, value, path, message) in cases {
        let val = every_val(period_rule("weeks", 1, None), vec![(key, value)]);
        assert_one_report(&val, path, message);
    }
}
