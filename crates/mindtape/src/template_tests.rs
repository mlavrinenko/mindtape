//! Tests for the slot-template engine's discovery half: `discover_slots`
//! schema reading and the `SlotKind`/`RawValue` spellings it builds on. The
//! `wrap` helper stays here so the fill half's `template_fill_tests.rs`
//! sibling reaches it via `super::tests`.
#![allow(clippy::unwrap_used)]

use super::{RawValue, SlotKind, discover_slots};

// Slots live in code context — inside a call's argument list. Wrap a fragment in
// a markup call so its `slot(...)` parses as a function call, not body text.
pub(super) fn wrap(inner: &str) -> String {
    format!("#f({inner})")
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
fn parses_declared_env() {
    let specs = discover_slots(&wrap(r#"x: slot("a", env: "MINDTAPE_PRINCIPAL")"#));
    assert_eq!(
        specs.first().unwrap().env.as_deref(),
        Some("MINDTAPE_PRINCIPAL")
    );
}

#[test]
fn env_defaults_to_implicit_lookup() {
    let specs = discover_slots(&wrap(r#"x: slot("a")"#));
    assert_eq!(specs.first().unwrap().env, None);
}

#[test]
fn discovers_real_kind() {
    let specs = discover_slots(&wrap(r#"x: slot("a", kind: "real")"#));
    assert_eq!(specs.first().unwrap().kind, SlotKind::Real);
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
    assert_eq!(SlotKind::Raw.as_str(), "raw");
    assert_eq!(SlotKind::Choice.as_str(), "choice");
}

#[test]
fn slotkind_parse_raw_returns_raw() {
    // Catches deletion of the "raw" arm in SlotKind::parse.
    assert_eq!(SlotKind::parse("raw"), Some(SlotKind::Raw));
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

/// The kind was spelled `typst` before it was renamed to `raw`. A template
/// still carrying the old spelling must be caught as an unknown kind, not
/// quietly accepted — see `unknown_kind_is_carried_for_fill_to_reject`.
#[test]
fn slotkind_parse_rejects_the_retired_typst_spelling() {
    assert_eq!(SlotKind::parse("typst"), None);
}

/// Discovery cannot fail, so a bad `kind:` rides along on the spec. Without
/// this the spelling is lost and the slot reads as a plain `string`.
#[test]
fn unknown_kind_is_carried_for_fill_to_reject() {
    let specs = discover_slots(r#"#f(body: slot("body", kind: "typst"))"#);
    assert_eq!(specs.len(), 1);
    assert_eq!(
        specs.first().unwrap().unknown_kind.as_deref(),
        Some("typst")
    );
}

/// A slot with no `kind:` at all is a `string` by design, and must not be
/// confused with one whose kind failed to parse.
#[test]
fn absent_kind_is_not_an_unknown_kind() {
    let specs = discover_slots(r#"#f(title: slot("title"))"#);
    assert_eq!(specs.len(), 1);
    assert_eq!(specs.first().unwrap().kind, SlotKind::String);
    assert_eq!(specs.first().unwrap().unknown_kind, None);
}
