//! Tests for the editor session's buffer: parsing a saved buffer back into
//! typed fill values, and the keys the editor changed. Also the error a refused
//! buffer carries when it re-opens. Scaffold rendering lives in
//! `cmd/add/scaffold_tests.rs`, and editor invocation and open/close gating in
//! `cmd/add/interactive_tests_editor.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use mindtape::RawValue;

use super::{Rejected, changed_keys, parse_buffer, rejected_buffer, strip_error_lines};
use crate::cmd::add::scaffold::tests::TEMPLATE;
use crate::cmd::add::support_tests::{parse, scaffold_of};
use crate::diag::Diag;
use crate::pre_write::Problem;

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
    let values = parse(buffer);
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
    let values = parse(buffer);
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
    let buffer = scaffold_of(TEMPLATE);
    let baseline = parse(&buffer);
    let saved = parse(&buffer);

    assert!(changed_keys(&baseline, &saved).is_empty());
}

#[test]
fn changed_keys_counts_a_value_that_differs_from_the_scaffold() {
    let buffer = scaffold_of(TEMPLATE);
    let baseline = parse(&buffer);
    let saved = parse("title: \"Edited\"\npriority: 5\ntags: []\nsummary: |-\n  Body\n");

    let changed = changed_keys(&baseline, &saved);

    assert!(changed.contains("title"), "{changed:?}");
    assert!(changed.contains("summary"), "{changed:?}");
    assert!(!changed.contains("priority"), "{changed:?}");
}

// ── rejected_buffer: a refused buffer re-opens with its error above the values ──

#[test]
fn a_rejected_buffer_carries_its_error_above_the_values() {
    let rejected = Rejected::from_error(
        "title: \"Sort tasks without a due date last\"\nimpact: five".to_owned(),
        &Diag::usage("slot has the wrong type").value("impact"),
    );
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
    let rejected = Rejected::from_error(
        "# error: the first refusal\ntitle: \"Kept\"\n".to_owned(),
        &Diag::usage("the second refusal"),
    );
    assert_eq!(
        rejected_buffer(&rejected),
        "# error: the second refusal\ntitle: \"Kept\"\n"
    );
}

#[test]
fn a_rejected_buffer_carries_every_breach_above_the_values() {
    let problems = [
        Problem {
            line:
                "<new task from template 'task'>:33: body.content.rules.mark-defect-step-with-red: \
                   open the step that shows the defect with 🔴"
                    .to_owned(),
            detail: Some("section `AS-IS` must match `(?m)^\\s*(\\+|\\d+\\.)\\s+🔴`".to_owned()),
        },
        Problem {
            line:
                "<new task from template 'task'>:40: body.content.rules.add-fenced-repro-to-as-is: \
                   quote what it printed"
                    .to_owned(),
            detail: None,
        },
    ];
    let rejected = Rejected::from_problems("title: \"Kept\"\n".to_owned(), &problems);

    assert_eq!(
        rejected_buffer(&rejected),
        "# error: <new task from template 'task'>:33: \
         body.content.rules.mark-defect-step-with-red: open the step that shows the defect with 🔴\n\
         # note: section `AS-IS` must match `(?m)^\\s*(\\+|\\d+\\.)\\s+🔴`\n\
         # error: <new task from template 'task'>:40: \
         body.content.rules.add-fenced-repro-to-as-is: quote what it printed\n\
         title: \"Kept\"\n"
    );
}

#[test]
fn a_blank_refusal_line_stays_a_comment() {
    let rejected = Rejected {
        buffer: "title: \"Kept\"\n".to_owned(),
        lines: vec![String::new()],
    };
    assert_eq!(rejected_buffer(&rejected), "#\ntitle: \"Kept\"\n");
}

#[test]
fn strip_error_lines_keeps_a_comment_that_is_not_ours() {
    let buffer = "# my own note\n# error: ours\ntitle: \"Kept\"\n";
    assert_eq!(strip_error_lines(buffer), buffer);
}

#[test]
fn strip_error_lines_drops_a_whole_refusal_block() {
    let buffer = "# error: ours\n# note: ours too\n# hint: ours as well\ntitle: \"Kept\"\n";
    assert_eq!(strip_error_lines(buffer), "title: \"Kept\"\n");
}

// ── parse_buffer: an explicit null parses to the unset value ──

#[test]
fn parse_buffer_null_returns_empty() {
    let values = parse("~");
    assert!(values.is_empty());
}

/// A run that skips verification accepts a buffer whose required slot is still
/// empty, so the editor doesn't re-open the buffer the user just saved. The
/// strict mode refuses the same buffer.
#[test]
fn a_skipped_verification_accepts_an_empty_required_slot() {
    let source = "#f(a: slot(\"a\"), b: slot(\"b\", kind: \"int\"))";
    let specs = mindtape::discover_slots(source);
    let values = BTreeMap::from([("b".to_owned(), RawValue::Int(2))]);

    assert!(super::refuse_buffer(&values, &specs, source, false).is_ok());
    let refused = super::refuse_buffer(&values, &specs, source, true).unwrap_err();
    assert!(
        refused.to_string().contains("missing a required slot: 'a'"),
        "{refused}"
    );
}
