#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::parse_config;
use mindtape::test_support::TASK_SRC;

use super::{Applied, MATCH_ALL, resolve, resolve_bare, trailer};
use crate::ls::LsArgs;
use crate::test_util::make_project;

fn args(filter: Option<&str>) -> LsArgs {
    LsArgs {
        filter: filter.map(str::to_owned),
        format: crate::output::OutputFormat::Table,
        json: false,
        limit: None,
        sort: vec![],
        columns: None,
        interactive: false,
    }
}

fn project(dir: &Path, config_toml: &str) -> crate::util::Project {
    project_with_config(dir, parse_config(config_toml).unwrap())
}

fn project_with_config(dir: &Path, config: mindtape::Config) -> crate::util::Project {
    make_project(dir, &[("x.typ", TASK_SRC)], config)
}

/// Give `dir` a `.git/HEAD` on `branch`, so `on_branch(...)` conditions can
/// resolve against it (mirrors `crates/mindtape/src/git_tests.rs`'s fixture).
fn checkout(dir: &Path, branch: &str) {
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    std::fs::write(
        dir.join(".git").join("HEAD"),
        format!("ref: refs/heads/{branch}\n"),
    )
    .unwrap();
}

#[test]
fn first_matching_rule_wins_over_later_ones() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
condition = 'on_branch("never-matches")'
filter = "priority > 1"

[[ls.rules]]
filter = "priority > 2"
"#,
    );
    let (bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.filter, "priority > 2");
    assert_eq!(applied.rule.as_deref(), Some(""));
}

#[test]
fn rule_composes_import_views_left_to_right_last_wins() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.inbox]
filter = 'status != "done"'
sort = ["due:asc"]
columns = ["status", "title"]

[views.release]
sort = ["priority:desc"]

[[ls.rules]]
name = "release"
import-views = ["inbox", "release(sort)"]
"#,
    );
    let (bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.filter, "status != \"done\"");
    assert_eq!(bundle.sort, vec!["priority:desc".to_owned()]);
    assert_eq!(
        bundle.columns,
        vec!["status".to_owned(), "title".to_owned()]
    );
    assert_eq!(applied.filter_from, "inbox");
    assert_eq!(applied.sort_from, "release");
    assert_eq!(applied.columns_from, "inbox");
}

#[test]
fn manual_override_beats_import_views() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.inbox]
filter = 'status != "done"'
sort = ["due:asc"]

[[ls.rules]]
import-views = ["inbox"]
filter = 'priority > 5'
sort = ["priority:desc"]
"#,
    );
    let (bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
    assert_eq!(bundle.sort, vec!["priority:desc".to_owned()]);
    assert_eq!(applied.filter_from, "manual");
    assert_eq!(applied.sort_from, "manual");
}

#[test]
fn manual_sort_splices_view_reference() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.release]
sort = ["priority:desc", "due:asc"]

[[ls.rules]]
sort = ["@release"]
"#,
    );
    let (bundle, _applied) = resolve_bare(&project).unwrap();
    assert_eq!(
        bundle.sort,
        vec!["priority:desc".to_owned(), "due:asc".to_owned()]
    );
}

#[test]
fn on_branch_condition_selects_matching_rule() {
    let dir = tempfile::tempdir().unwrap();
    checkout(dir.path(), "release/1.0");
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
name = "release"
condition = 'on_branch("release/*")'
filter = "priority > 1"

[[ls.rules]]
filter = "priority > 2"
"#,
    );
    let (bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.filter, "priority > 1");
    assert_eq!(applied.rule.as_deref(), Some("release"));
}

#[test]
fn on_branch_condition_skips_when_branch_differs() {
    let dir = tempfile::tempdir().unwrap();
    checkout(dir.path(), "main");
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
condition = 'on_branch("release/*")'
filter = "priority > 1"

[[ls.rules]]
filter = "priority > 2"
"#,
    );
    let (bundle, _applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.filter, "priority > 2");
}

#[test]
fn default_view_fallback_when_no_rule_matches() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
condition = 'on_branch("release/*")'
filter = "priority > 1"

[views.default]
filter = "terminal"
"#,
    );
    let (bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.filter, "terminal");
    assert_eq!(applied.rule, None);
    assert_eq!(applied.filter_from, "default");
}

#[test]
fn match_all_fallback_when_nothing_resolves() {
    // A bare `Config::default()` (bypassing `parse_config`, so no embedded
    // floor `[views.default]` applies): no rules, no default view at all.
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_config(dir.path(), mindtape::Config::default());
    let (bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.filter, "");
    assert!(bundle.sort.is_empty());
    assert!(applied.is_trivial());
}

#[test]
fn explicit_filter_bypasses_rules_and_default_view() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
filter = "priority > 1"

[views.default]
filter = "terminal"
"#,
    );
    let (bundle, applied) = resolve(&project, &args(Some("priority > 9"))).unwrap();
    assert_eq!(bundle.filter, "priority > 9");
    assert_eq!(applied.rule, None);
    assert_eq!(applied.filter_from, "manual");
}

#[test]
fn explicit_view_ref_bypasses_rules_and_carries_its_own_bundle() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.special]
filter = "priority > 5"
sort = ["due:asc"]
columns = ["status", "title"]

