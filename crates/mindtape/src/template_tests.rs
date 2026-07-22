//! Tests for the slot-template engine.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{FillError, RawValue, SlotKind, discover_slots, fill};

fn values(pairs: &[(&str, RawValue)]) -> BTreeMap<String, RawValue> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect()
}

// Slots live in code context — inside a call's argument list. Wrap a fragment in
// a markup call so its `slot(...)` parses as a function call, not body text.
fn wrap(inner: &str) -> String {
    format!("#f({inner})")
}

fn fill_wrapped(inner: &str, pairs: &[(&str, RawValue)]) -> Result<String, FillError> {
    fill(&wrap(inner), &values(pairs))
}

#[test]
fn discovers_name_kind_and_required() {
    let specs = discover_slots(&wrap(r#"title: slot("title", kind: "string")"#));
    assert_eq!(specs.len(), 1);
    assert_eq!(specs.first().unwrap().name, "title");
    assert_eq!(specs.first().unwrap().kind, SlotKind::String);
    assert!(specs.first().unwrap().required);
}

#[test]
fn defaults_to_string_kind_and_required() {
    let specs = discover_slots(&wrap(r#"x: slot("a")"#));
    assert_eq!(specs.first().unwrap().kind, SlotKind::String);
    assert!(specs.first().unwrap().required);
    assert!(specs.first().unwrap().default.is_none());
}

#[test]
fn reads_default_source_verbatim() {
    let specs = discover_slots(&wrap(r#"x: slot("a", kind: "list", default: ())"#));
    assert_eq!(specs.first().unwrap().default.as_deref(), Some("()"));
}

#[test]
fn default_none_is_no_default() {
    let specs = discover_slots(&wrap(r#"x: slot("a", default: none)"#));
    assert!(specs.first().unwrap().default.is_none());
}

#[test]
fn parses_choices() {
    let specs = discover_slots(&wrap(
        r#"x: slot("a", kind: "choice", choices: ("bug", "feat"))"#,
    ));
    assert_eq!(
        specs.first().unwrap().choices.as_deref(),
        Some(["bug".to_owned(), "feat".to_owned()].as_slice())
    );
}

#[test]
fn fills_string_with_quoting_and_escape() {
    let out = fill_wrapped(
        r#"title: slot("title")"#,
        &[("title", RawValue::Str("say \"hi\"".to_owned()))],
    )
    .unwrap();
    assert_eq!(out, wrap(r#"title: "say \"hi\"""#));
}

#[test]
fn fills_int_and_float() {
    let out = fill_wrapped(
        r#"a: slot("a", kind: "int"), b: slot("b", kind: "float")"#,
        &[("a", RawValue::Int(8)), ("b", RawValue::Float(0.7))],
    )
    .unwrap();
    assert_eq!(out, wrap(r"a: 8, b: 0.7"));
}

#[test]
fn float_keeps_decimal_for_integral() {
    let out = fill_wrapped(
        r#"b: slot("b", kind: "float")"#,
        &[("b", RawValue::Float(8.0))],
    )
    .unwrap();
    assert_eq!(out, wrap("b: 8.0"));
}

#[test]
fn discovers_real_kind() {
    let specs = discover_slots(&wrap(r#"x: slot("a", kind: "real")"#));
    assert_eq!(specs.first().unwrap().kind, SlotKind::Real);
}

#[test]
fn fills_real_without_truncation() {
    let out = fill_wrapped(
        r#"priority: slot("priority", kind: "real")"#,
        &[("priority", RawValue::Float(2.5))],
    )
    .unwrap();
    assert_eq!(out, wrap("priority: 2.5"));
}

#[test]
fn fills_real_from_string_value_without_truncation() {
    // `--set priority=2.5` always arrives as a string; a real-kind slot must
    // coerce it to a float, not fail the way an `int`-kind slot would.
    let out = fill_wrapped(
        r#"priority: slot("priority", kind: "real")"#,
        &[("priority", RawValue::Str("2.5".to_owned()))],
    )
    .unwrap();
    assert_eq!(out, wrap("priority: 2.5"));
}

#[test]
fn real_keeps_decimal_for_integral() {
    let out = fill_wrapped(
        r#"priority: slot("priority", kind: "real")"#,
        &[("priority", RawValue::Float(8.0))],
    )
    .unwrap();
    assert_eq!(out, wrap("priority: 8.0"));
}

#[test]
fn fills_date_as_on_call() {
    let out = fill_wrapped(
        r#"due: slot("due", kind: "date")"#,
        &[("due", RawValue::Str("2026-07-01".to_owned()))],
    )
    .unwrap();
    assert_eq!(out, wrap("due: on(2026, 7, 1)"));
}

#[test]
fn bad_date_errors() {
    let err = fill_wrapped(
        r#"due: slot("due", kind: "date")"#,
        &[("due", RawValue::Str("nope".to_owned()))],
    )
    .unwrap_err();
    assert!(matches!(err, FillError::BadDate { .. }));
}

#[test]
fn fills_list_with_trailing_comma() {
    let out = fill_wrapped(
        r#"tags: slot("tags", kind: "list")"#,
        &[("tags", RawValue::List(vec!["a".to_owned(), "b".to_owned()]))],
    )
    .unwrap();
    assert_eq!(out, wrap(r#"tags: ("a", "b",)"#));
}

#[test]
fn empty_list_renders_unit() {
    let out = fill_wrapped(
        r#"tags: slot("tags", kind: "list")"#,
        &[("tags", RawValue::List(vec![]))],
    )
    .unwrap();
    assert_eq!(out, wrap("tags: ()"));
}

#[test]
fn bare_string_into_list_slot_errors_with_brackets_hint() {
    let err = fill_wrapped(
        r#"tags: slot("tags", kind: "list")"#,
        &[("tags", RawValue::Str("a,b,c".to_owned()))],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        FillError::Type {
            want: SlotKind::List,
            ..
        }
    ));
    assert!(err.to_string().contains("tags[]="), "{err}");
}

#[test]
fn choice_outside_set_errors() {
    let err = fill_wrapped(
        r#"k: slot("k", kind: "choice", choices: ("bug", "feat"))"#,
        &[("k", RawValue::Str("spike".to_owned()))],
    )
    .unwrap_err();
    assert!(matches!(err, FillError::BadChoice { .. }));
}

#[test]
fn unfilled_uses_default_source() {
    let out = fill_wrapped(r#"a: slot("a", kind: "int", default: 5)"#, &[]).unwrap();
    assert_eq!(out, wrap("a: 5"));
}

#[test]
fn unfilled_required_without_default_errors() {
    let err = fill_wrapped(r#"a: slot("a"), b: slot("b")"#, &[]).unwrap_err();
    match err {
        FillError::Missing(names) => assert_eq!(names, vec!["a", "b"]),
        other => panic!("expected Missing, got {other:?}"),
    }
}

#[test]
fn optional_unfilled_renders_none() {
    let out = fill_wrapped(r#"a: slot("a", required: false)"#, &[]).unwrap();
    assert_eq!(out, wrap("a: none"));
}

#[test]
fn type_mismatch_errors() {
    let err = fill_wrapped(
        r#"a: slot("a", kind: "int")"#,
        &[("a", RawValue::Bool(true))],
    )
    .unwrap_err();
    assert!(matches!(err, FillError::Type { .. }));
}

#[test]
fn markup_text_slot_drops_hash() {
    let src = "== Summary\n\n#slot(\"summary\", kind: \"text\")\n";
    let out = fill(
        src,
        &values(&[("summary", RawValue::Str("Body line.".to_owned()))]),
    )
    .unwrap();
    assert_eq!(out, "== Summary\n\nBody line.\n");
}

#[test]
fn text_slot_default_lands_as_body_markup() {
    let src = "== Scope\n\n#slot(\"scope\", kind: \"text\", default: \"- TODO\")\n";
    let out = fill(src, &BTreeMap::new()).unwrap();
    assert_eq!(out, "== Scope\n\n- TODO\n");
}

#[test]
fn multiple_slots_in_nested_calls() {
    let out = fill_wrapped(
        r#"priority: ice(impact: slot("i", kind: "int"), ease: slot("e", kind: "int"))"#,
        &[("i", RawValue::Int(8)), ("e", RawValue::Int(6))],
    )
    .unwrap();
    assert_eq!(out, wrap("priority: ice(impact: 8, ease: 6)"));
}

#[test]
fn slotkind_as_str_returns_correct_spelling() {
    // Catches mutants that replace the as_str match with "" or "xyzzy".
    assert_eq!(SlotKind::String.as_str(), "string");
    assert_eq!(SlotKind::Int.as_str(), "int");
    assert_eq!(SlotKind::Float.as_str(), "float");
    assert_eq!(SlotKind::Real.as_str(), "real");
    assert_eq!(SlotKind::Bool.as_str(), "bool");
    assert_eq!(SlotKind::Date.as_str(), "date");
    assert_eq!(SlotKind::List.as_str(), "list");
    assert_eq!(SlotKind::Text.as_str(), "text");
    assert_eq!(SlotKind::Choice.as_str(), "choice");
}

#[test]
fn slotkind_parse_string_returns_string() {
    // Catches deletion of the "string" arm in SlotKind::parse.
    assert_eq!(SlotKind::parse("string"), Some(SlotKind::String));
}

#[test]
fn slotkind_parse_bool_returns_bool() {
    // Catches deletion of the "bool" arm in SlotKind::parse.
    assert_eq!(SlotKind::parse("bool"), Some(SlotKind::Bool));
}

#[test]
fn rawvalue_type_name_returns_correct_name() {
    // Catches mutants that replace the type_name match with "" or "xyzzy".
    assert_eq!(RawValue::Str("x".into()).type_name(), "string");
    assert_eq!(RawValue::Int(0).type_name(), "int");
    assert_eq!(RawValue::Float(0.0).type_name(), "float");
    assert_eq!(RawValue::Bool(true).type_name(), "bool");
    assert_eq!(RawValue::List(vec![]).type_name(), "list");
}

#[test]
fn fills_int_from_string_value() {
    // Catches deletion of the RawValue::Str arm in coerce_int.
    let out = fill_wrapped(
        r#"a: slot("a", kind: "int")"#,
        &[("a", RawValue::Str("42".to_owned()))],
    )
    .unwrap();
    assert_eq!(out, wrap("a: 42"));
}

#[test]
fn fills_float_from_int_value() {
    // Catches deletion of the RawValue::Int arm in coerce_float.
    let out = fill_wrapped(r#"b: slot("b", kind: "float")"#, &[("b", RawValue::Int(7))]).unwrap();
    assert_eq!(out, wrap("b: 7.0"));
}

#[test]
fn fills_bool_false_from_string_false() {
    // Catches mutant replacing coerce_bool body with Ok(true).
    let out = fill_wrapped(
        r#"flag: slot("flag", kind: "bool")"#,
        &[("flag", RawValue::Str("false".to_owned()))],
    )
    .unwrap();
    assert_eq!(out, wrap("flag: false"));
}

#[test]
fn fills_bool_true_from_string_true() {
    // Catches mutant replacing coerce_bool body with Ok(false) AND deletion of
    // the RawValue::Str arm (both cause the test to fail).
    let out = fill_wrapped(
        r#"flag: slot("flag", kind: "bool")"#,
        &[("flag", RawValue::Str("true".to_owned()))],
    )
    .unwrap();
    assert_eq!(out, wrap("flag: true"));
}

#[test]
fn fills_bool_from_bool_value() {
    // Catches deletion of the RawValue::Bool(flag) arm in coerce_bool.
    let out = fill_wrapped(
        r#"flag: slot("flag", kind: "bool")"#,
        &[("flag", RawValue::Bool(true))],
    )
    .unwrap();
    assert_eq!(out, wrap("flag: true"));
}
