#![allow(clippy::unwrap_used)]

use evalexpr::{DefaultNumericTypes, Operator, build_operator_tree};

use super::{
    NAME_HYPHEN, call_args, compare_ordering, connective, extract_identifier, extract_literal,
    join_kebab_names, resolve_now_or_literal,
};

fn tree(expr: &str) -> evalexpr::Node<DefaultNumericTypes> {
    build_operator_tree::<DefaultNumericTypes>(expr).unwrap()
}

/// The argument-list wrapper child of a call: `f()` with no arguments.
fn zero_arg_call() -> evalexpr::Node<DefaultNumericTypes> {
    let root = tree("f()");
    let [func] = root.children() else {
        panic!("a call is one node")
    };
    let [wrapper] = func.children() else {
        panic!("a call wraps its argument list in one child")
    };
    wrapper.clone()
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
fn extract_literal_reads_float() {
    let node = tree("3.14");
    assert_eq!(extract_literal(&node).unwrap(), "3.14");
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

#[test]
fn call_args_of_a_zero_arg_call_is_empty() {
    let wrapper = zero_arg_call();
    assert!(call_args(std::slice::from_ref(&wrapper)).is_empty());
}

#[test]
fn connective_reports_a_root_with_the_wrong_arity() {
    let wrapper = zero_arg_call();
    let result = connective(&wrapper, &|_| Ok::<bool, String>(true), String::from);
    let outcome = result.expect("a root node is connective's to report on");
    assert!(outcome.is_err(), "an arity-less root is an error");
}

#[test]
fn join_kebab_names_keeps_an_escaped_quote_inside_a_string() {
    // `"a\"x-y"` — the escaped quote does not close the literal, so the hyphen
    // stays string text and must never be rewritten.
    let expr = "\"a\\\"x-y\"";
    assert_eq!(join_kebab_names(expr), expr);
    assert!(!join_kebab_names(expr).contains(NAME_HYPHEN));
}

#[test]
fn join_kebab_names_treats_a_dot_as_a_name_character() {
    // The run survives the dot, so the hyphen beside it still joins.
    assert_eq!(join_kebab_names("a.-b"), format!("a.{NAME_HYPHEN}b"));
}

#[test]
fn join_kebab_names_does_not_join_a_hyphen_before_a_non_name() {
    assert_eq!(join_kebab_names("a- b"), "a- b");
}

#[test]
fn join_kebab_names_keeps_a_run_going_across_a_digit() {
    // The run began at `u`, so the `8` leaves it a name and the hyphen joins.
    assert_eq!(
        join_kebab_names("utf8-name"),
        format!("utf8{NAME_HYPHEN}name")
    );
}

#[test]
fn join_kebab_names_does_not_start_a_name_on_a_digit() {
    // A run beginning with a digit is not a name, so `1-b` stays subtraction.
    assert_eq!(join_kebab_names("1-b"), "1-b");
}
