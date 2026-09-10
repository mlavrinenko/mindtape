//! Regression tests for cargo-mutants survivors in `ls_rules.rs`, plus the
//! `--columns` suggestion wiring. Split out of `ls_rules_tests.rs` to keep both
//! files under the size gate: the cascade's own behaviour is covered there and
//! `Applied`'s trailer in `ls_rules_tests_applied.rs`. Shares
//! `ls_rules_tests.rs`'s `args`/`project` fixtures.
#![allow(clippy::unwrap_used)]

use mindtape::parse_config;

use crate::ls_rules::tests::{args, project, resolve, resolve_bare};

// Pins delete rule from Self in Applied::match_all.
//   PROVABLY EQUIVALENT: ..Self::default() already supplies rule: None.
//   No test needed.

#[test]
fn explicit_filter_sets_filter_value_in_applied() {
    // Kills mutant at delete filter_value from Applied in
    // apply_explicit_filter.
    // Real:   applied.filter_value == expr
    // Mutant: applied.filter_value == "priority > 1"  (the rule's, untouched)
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
filter = "priority > 1"
"#,
    );
    let (_bundle, applied) = resolve(&project, &args(Some("priority > 9"))).unwrap();
    assert_eq!(applied.filter_value, "priority > 9");
}

#[test]
fn import_sort_from_empty_sort_view_preserves_earlier_sort() {
    // Kills mutant at !view.sort.is_empty() → true.
    // Real: guard fails → no overwrite → sort stays ["due:asc"].
    // Mutant: guard always true → overwrites with [] → sort empty.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.a]
sort = ["due:asc"]

[views.b]
filter = "done"

[[ls.rules]]
import-views = ["a", "b(sort)"]
"#,
    );
    let (bundle, _applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.sort, vec!["due:asc".to_owned()]);
}

#[test]
fn import_columns_from_empty_columns_view_preserves_earlier_columns() {
    // Kills mutant at !view.columns.is_empty() → true.
    // Real: guard fails → no overwrite → columns stay ["status", "title"].
    // Mutant: guard always true → overwrites with [] → columns empty.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.a]
columns = ["status", "title"]

[views.b]
filter = "done"

[[ls.rules]]
import-views = ["a", "b(columns)"]
"#,
    );
    let (bundle, _applied) = resolve_bare(&project).unwrap();
    assert_eq!(
        bundle.columns,
        vec!["status".to_owned(), "title".to_owned()]
    );
}

// --- suggest wiring for a `--columns` spec
// (`truth/tasks/adopt-nearest-match-suggestions-at-the-remaining-unknown-name-errors.typ`) --

#[test]
fn resolve_columns_near_miss_offers_a_suggestion() {
    let config = parse_config("").unwrap();
    let err = super::resolve_columns("staatus", &config).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("expected one of"), "{msg}");
    assert!(msg.contains("did you mean: status?"), "{msg}");
}

#[test]
fn resolve_columns_far_miss_omits_the_suggestion() {
    let config = parse_config("").unwrap();
    let err = super::resolve_columns("zzzzzzzz", &config).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("expected one of"), "{msg}");
    assert!(!msg.contains("did you mean"), "{msg}");
}
