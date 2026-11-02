#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{matches_external, ref_text, split_refs, value_stem};

#[test]
fn a_value_splits_on_commas_and_whitespace() {
    assert_eq!(split_refs("redesign r#45782"), ["redesign", "r#45782"]);
    assert_eq!(split_refs("redesign, r#45782"), ["redesign", "r#45782"]);
    assert_eq!(split_refs("a,, b"), ["a", "b"]);
    assert_eq!(split_refs("  a\t b  "), ["a", "b"]);
}

#[test]
fn a_quoted_ref_keeps_a_separator_and_a_backslash_escapes_one() {
    assert_eq!(
        split_refs("\"redesign one\", r#45782"),
        ["redesign one", "r#45782"]
    );
    assert_eq!(split_refs("redesign\\ one"), ["redesign one"]);
    assert_eq!(split_refs("a\\,b"), ["a,b"]);
    assert_eq!(split_refs("C:\\project"), ["C:\\project"]);
}

#[test]
fn a_path_names_the_stem_of_its_file() {
    assert_eq!(value_stem("foo.task-261023"), "foo.task-261023");
    assert_eq!(value_stem("tasks/foo.task-261023.typ"), "foo.task-261023");
    assert_eq!(value_stem("foo.task-261023.typ"), "foo.task-261023");
}

#[test]
fn an_external_pattern_matches_the_whole_token() {
    let matchers = super::external_matchers(&["^r#[0-9]+$".to_owned()]);
    assert!(matches_external("r#45782", &matchers));
    assert!(!matches_external("redesign", &matchers));
}

#[test]
fn a_written_ref_round_trips() {
    assert_eq!(ref_text("parser"), "parser");
    assert_eq!(ref_text("write docs"), "\"write docs\"");
    assert_eq!(ref_text("a,b"), "\"a,b\"");
    for value in ["parser", "write docs", "a,b", "a\"b", "a\\b"] {
        assert_eq!(split_refs(&ref_text(value)), [value], "{value:?}");
    }
}
