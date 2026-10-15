//! Tests for the slot-template engine's fill errors: what a user reads for each
//! `FillError`, and the rejection of an unknown slot kind. Split out of
//! `template/tests_fill.rs`, which keeps the successful fills.
#![allow(clippy::unwrap_used)]

use super::tests_fill::fill_wrapped;
use super::{FillError, RawValue, SlotKind};

#[test]
fn edit_failure_displays_rewrite_message() {
    // `FillError::Edit` is built only from `apply`'s changeset validation,
    // which no real `fill` call can trip (slot ranges come from the parser and
    // never overlap), so exercise the display arm directly.
    let err = FillError::Edit("boom".to_owned());
    assert_eq!(err.to_string(), "rewrite failed: boom");
}

/// The text a user reads for every `FillError`: each names the slot and, where
/// one exists, the way out.
#[test]
fn every_error_displays_its_slot_and_remedy() {
    let type_err = |want| FillError::Type {
        slot: "n".to_owned(),
        want,
        got: "string",
    };
    let cases = [
        (
            FillError::Missing(vec!["a".to_owned(), "b".to_owned()]),
            "missing required slots: a, b",
        ),
        (
            type_err(SlotKind::Int),
            "slot \"n\" expects int, got string",
        ),
        (type_err(SlotKind::MultiChoice), "--set n+=value"),
        (
            FillError::BadChoice {
                slot: "k".to_owned(),
                value: "x".to_owned(),
                choices: vec!["bug".to_owned(), "feature".to_owned()],
            },
            "slot \"k\" value \"x\" is not one of: bug, feature",
        ),
        (
            FillError::BadDate {
                slot: "d".to_owned(),
                value: "soon".to_owned(),
            },
            "slot \"d\" value \"soon\" is not a YYYY-MM-DD date",
        ),
        (
            FillError::BadMarkup {
                slot: "m".to_owned(),
                message: "unclosed".to_owned(),
            },
            "slot \"m\" is not valid Typst markup: unclosed",
        ),
    ];
    for (err, expected) in cases {
        assert!(err.to_string().contains(expected), "{err} lacks {expected}");
    }
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
