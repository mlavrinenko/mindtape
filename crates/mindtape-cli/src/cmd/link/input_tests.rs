//! Unit tests for the adjacency map parser.

use super::{Entry, parse_adjacency};
use crate::cmd::link::edit::EdgeSpec;

fn edge(kind: &str, target: &str, note: Option<&str>) -> EdgeSpec {
    EdgeSpec {
        kind: kind.to_owned(),
        target: target.to_owned(),
        note: note.map(ToOwned::to_owned),
    }
}

fn parse(text: &str) -> Vec<Entry> {
    parse_adjacency(text).expect("the map must parse")
}

fn error(text: &str) -> String {
    crate::diag::plain(&parse_adjacency(text).expect_err("the map must be rejected"))
}

#[test]
fn the_documented_shape_parses() {
    let entries = parse(
        "seed-content-from-telegram-export:\n  \
         depends-on: [article-content-type, error-record-content-type]\n",
    );
    assert_eq!(entries.len(), 1);
    let entry = entries.first().expect("one entry");
    assert_eq!(entry.stem, "seed-content-from-telegram-export");
    assert_eq!(
        entry.edges,
        vec![
            edge("depends-on", "article-content-type", None),
            edge("depends-on", "error-record-content-type", None),
        ]
    );
}

#[test]
fn a_bare_string_is_a_single_target() {
    let entries = parse("child:\n  parent: umbrella\n");
    assert_eq!(
        entries.first().map(|entry| entry.edges.clone()),
        Some(vec![edge("parent", "umbrella", None)])
    );
}

#[test]
fn a_target_mapping_carries_its_own_note() {
    let entries = parse("a:\n  related:\n    - target: b\n      note: same recital\n");
    assert_eq!(
        entries.first().map(|entry| entry.edges.clone()),
        Some(vec![edge("related", "b", Some("same recital"))])
    );
}

/// YAML 1.2 is a superset of JSON, so the `.json` spelling the docs advertise
/// needs no second parser and no extension sniffing.
#[test]
fn the_json_spelling_parses_through_the_same_reader() {
    let entries = parse(r#"{"a": {"depends-on": ["b", "c"]}}"#);
    assert_eq!(
        entries.first().map(|entry| entry.edges.len()),
        Some(2),
        "JSON must read as YAML"
    );
}

#[test]
fn stems_keep_document_order() {
    let entries = parse("zeta:\n  parent: p\nalpha:\n  parent: p\n");
    let stems: Vec<&str> = entries.iter().map(|entry| entry.stem.as_str()).collect();
    assert_eq!(stems, vec!["zeta", "alpha"]);
}

/// An empty map is an empty run, not a parse failure: `mt link --apply` fed by
/// a pipeline that matched nothing has nothing to do, and nothing to do is not
/// an error.
#[test]
fn an_empty_document_is_an_empty_run() {
    assert!(parse("").is_empty());
    assert!(parse("---\n").is_empty());
}

#[test]
fn a_non_mapping_root_is_rejected_by_shape() {
    let err = error("- a\n- b\n");
    assert_eq!(
        err,
        "error: the adjacency map must be a mapping of stem to kind to targets\nnote: got: a list\n"
    );
}

#[test]
fn a_non_mapping_stem_body_names_the_stem() {
    let err = error("alpha: b\n");
    assert_eq!(
        err,
        "error: expected a mapping of kind to targets: 'alpha'\n"
    );
}

#[test]
fn a_malformed_target_names_the_stem_and_the_kind() {
    let err = error("alpha:\n  depends-on: [3]\n");
    assert_eq!(
        err,
        "error: alpha: a target must be a stem string or a {target, note} mapping under 'depends-on'\n\
         note: got: a number\n"
    );
}

#[test]
fn a_target_mapping_without_a_target_key_is_rejected() {
    let err = error("alpha:\n  related:\n    - note: why\n");
    assert_eq!(
        err,
        "error: alpha: a target mapping needs a target key naming the task stem under 'related'\n"
    );
}

#[test]
fn an_unknown_target_key_is_rejected_rather_than_ignored() {
    let err = error("alpha:\n  related:\n    - target: b\n      reason: why\n");
    assert_eq!(
        err,
        "error: alpha: unknown target key: 'reason' under 'related'\n\
         hint: a target mapping takes target and note\n"
    );
}

#[test]
fn a_non_string_note_is_rejected() {
    let err = error("alpha:\n  related:\n    - target: b\n      note: 7\n");
    assert_eq!(err, "error: alpha: note must be a string under 'related'\n");
}

#[test]
fn malformed_yaml_is_reported_as_such() {
    let err = error("alpha:\n  - [\n");
    assert!(
        err.starts_with("error: can't parse the adjacency map: "),
        "{err}"
    );
}

/// `shape_of` labels what arrived instead of what was asked for; one test per
/// arm the parser can reach, so a mislabeled arm is caught rather than merely
/// executed.
#[test]
fn a_null_target_is_named_null() {
    let err = error("alpha:\n  parent: ~\n");
    assert!(err.contains("note: got: null\n"), "{err}");
}

#[test]
fn a_boolean_target_is_named_a_boolean() {
    let err = error("alpha:\n  parent: true\n");
    assert!(err.contains("note: got: a boolean\n"), "{err}");
}

#[test]
fn a_string_root_is_named_a_string() {
    let err = error("just-a-stem\n");
    assert!(err.contains("note: got: a string\n"), "{err}");
}

#[test]
fn a_tagged_root_is_named_a_tagged_value() {
    let err = error("!custom [a, b]\n");
    assert!(err.contains("note: got: a tagged value\n"), "{err}");
}
