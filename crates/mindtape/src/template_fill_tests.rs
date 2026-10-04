//! Tests for the slot-template engine's fill half — value coercion against a
//! slot's kind, per-kind rendering, defaults, the markup-splice pipeline, and
//! the `FillError` display arms — split out of `template_tests.rs` along the
//! same discover/fill seam `template.rs` documents for its two source files.
//! Reaches the shared `wrap` helper from the sibling discovery module via
//! `super::tests`.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::tests::wrap;
use super::{FillError, RawValue, SlotKind, fill};

fn values(pairs: &[(&str, RawValue)]) -> BTreeMap<String, RawValue> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect()
}

fn fill_wrapped(inner: &str, pairs: &[(&str, RawValue)]) -> Result<String, FillError> {
    fill(&wrap(inner), &values(pairs))
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
    assert!(err.to_string().contains("YYYY-MM-DD"), "{err}");
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
    assert!(err.to_string().contains("tags+="), "{err}");
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
    let msg = err.to_string();
    assert!(
        msg.contains("\"a\"") && msg.contains("int") && msg.contains("bool"),
        "{msg}"
    );
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

/// Round trip: a code span in, a code span out, no backslashes — the opt-out
/// `text` slots deferred (`add-a-typst-slot-kind-for-pre-formatted-markup.typ`).
#[test]
fn markup_raw_slot_splices_without_escaping() {
    let src = "== Example\n\n#slot(\"body\", kind: \"raw\")\n";
    let out = fill(
        src,
        &values(&[(
            "body",
            RawValue::Str("See `mt add` for details.".to_owned()),
        )]),
    )
    .unwrap();
    assert_eq!(out, "== Example\n\nSee `mt add` for details.\n");
    assert!(!out.contains('\\'), "raw slot must not escape: {out}");
}

#[test]
fn raw_slot_default_lands_as_body_markup() {
    let src = "== Scope\n\n#slot(\"scope\", kind: \"raw\", default: \"- `TODO`\")\n";
    let out = fill(src, &BTreeMap::new()).unwrap();
    assert_eq!(out, "== Scope\n\n- `TODO`\n");
}

#[test]
fn raw_slot_unparseable_markup_names_the_slot() {
    let err = fill_wrapped(
        r#"body: slot("body", kind: "raw")"#,
        &[("body", RawValue::Str("#unclosed[".to_owned()))],
    )
    .unwrap_err();
    assert!(
        matches!(&err, FillError::BadMarkup { slot, .. } if slot == "body"),
        "{err:?}"
    );
    assert!(err.to_string().contains("\"body\""), "{err}");
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

#[test]
fn fills_string_kind_from_scalar_values() {
    // Catches deletion of the Int/Float/Bool arms in coerce_str: a `--set`
    // value always arrives as a string, but a `-f` JSON/YAML file can hand any
    // scalar to a string-kind slot.
    let out = fill_wrapped(
        r#"a: slot("a"), b: slot("b"), c: slot("c")"#,
        &[
            ("a", RawValue::Int(7)),
            ("b", RawValue::Float(2.5)),
            ("c", RawValue::Bool(true)),
        ],
    )
    .unwrap();
    assert_eq!(out, wrap(r#"a: "7", b: "2.5", c: "true""#));
}

#[test]
fn list_value_into_string_kind_errors() {
    let err = fill_wrapped(
        r#"a: slot("a")"#,
        &[("a", RawValue::List(vec!["x".to_owned()]))],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        FillError::Type {
            want: SlotKind::String,
            got: "list",
            ..
        }
    ));
}

#[test]
fn non_numeric_value_into_float_slot_errors() {
    let err = fill_wrapped(
        r#"b: slot("b", kind: "float")"#,
        &[("b", RawValue::Bool(true))],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        FillError::Type {
            want: SlotKind::Float,
            got: "bool",
            ..
        }
    ));
}

#[test]
fn non_bool_value_into_bool_slot_errors() {
    let err = fill_wrapped(
        r#"flag: slot("flag", kind: "bool")"#,
        &[("flag", RawValue::Int(3))],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        FillError::Type {
            want: SlotKind::Bool,
            got: "int",
            ..
        }
    ));
}

#[test]
fn edit_failure_displays_rewrite_message() {
    // `FillError::Edit` is built only from `apply`'s changeset validation,
    // which no real `fill` call can trip (slot ranges come from the parser and
    // never overlap), so exercise the display arm directly.
    let err = FillError::Edit("boom".to_owned());
    assert_eq!(err.to_string(), "rewrite failed: boom");
}

/// The migration path off the retired `typst` spelling. Before `unknown_kind`
/// existed this filled as a `string` — the value quoted and escaped, a wrong
/// task written with exit 0 and no diagnostic anywhere.
#[test]
fn retired_typst_kind_fails_naming_the_slot_and_the_valid_kinds() {
    let err = fill_wrapped(
        r#"body: slot("body", kind: "typst")"#,
        &[("body", RawValue::Str("See `mt add`.".to_owned()))],
    )
    .unwrap_err();
    assert!(
        matches!(&err, FillError::UnknownKind { slot, kind } if slot == "body" && kind == "typst"),
        "{err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains("raw"), "{msg}");
}

/// An unknown kind is rejected even when no value is supplied for the slot:
/// the default path must not sneak a mis-declared slot past the check.
#[test]
fn unknown_kind_fails_even_when_the_slot_is_unfilled() {
    let err =
        fill_wrapped(r#"body: slot("body", kind: "markdown", default: "x")"#, &[]).unwrap_err();
    assert!(matches!(&err, FillError::UnknownKind { .. }), "{err:?}");
}

#[test]
fn an_example_is_never_filed() {
    let source = "#f(t: slot(\"t\", example: \"Sample title\"))";
    let values = BTreeMap::from([("t".to_owned(), RawValue::Str("Real".to_owned()))]);
    let filled = fill(source, &values).unwrap();
    assert!(!filled.contains("Sample"), "{filled}");
    assert_eq!(
        fill(source, &BTreeMap::new()),
        Err(FillError::Missing(vec!["t".to_owned()])),
        "an example does not stand in for a missing value"
    );
}

#[test]
fn a_raw_block_default_fills_its_dedented_text() {
    let source = "== P\n#slot(\"p\", kind: \"raw\", default: ```typ\n  + one\n  + two\n  ```)\n";
    let filled = fill(source, &BTreeMap::new()).unwrap();
    assert_eq!(filled, "== P\n+ one\n+ two\n");
}

#[test]
fn fill_examples_takes_the_example_then_the_default() {
    let source = "#f(a: slot(\"a\", default: 1, example: 2), b: slot(\"b\", default: 3))\n\
                  #slot(\"c\", kind: \"raw\", example: ```typ\n+ one\n```)\n";
    let filled = super::fill_examples(source).unwrap();
    assert_eq!(filled, "#f(a: 2, b: 3)\n+ one\n");
}

#[test]
fn fill_examples_names_each_slot_it_cannot_fill() {
    let source = "#f(a: slot(\"a\"), b: slot(\"b\", required: false))";
    assert_eq!(
        super::fill_examples(source),
        Err(FillError::Missing(vec!["a".to_owned(), "b".to_owned()]))
    );
}
