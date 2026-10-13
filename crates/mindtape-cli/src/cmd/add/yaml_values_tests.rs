//! Tests for the shared YAML mapping -> `RawValue` converter.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::RawValue;

use super::parse_mapping;

fn parse(
    text: &str,
) -> Result<std::collections::BTreeMap<String, Option<RawValue>>, crate::diag::Diag> {
    parse_mapping(text, Some("v.yaml"))
}

/// The refusal as stderr prints it.
fn refusal(text: &str) -> String {
    crate::diag::plain(&parse(text).unwrap_err())
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
fn non_mapping_root_names_the_source() {
    assert_eq!(
        refusal("- a\n- b\n"),
        "error: values must be a YAML mapping: 'v.yaml'\n"
    );
}

#[test]
fn malformed_yaml_names_the_source() {
    let err = refusal("key: [unterminated\n");
    assert!(
        err.starts_with("error: can't parse YAML: 'v.yaml': "),
        "{err}"
    );
}

#[test]
fn the_editor_buffer_has_no_source_to_name() {
    let err = parse_mapping("- a\n", None).unwrap_err();
    crate::diag::assert_plain(&err, "error: values must be a YAML mapping\n");
}

#[test]
fn multi_document_stream_is_rejected() {
    let err = refusal("---\na: 1\n---\nb: 2\n");
    assert!(
        err.starts_with("error: can't parse YAML: 'v.yaml': "),
        "{err}"
    );
}

#[test]
fn non_string_key_errors() {
    assert_eq!(
        refusal("1: a\n"),
        "error: YAML keys must be strings: 'v.yaml'\n"
    );
}

#[test]
fn nested_mapping_value_errors() {
    assert_eq!(
        refusal("nested:\n  a: 1\n"),
        "error: nested mappings are unsupported: 'nested'\n"
    );
}

#[test]
fn non_string_list_item_errors() {
    assert_eq!(
        refusal("tags:\n  - 1\n"),
        "error: list items must be strings: 'tags'\n"
    );
}
