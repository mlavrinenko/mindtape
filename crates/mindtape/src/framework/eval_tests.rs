#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{ParsedFormula, eval_predicate, parse_check, score_interval};
use crate::framework::{Input, InputKind};
use crate::model::numeric::InputValue;

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

fn boolean() -> Input {
    Input {
        kind: InputKind::Bool,
        min: None,
        max: None,
        values: Vec::new(),
    }
}

const ICE: &str = "round(impact * confidence * ease * 0.09)";

/// The ICE inputs for one set of values.
fn ice_inputs(impact: f64, confidence: f64, ease: f64) -> BTreeMap<String, InputValue> {
    BTreeMap::from([
        ("impact".to_owned(), num(impact)),
        ("confidence".to_owned(), num(confidence)),
        ("ease".to_owned(), num(ease)),
    ])
}

/// The ICE score for one set of inputs, through a fresh cache.
fn score_ice(impact: f64, confidence: f64, ease: f64) -> f64 {
    ParsedFormula::default()
        .score(ICE, &ice_inputs(impact, confidence, ease))
        .unwrap()
}

#[test]
fn eval_score_computes_ice() {
    assert_eq!(score_ice(10.0, 0.9, 10.0), 8.0);
}

#[test]
fn eval_score_handles_low_inputs() {
    // 5 * 0.5 * 8 * 0.09 = 1.8 -> round 2.
    assert_eq!(score_ice(5.0, 0.5, 8.0), 2.0);
}

#[test]
fn a_parsed_formula_answers_a_second_evaluation_from_the_cache() {
    let parsed = ParsedFormula::default();
    let score = |inputs: &BTreeMap<String, InputValue>| parsed.score(ICE, inputs).unwrap();
    // The second call reads the tree the first one parsed, and still evaluates
    // it against its own inputs.
    assert_eq!(score(&ice_inputs(10.0, 0.9, 10.0)), 8.0);
    assert_eq!(score(&ice_inputs(5.0, 0.5, 8.0)), 2.0);
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

#[test]
fn score_interval_handles_bool_input() {
    // A bool-kind input spans [0, 1] — the same interval a `when` predicate
    // treats it as truthy/falsy over.
    let inputs = BTreeMap::from([("urgent".to_owned(), boolean())]);
    let iv = score_interval("urgent", &inputs).unwrap();
    assert_eq!(iv.lo, 0.0);
    assert_eq!(iv.hi, 1.0);
}

#[test]
fn score_interval_rejects_non_arithmetic_operator() {
    // A comparison is a valid `when` predicate but not a valid `score`
    // formula — interval arithmetic has no meaning for `==`.
    let inputs = BTreeMap::from([
        ("a".to_owned(), number(1.0, 10.0)),
        ("b".to_owned(), number(1.0, 10.0)),
    ]);
    let err = score_interval("a == b", &inputs).unwrap_err();
    assert!(err.contains("can't use '=='"), "{err}");
}

#[test]
fn score_interval_rejects_non_numeric_constant() {
    let err = score_interval("\"oops\"", &BTreeMap::new()).unwrap_err();
    assert!(err.contains("non-numeric constant"), "{err}");
}

#[test]
fn score_interval_rejects_wrong_function_arity() {
    // `round` takes exactly one argument; with none, `evalexpr` still wraps
    // the (empty) argument list in one `RootNode` child, which then has zero
    // children of its own where interval evaluation expects exactly one —
    // the same "wrapped argument list" shape `condition.rs`'s `no_args`
    // relies on for the zero-arg case, but unwanted here.
    let inputs = BTreeMap::from([("a".to_owned(), number(1.0, 10.0))]);
    let err = score_interval("round()", &inputs).unwrap_err();
    assert!(err.contains("incomplete"), "{err}");
}

#[test]
fn score_interval_handles_floor_and_ceil() {
    let inputs = BTreeMap::from([("a".to_owned(), number(1.2, 4.7))]);
    let floor = score_interval("floor(a)", &inputs).unwrap();
    assert_eq!(floor.lo, 1.0);
    assert_eq!(floor.hi, 4.0);
    let ceil = score_interval("ceil(a)", &inputs).unwrap();
    assert_eq!(ceil.lo, 2.0);
    assert_eq!(ceil.hi, 5.0);
}

#[test]
fn score_interval_rejects_unsupported_function() {
    let inputs = BTreeMap::from([("a".to_owned(), number(1.0, 10.0))]);
    let err = score_interval("sqrt(a)", &inputs).unwrap_err();
    assert!(err.contains("can't call 'sqrt'"), "{err}");
}
