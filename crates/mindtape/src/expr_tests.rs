#![allow(clippy::unwrap_used)]

use evalexpr::{DefaultNumericTypes, Operator, build_operator_tree};

use super::{compare_ordering, extract_identifier, extract_literal, resolve_now_or_literal};

fn tree(expr: &str) -> evalexpr::Node<DefaultNumericTypes> {
    build_operator_tree::<DefaultNumericTypes>(expr).unwrap()
}

#[test]
fn extract_identifier_reads_bare_name() {
    let node = tree("priority");
    assert_eq!(extract_identifier(&node).unwrap(), "priority");
}

#[test]
fn extract_identifier_rejects_literal() {
    let node = tree("\"nope\"");
    assert!(extract_identifier(&node).is_err());
}

#[test]
fn extract_literal_reads_string() {
    let node = tree("\"hello\"");
    assert_eq!(extract_literal(&node).unwrap(), "hello");
}

#[test]
fn extract_literal_reads_negative_number() {
    let node = tree("-5");
    assert_eq!(extract_literal(&node).unwrap(), "-5");
}

#[test]
fn extract_literal_reads_bool() {
    let node = tree("true");
    assert_eq!(extract_literal(&node).unwrap(), "true");
}

#[test]
fn extract_literal_rejects_identifier() {
    let node = tree("priority");
    assert!(extract_literal(&node).is_err());
}

#[test]
fn resolve_now_or_literal_resolves_now_call() {
    let node = tree("now()");
    let value = resolve_now_or_literal(&node).unwrap();
    assert_eq!(value.len(), 10, "expected an ISO date, got {value:?}");
}

#[test]
fn resolve_now_or_literal_rejects_other_function() {
    let node = tree("weekday()");
    assert!(resolve_now_or_literal(&node).is_err());
}

#[test]
fn resolve_now_or_literal_passes_through_literal() {
    let node = tree("\"2026-07-01\"");
    assert_eq!(resolve_now_or_literal(&node).unwrap(), "2026-07-01");
}

#[test]
fn compare_ordering_handles_each_operator() {
    use std::cmp::Ordering::{Equal, Greater, Less};
    assert!(compare_ordering(&Operator::Eq, Some(Equal)));
    assert!(compare_ordering(&Operator::Neq, Some(Less)));
    assert!(compare_ordering(&Operator::Lt, Some(Less)));
    assert!(compare_ordering(&Operator::Gt, Some(Greater)));
    assert!(compare_ordering(&Operator::Leq, Some(Equal)));
    assert!(compare_ordering(&Operator::Geq, Some(Equal)));
}

#[test]
fn compare_ordering_none_only_satisfies_neq() {
    assert!(compare_ordering(&Operator::Neq, None));
    assert!(!compare_ordering(&Operator::Eq, None));
    assert!(!compare_ordering(&Operator::Lt, None));
}
