//! Tests for the `resolve`/`resolve_bare` cascade: rule matching,
//! `import-views` composition, CLI-argument override precedence. `Applied`'s
//! trailer rendering, `is_trivial`, and `bare_view_ref` parsing live in
//! `ls_rules_tests_applied.rs`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::parse_config;
use mindtape::test_support::TASK_SRC;

use super::{resolve, resolve_bare};
use crate::ls::{LsArgs, LsFlags};
use crate::test_util::make_project;

pub(super) fn args(filter: Option<&str>) -> LsArgs {
    LsArgs {
        filter: filter.map(str::to_owned),
        flags: LsFlags {
            format: crate::output::OutputFormat::Table,
            json: false,
            limit: None,
            sort: vec![],
            columns: None,
            interactive: false,
        },
    }
}

pub(super) fn project(dir: &Path, config_toml: &str) -> crate::util::Project {
    project_with_config(dir, parse_config(config_toml).unwrap())
}

pub(super) fn project_with_config(dir: &Path, config: mindtape::Config) -> crate::util::Project {
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
    cli_args.flags.sort = vec!["due:asc".to_owned()];
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
    // "ghost" is far from the only view the embedded floor supplies
    // ("default"), so the message enumerates without a suggestion
    // (`truth/tasks/suggest-a-nearest-match-on-an-unknown-view-name.typ`).
    let err = resolve_bare(&project).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("known views: default"), "{msg}");
    assert!(!msg.contains("did you mean"), "{msg}");
}

#[test]
fn unknown_import_view_near_miss_suggests_the_declared_view() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.release]
filter = ""
[[ls.rules]]
import-views = ["releasee"]
"#,
    );
    let err = resolve_bare(&project).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("did you mean: release?"), "{msg}");
}

// ---------------------------------------------------------------------------
// Regression tests for cargo-mutants survivors in ls_rules.rs
// ---------------------------------------------------------------------------

// Pins delete rule from Self in Applied::match_all.
//   PROVABLY EQUIVALENT: ..Self::default() already supplies rule: None.
//   No test needed.

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
