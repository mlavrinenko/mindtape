//! Tests for `mt rm`'s target resolution message: a lone failure reads as its
//! own line, several are numbered under one count.
#![allow(clippy::unwrap_used)]

use super::unresolved_message;

/// A single failure is returned verbatim — no count, no list.
#[test]
fn a_single_problem_is_returned_verbatim() {
    assert_eq!(
        unresolved_message(&["alpha: names no task".to_owned()]),
        "alpha: names no task"
    );
}

/// Several failures are counted and listed, one per line.
#[test]
fn several_problems_are_counted_and_listed() {
    let message = unresolved_message(&["alpha: x".to_owned(), "beta: y".to_owned()]);
    assert_eq!(
        message,
        "2 target(s) did not resolve:\n  alpha: x\n  beta: y"
    );
}
