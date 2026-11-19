//! Tests for the editor session's buffer: parsing a saved buffer back into
//! typed fill values, and the keys the editor changed. Also the error a refused
//! buffer carries when it re-opens. Scaffold rendering lives in
//! `cmd/add/scaffold_tests.rs`, and editor invocation and open/close gating in
//! `cmd/add/interactive_tests_editor.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::{AddConfig, RawValue};

use super::{Rejected, changed_keys, parse_buffer, rejected_buffer, strip_error_lines};
use crate::cmd::add::scaffold::scaffold;
use crate::cmd::add::scaffold::tests::specs;
use crate::diag::Diag;

#[test]
fn parse_back_yields_typed_values() {
    let buffer = "\
title: \"Fix the parser\"
priority: 8
tags:
  - backend
  - parser
summary: |-
  Line one
  Line two
";
    let values = parse_buffer(buffer).unwrap();
    assert_eq!(values["title"], RawValue::Str("Fix the parser".to_owned()));
    assert_eq!(values["priority"], RawValue::Int(8));
    assert_eq!(
        values["tags"],
        RawValue::List(vec!["backend".to_owned(), "parser".to_owned()])
    );
    assert_eq!(
        values["summary"],
        RawValue::Str("Line one\nLine two".to_owned())
    );
}

#[test]
fn parse_back_drops_empty_and_null() {
    let buffer = "\
title: \"Kept\"
priority:
summary: |-
";
    let values = parse_buffer(buffer).unwrap();
    assert!(values.contains_key("title"));
    assert!(!values.contains_key("priority"), "null dropped");
    assert!(!values.contains_key("summary"), "empty block dropped");
}

#[test]
fn parse_back_rejects_non_mapping() {
    assert!(parse_buffer("- a\n- b\n").is_err());
}

// ── changed_keys: a scaffold default the editor left alone is not input ──

#[test]
fn changed_keys_ignores_defaults_the_scaffold_prefilled() {
    let buffer = scaffold(&AddConfig::default(), &specs(), &BTreeMap::new());
    let baseline = parse_buffer(&buffer).unwrap();
    let saved = parse_buffer(&buffer).unwrap();

    assert!(changed_keys(&baseline, &saved).is_empty());
}

#[test]
fn changed_keys_counts_a_value_that_differs_from_the_scaffold() {
    let buffer = scaffold(&AddConfig::default(), &specs(), &BTreeMap::new());
    let baseline = parse_buffer(&buffer).unwrap();
    let saved =
        parse_buffer("title: \"Edited\"\npriority: 5\ntags: []\nsummary: |-\n  Body\n").unwrap();

    let changed = changed_keys(&baseline, &saved);

    assert!(changed.contains("title"), "{changed:?}");
    assert!(changed.contains("summary"), "{changed:?}");
    assert!(!changed.contains("priority"), "{changed:?}");
}

// ── rejected_buffer: a refused buffer re-opens with its error above the values ──

#[test]
fn a_rejected_buffer_carries_its_error_above_the_values() {
    let rejected = Rejected {
        buffer: "title: \"Sort tasks without a due date last\"\nimpact: five".to_owned(),
        error: Diag::usage("slot has the wrong type").value("impact"),
    };
    assert_eq!(
        rejected_buffer(&rejected),
        "# error: slot has the wrong type: 'impact'\n\
         title: \"Sort tasks without a due date last\"\n\
         impact: five"
    );
}

#[test]
fn a_buffer_that_comes_back_twice_carries_one_error() {
    // The editor saved the previous round's error line back with the values.
    let rejected = Rejected {
        buffer: "# error: the first refusal\ntitle: \"Kept\"\n".to_owned(),
        error: Diag::usage("the second refusal"),
    };
    assert_eq!(
        rejected_buffer(&rejected),
        "# error: the second refusal\ntitle: \"Kept\"\n"
    );
}

#[test]
fn strip_error_lines_keeps_a_comment_that_is_not_ours() {
    let buffer = "# my own note\n# error: ours\ntitle: \"Kept\"\n";
    assert_eq!(strip_error_lines(buffer), buffer);
}

// ── parse_buffer: an explicit null parses to the unset value ──

#[test]
fn parse_buffer_null_returns_empty() {
    let values = parse_buffer("~").unwrap();
    assert!(values.is_empty());
}
