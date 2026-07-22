#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{eval_predicate, eval_score, parse_check, score_interval};
use crate::framework::{Input, InputKind};
use crate::numeric::InputValue;

fn num(value: f64) -> InputValue {
    InputValue::Num(value)
}

fn number(min: f64, max: f64) -> Input {
    Input {
        kind: InputKind::Number,
        min: Some(min),
        max: Some(max),
        values: Vec::new(),
    }
}

const ICE: &str = "round(impact * confidence * ease * 0.09)";

#[test]
fn eval_score_computes_ice() {
    let inputs = BTreeMap::from([
        ("impact".to_owned(), num(10.0)),
        ("confidence".to_owned(), num(0.9)),
        ("ease".to_owned(), num(10.0)),
    ]);
    assert_eq!(eval_score(ICE, &inputs).unwrap(), 8.0);
}

#[test]
fn eval_score_handles_low_inputs() {
    let inputs = BTreeMap::from([
        ("impact".to_owned(), num(5.0)),
        ("confidence".to_owned(), num(0.5)),
        ("ease".to_owned(), num(8.0)),
    ]);
    // 5 * 0.5 * 8 * 0.09 = 1.8 -> round 2.
    assert_eq!(eval_score(ICE, &inputs).unwrap(), 2.0);
}

#[test]
fn eval_predicate_reads_bool_and_enum() {
    let inputs = BTreeMap::from([
        ("urgent".to_owned(), InputValue::Bool(true)),
        ("important".to_owned(), InputValue::Bool(false)),
    ]);
    assert!(eval_predicate("urgent && !important", &inputs).unwrap());
    assert!(!eval_predicate("urgent && important", &inputs).unwrap());

    let level = BTreeMap::from([("level".to_owned(), InputValue::Text("must".to_owned()))]);
    assert!(eval_predicate("level == \"must\"", &level).unwrap());
}

#[test]
fn score_interval_bounds_ice_to_scale() {
    let inputs = BTreeMap::from([
        ("impact".to_owned(), number(1.0, 10.0)),
        ("confidence".to_owned(), number(0.0, 1.0)),
        ("ease".to_owned(), number(1.0, 10.0)),
    ]);
    let interval = score_interval(ICE, &inputs).unwrap();
    assert_eq!(interval.lo, 0.0);
    assert_eq!(interval.hi, 9.0);
}

#[test]
fn score_interval_handles_division() {
    let inputs = BTreeMap::from([
        ("cost_of_delay".to_owned(), number(0.0, 10.0)),
        ("job_size".to_owned(), number(1.0, 10.0)),
    ]);
    let interval = score_interval("round(cost_of_delay / job_size * 0.9)", &inputs).unwrap();
    assert_eq!(interval.lo, 0.0);
    assert_eq!(interval.hi, 9.0);
}

#[test]
fn score_interval_rejects_division_spanning_zero() {
    let inputs = BTreeMap::from([
        ("a".to_owned(), number(1.0, 10.0)),
        ("b".to_owned(), number(0.0, 10.0)),
    ]);
    assert!(score_interval("a / b", &inputs).is_err());
}

#[test]
fn score_interval_rejects_unbounded_input() {
    let inputs = BTreeMap::from([("a".to_owned(), number(1.0, 10.0))]);
    // `b` is undeclared -> unbounded -> rejected.
    assert!(score_interval("a + b", &inputs).is_err());
}

#[test]
fn parse_check_rejects_invalid_syntax() {
    assert!(parse_check("(").is_err());
}

#[test]
fn score_interval_handles_negation() {
    let inputs = BTreeMap::from([("a".to_owned(), number(1.0, 10.0))]);
    let iv = score_interval("-a", &inputs).unwrap();
    assert_eq!(iv.lo, -10.0);
    assert_eq!(iv.hi, -1.0);
}

#[test]
fn score_interval_handles_addition() {
    let inputs = BTreeMap::from([
        ("a".to_owned(), number(1.0, 10.0)),
        ("b".to_owned(), number(1.0, 5.0)),
    ]);
    let iv = score_interval("a + b", &inputs).unwrap();
    assert_eq!(iv.lo, 2.0);
    assert_eq!(iv.hi, 15.0);
}

#[test]
fn score_interval_handles_subtraction() {
    let inputs = BTreeMap::from([
        ("a".to_owned(), number(1.0, 10.0)),
        ("b".to_owned(), number(1.0, 5.0)),
    ]);
    let iv = score_interval("a - b", &inputs).unwrap();
    assert_eq!(iv.lo, -4.0);
    assert_eq!(iv.hi, 9.0);
}

#[test]
fn score_interval_handles_multiplication() {
    let inputs = BTreeMap::from([
        ("a".to_owned(), number(0.25, 0.5)),
        ("b".to_owned(), number(4.0, 8.0)),
    ]);
    let iv = score_interval("a * b", &inputs).unwrap();
    assert_eq!(iv.lo, 1.0);
    assert_eq!(iv.hi, 4.0);
}

#[test]
fn score_interval_arithmetic_division() {
    let inputs = BTreeMap::from([
        ("a".to_owned(), number(2.0, 8.0)),
        ("b".to_owned(), number(2.0, 4.0)),
    ]);
    let iv = score_interval("a / b", &inputs).unwrap();
    assert_eq!(iv.lo, 0.5);
    assert_eq!(iv.hi, 4.0);
}
