//! Tests for the shared YAML mapping -> `RawValue` converter.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::RawValue;

use super::parse_mapping;

fn parse(text: &str) -> anyhow::Result<std::collections::BTreeMap<String, Option<RawValue>>> {
    parse_mapping(
        text,
        || "syntax context".to_owned(),
        || "non-mapping root".to_owned(),
    )
}

#[test]
fn parses_scalar_shapes() {
    let values = parse("title: Fix\ncount: 3\nweight: 1.5\nactive: true\n").unwrap();
    assert_eq!(values["title"], Some(RawValue::Str("Fix".to_owned())));
    assert_eq!(values["count"], Some(RawValue::Int(3)));
    assert_eq!(values["weight"], Some(RawValue::Float(1.5)));
    assert_eq!(values["active"], Some(RawValue::Bool(true)));
}

#[test]
fn parses_empty_string_as_a_real_value() {
    // Unlike the editor caller's post-filter, the shared converter never
    // drops "" — that decision belongs to each caller.
    let values = parse("note: \"\"\n").unwrap();
    assert_eq!(values["note"], Some(RawValue::Str(String::new())));
}

#[test]
fn parses_sequence_of_strings() {
    let values = parse("tags:\n  - a\n  - b\n").unwrap();
    assert_eq!(
        values["tags"],
        Some(RawValue::List(vec!["a".to_owned(), "b".to_owned()]))
    );
}

#[test]
fn null_value_keeps_its_key_mapped_to_none() {
    let values = parse("priority: ~\n").unwrap();
    assert!(values.contains_key("priority"), "key must survive as None");
    assert_eq!(values["priority"], None);
}

#[test]
fn null_root_is_an_empty_mapping() {
    let values = parse("~").unwrap();
    assert!(values.is_empty());
}

#[test]
fn non_mapping_root_uses_the_callers_message() {
    let err = parse("- a\n- b\n").unwrap_err().to_string();
    assert_eq!(err, "non-mapping root");
}

#[test]
fn malformed_yaml_uses_the_callers_syntax_context() {
    let err = parse("key: [unterminated\n").unwrap_err().to_string();
    assert_eq!(err, "syntax context");
}

#[test]
fn multi_document_stream_is_rejected() {
    // The underlying parser already refuses more than one document; the
    // caller's syntax context wraps that failure.
    let err = parse("---\na: 1\n---\nb: 2\n").unwrap_err().to_string();
    assert_eq!(err, "syntax context");
}

#[test]
fn non_string_key_errors() {
    let err = parse("1: a\n").unwrap_err().to_string();
    assert!(err.contains("keys must be strings"), "{err}");
}

#[test]
fn nested_mapping_value_errors() {
    let err = parse("nested:\n  a: 1\n").unwrap_err().to_string();
    assert!(err.contains("value for \"nested\""), "{err}");
}

#[test]
fn non_string_list_item_errors() {
    let err = parse("tags:\n  - 1\n").unwrap_err().to_string();
    assert!(err.contains("value for \"tags\""), "{err}");
}
