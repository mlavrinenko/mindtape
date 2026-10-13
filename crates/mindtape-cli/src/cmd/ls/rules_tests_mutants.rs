//! Regression tests for cargo-mutants survivors in `cmd/ls/rules.rs`, plus the
//! `--columns` suggestion wiring and the documented `@default` escape. Split
//! out of `cmd/ls/rules_tests.rs` to keep both files under the size gate: the
//! cascade's own behaviour is covered there and `Applied`'s trailer in
//! `cmd/ls/rules_applied_ls_rules_tests_applied.rs`. Shares `cmd/ls/rules_tests.rs`'s `args`/`project`
//! fixtures.
#![allow(clippy::unwrap_used)]

use mindtape::parse_config;
use mindtape::test_support::tempdir;

use crate::cmd::ls::rules::tests::{args, project, resolve, resolve_bare};

// Pins delete rule from Self in Applied::match_all.
//   PROVABLY EQUIVALENT: ..Self::default() already supplies rule: None.
//   No test needed.

#[test]
fn explicit_filter_sets_filter_value_in_applied() {
    // Kills mutant at delete filter_value from Applied in
    // apply_explicit_filter.
    // Real:   applied.filter_value == expr
    // Mutant: applied.filter_value == "priority > 1"  (the rule's, untouched)
    let dir = tempdir();
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
    let dir = tempdir();
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
    let dir = tempdir();
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

// --- view_bundle: a named view's Applied facets
// (`view_bundle` in `cmd/ls/rules.rs`).

#[test]
fn default_view_bundle_carries_the_views_filter_value() {
    // Kills mutant at delete filter_value from Applied in view_bundle.
    // Real:   applied.filter_value == the view's filter.
    // Mutant: Applied::default()'s empty value — the JSON envelope's
    //         filter.value comes back blank.
    let dir = tempdir();
    let project = project(dir.path(), "[views.default]\nfilter = \"terminal\"\n");
    let (_bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(applied.filter_value, "terminal");
}

#[test]
fn default_view_bundle_attributes_columns_to_the_view() {
    // Kills mutant at delete columns_from from Applied in view_bundle.
    // Real:   columns_from names the view that set the columns.
    // Mutant: the empty default, losing the attribution.
    let dir = tempdir();
    let project = project(
        dir.path(),
        "[views.default]\ncolumns = [\"status\", \"title\"]\n",
    );
    let (_bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(applied.columns_from, "default");
}

#[test]
fn explicit_filter_keeps_the_default_views_columns_value() {
    // Kills mutant at delete columns_value from Applied in view_bundle.
    // Real:   bundle_of re-reads the view's columns, so they survive the filter.
    // Mutant: the empty default, so narrowing drops the presentation.
    let dir = tempdir();
    let project = project(
        dir.path(),
        "[views.default]\ncolumns = [\"status\", \"title\"]\n",
    );
    let (bundle, applied) = resolve(&project, &args(Some("priority > 9"))).unwrap();
    assert_eq!(
        bundle.columns,
        vec!["status".to_owned(), "title".to_owned()]
    );
    assert_eq!(applied.columns_from, "default");
}

// --- suggest wiring for a `--columns` spec
// (`truth/tasks/adopt-nearest-match-suggestions-at-the-remaining-unknown-name-errors.typ`) --

#[test]
fn resolve_columns_near_miss_offers_a_suggestion() {
    let config = parse_config("").unwrap();
    let err = super::resolve_columns("staatus", &config).unwrap_err();
    let msg = crate::diag::plain(&err);
    assert_eq!(
        msg,
        "error: unknown column: 'staatus'\nhint: did you mean status?\n"
    );
}

#[test]
fn resolve_columns_far_miss_omits_the_suggestion() {
    let config = parse_config("").unwrap();
    let err = super::resolve_columns("zzzzzzzz", &config).unwrap_err();
    let msg = crate::diag::plain(&err);
    assert!(
        msg.starts_with("error: unknown column: 'zzzzzzzz'\nhint: columns are "),
        "{msg}"
    );
    assert!(!msg.contains("did you mean"), "{msg}");
}

/// The far-miss hint lists the same columns `--columns` accepts.
#[test]
fn the_column_hint_lists_every_column() {
    assert_eq!(
        super::COLUMN_NAMES,
        format!("columns are {}", mindtape::LS_COLUMNS.join(", "))
    );
}

// --- the escape hatch the CLI docs promise
// (`truth/tasks/ls-rules-lose-their-sort-and-columns-facets-under-an-explicit-filter.typ`) --

#[test]
fn splicing_the_default_view_over_both_facets_drops_a_rules_presentation() {
    // `mt ls '<filter>' --columns @default --sort @default` is what the CLI
    // page tells a caller to type to read a narrowed listing in the project's
    // ordinary table. It works because the shipped `[views.default]` declares
    // neither facet and an empty facet IS the built-in — so a default view
    // that grows a `sort` or `columns` would quietly change what the
    // documented command means.
    let dir = tempdir();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
name = "release"
sort = ["due:asc"]
columns = ["due", "title"]
"#,
    );
    let mut cli_args = args(Some("priority > 1"));
    cli_args.flags.sort = vec!["@default".to_owned()];
    cli_args.flags.columns = Some("@default".to_owned());
    let (bundle, applied) = resolve(&project, &cli_args).unwrap();
    assert!(bundle.sort.is_empty(), "{:?}", bundle.sort);
    assert!(bundle.columns.is_empty(), "{:?}", bundle.columns);
    assert!(applied.rule_superseded);
}
