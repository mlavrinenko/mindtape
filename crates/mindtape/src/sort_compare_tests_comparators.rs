//! Direct unit tests of the comparator helpers `compare_field` delegates to:
//! `priority_value` resolving a task's priority through config, and the
//! `cmp_field_value` / `cmp_num_asc` missing-last and tie-fallback rules.
//! Split out of `sort_compare_tests.rs` (file-size gate), which keeps the
//! end-to-end `sort_by_specs` dispatch arms.

#![allow(clippy::unwrap_used)]

use super::tests::{config, task};
use super::{cmp_field_value, cmp_num_asc, priority_value};
use crate::fields::FieldValue;

#[test]
fn priority_value_resolves_through_config() {
    let cfg = config();
    let item = task("a", "wip", 7.0, None);
    assert_eq!(priority_value(&item, &cfg), Some(7.0));
}

#[test]
fn cmp_field_value_orders_missing_after_present() {
    use std::cmp::Ordering;
    // Missing-missing ties; a missing value sorts after any present one — the
    // same missing-last rule `cmp_num_asc`/`cmp_opt_asc` use.
    assert_eq!(
        cmp_field_value(&FieldValue::Missing, &FieldValue::Missing),
        Ordering::Equal
    );
    assert_eq!(
        cmp_field_value(&FieldValue::Missing, &FieldValue::Num(1.0)),
        Ordering::Greater
    );
    assert_eq!(
        cmp_field_value(&FieldValue::Str("a".to_owned()), &FieldValue::Missing),
        Ordering::Less
    );
}

#[test]
fn cmp_field_value_compares_numbers_numerically() {
    use std::cmp::Ordering;
    assert_eq!(
        cmp_field_value(&FieldValue::Num(2.0), &FieldValue::Num(8.0)),
        Ordering::Less
    );
    assert_eq!(
        cmp_field_value(&FieldValue::Num(8.0), &FieldValue::Num(2.0)),
        Ordering::Greater
    );
    // `partial_cmp` yields None for NaN; the fallback ties rather than panics.
    assert_eq!(
        cmp_field_value(&FieldValue::Num(f64::NAN), &FieldValue::Num(1.0)),
        Ordering::Equal
    );
}

#[test]
fn cmp_field_value_ties_mismatched_variants() {
    use std::cmp::Ordering;
    // A Num vs Str pair can't come from `--sort` (both sides resolve from the
    // same field), so the fallback ties rather than panics.
    assert_eq!(
        cmp_field_value(&FieldValue::Num(1.0), &FieldValue::Str("a".to_owned())),
        Ordering::Equal
    );
    assert_eq!(
        cmp_field_value(&FieldValue::Str("a".to_owned()), &FieldValue::Num(1.0)),
        Ordering::Equal
    );
}

#[test]
fn cmp_num_asc_orders_missing_last() {
    use std::cmp::Ordering;
    assert_eq!(cmp_num_asc(None, None), Ordering::Equal);
    assert_eq!(cmp_num_asc(Some(1.0), None), Ordering::Less);
    assert_eq!(cmp_num_asc(None, Some(1.0)), Ordering::Greater);
    assert_eq!(cmp_num_asc(Some(2.0), Some(8.0)), Ordering::Less);
}