[[ls.rules]]
filter = "priority > 1"
"#,
    );
    let (bundle, applied) = resolve(&project, &args(Some("@special"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
    assert_eq!(bundle.sort, vec!["due:asc".to_owned()]);
    assert_eq!(applied.rule, None);
    assert_eq!(applied.filter_from, "special");
    assert_eq!(applied.sort_from, "special");
}

#[test]
fn cli_sort_flag_overrides_a_matched_rule() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
sort = ["priority:desc"]
"#,
    );
    let mut cli_args = args(None);
    cli_args.sort = vec!["due:asc".to_owned()];
    let (bundle, applied) = resolve(&project, &cli_args).unwrap();
    assert_eq!(bundle.sort, vec!["due:asc".to_owned()]);
    assert_eq!(applied.sort_from, "manual");
}

#[test]
fn unknown_import_view_errors() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
import-views = ["ghost"]
"#,
    );
    assert!(resolve_bare(&project).is_err());
}

#[test]
fn trailer_groups_by_source_in_first_appearance_order() {
    let applied = Applied {
        rule: Some("release".to_owned()),
        filter_from: "inbox".to_owned(),
        filter_value: String::new(),
        sort_from: "release".to_owned(),
        sort_value: vec![],
        columns_from: "inbox".to_owned(),
        columns_value: vec![],
    };
    assert_eq!(
        trailer(&applied).as_deref(),
        Some("applied: inbox (filter, columns) → release (sort)")
    );
}

#[test]
fn trailer_omits_match_all_facets() {
    let applied = Applied {
        rule: Some("release".to_owned()),
        filter_from: "match-all".to_owned(),
        sort_from: "release".to_owned(),
        columns_from: "match-all".to_owned(),
        ..Applied::default()
    };
    assert_eq!(
        trailer(&applied).as_deref(),
        Some("applied: release (sort)")
    );
}

#[test]
fn trailer_is_none_when_trivial() {
    let applied = Applied {
        rule: None,
        filter_from: "match-all".to_owned(),
        sort_from: "match-all".to_owned(),
        columns_from: "match-all".to_owned(),
        ..Applied::default()
    };
    assert!(applied.is_trivial());
    assert_eq!(trailer(&applied).as_deref(), None);
}

// ---------------------------------------------------------------------------
// Regression tests for cargo-mutants survivors in ls_rules.rs
// ---------------------------------------------------------------------------

// Pins delete rule from Self in Applied::match_all.
//   PROVABLY EQUIVALENT: ..Self::default() already supplies rule: None.
//   No test needed.

#[test]
fn is_trivial_false_when_rule_is_some_but_sort_columns_match_all() {
    // Kills mutant at second && → || in is_trivial.
    // Real:   rule.is_some() && filter_from != match-all && sort_from == match-all && ..
    //         → false
    // Mutant: (rule.is_none() && filter_from==match-all) || (sort_from==match-all && ..)
    //         → true
    let applied = Applied {
        rule: Some("x".to_owned()),
        filter_from: "manual".to_owned(),
        sort_from: MATCH_ALL.to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    assert!(!applied.is_trivial());
}

#[test]
fn is_trivial_false_when_filter_from_not_match_all() {
    // Kills mutant at first && → || in is_trivial.
    // Real:   rule.is_none() && filter_from != match-all && …
    //         → false
    // Mutant: rule.is_none() || (filter_from==match-all && sort_from==match-all && …)
    //         → true
    let applied = Applied {
        rule: None,
        filter_from: "manual".to_owned(),
        sort_from: MATCH_ALL.to_owned(),
        columns_from: MATCH_ALL.to_owned(),
        ..Applied::default()
    };
    assert!(!applied.is_trivial());
}

#[test]
fn explicit_filter_sets_filter_value_in_applied() {
    // Kills mutant at delete filter_value from Applied in explicit_bundle.
    // Real:   applied.filter_value == expr
    // Mutant: applied.filter_value == ""  (falls through to ..Applied::match_all())
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
fn bare_at_sign_is_manual_filter_not_view_ref() {
    // Kills mutant at && → || in bare_view_ref.
    // "@" has empty name after @, so bare_view_ref returns None in real code.
    // Mutant makes it return Some(""), which then errors looking up view "".
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_config(dir.path(), mindtape::Config::default());
    let (bundle, applied) = resolve(&project, &args(Some("@"))).unwrap();
    assert_eq!(bundle.filter, "@");
    assert_eq!(applied.filter_from, "manual");
}

#[test]
fn view_ref_with_underscore_is_valid() {
    // Kills mutant at second || → && in bare_view_ref.
    // "_" is a valid view-name character. Mutant: c.is_ascii_alphanumeric() || (c=='_' && c=='-')
    // → only alphanumeric passes, so "@foo_bar" is misrecognised as manual.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.foo_bar]
filter = "priority > 5"
"#,
    );
    let (bundle, _applied) = resolve(&project, &args(Some("@foo_bar"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
}

#[test]
fn view_ref_with_hyphen_is_valid() {
    // Kills mutant at == → != in bare_view_ref.
    // "-" is a valid view-name char under ==; under != it is rejected.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.my-view]
filter = "priority > 5"
"#,
    );
    let (bundle, _applied) = resolve(&project, &args(Some("@my-view"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
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
