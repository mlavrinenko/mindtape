//! Tests for [`super::FillMode`]: the fill that leaves an unfilled required
//! slot as its `slot(...)` call, which is what a run that skips verification
//! files. They live apart from `tests_fill.rs` along that seam, and because
//! that file sits at its line cap.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::tests_fill::values;
use super::{FillError, FillMode, RawValue, SlotKind, fill, fill_with};

/// `FillMode::KeepMissing` leaves a required slot with no value and no default
/// as its `slot(...)` call, so a task that skips verification files
/// incomplete. Every other slot still fills, and `fill` keeps refusing.
#[test]
fn keep_missing_leaves_the_slot_call_in_place() {
    let source = "#f(a: slot(\"a\"), b: slot(\"b\", default: 2), c: slot(\"c\", required: false))";
    let filled = fill_with(source, &BTreeMap::new(), FillMode::KeepMissing).unwrap();

    assert_eq!(filled, "#f(a: slot(\"a\"), b: 2, c: none)");
    assert_eq!(
        fill(source, &BTreeMap::new()),
        Err(FillError::Missing(vec!["a".to_owned()])),
        "the strict mode still refuses"
    );
}

/// A value the caller supplies still fills in `KeepMissing`, and a value the
/// slot's kind refuses is still an error: the mode relaxes the missing-slot
/// refusal alone.
#[test]
fn keep_missing_still_coerces_a_supplied_value() {
    let source = "#f(a: slot(\"a\"), n: slot(\"n\", kind: \"int\"))";
    let filled = fill_with(
        source,
        &values(&[("n", RawValue::Str("3".to_owned()))]),
        FillMode::KeepMissing,
    )
    .unwrap();
    assert_eq!(filled, "#f(a: slot(\"a\"), n: 3)");

    assert_eq!(
        fill_with(
            source,
            &values(&[("n", RawValue::Str("three".to_owned()))]),
            FillMode::KeepMissing,
        ),
        Err(FillError::Type {
            slot: "n".to_owned(),
            want: SlotKind::Int,
            got: "string",
        })
    );
}
