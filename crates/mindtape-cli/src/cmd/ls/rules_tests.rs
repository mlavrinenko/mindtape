//! Tests for the `resolve`/`resolve_bare` cascade: rule matching,
//! `import-views` composition, CLI-argument override precedence. `Applied`'s
//! trailer rendering, `is_trivial`, and `bare_view_ref` parsing live in
//! `cmd/ls/rules_applied_ls_rules_tests_applied.rs`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::parse_config;
use mindtape::test_support::TASK_SRC;

use super::{Applied, Bundle, build_env};
use crate::cmd::ls::{LsArgs, LsFlags};
use crate::test_util::make_project;
use crate::util::Project;

/// [`super::resolve_bare`] against the ambient environment the CLI would
/// build, pinned to the table format. These tests exercise rule matching and
/// facet composition, not the environment; the environment's own predicates
/// are covered in `mindtape`'s `condition_tests.rs` and the per-column
/// resolution in `render/columns_tests.rs`.
pub(crate) fn resolve_bare(project: &Project) -> anyhow::Result<(Bundle, Applied)> {
    super::resolve_bare(project, &build_env(project, "table"))
}

/// [`super::resolve`], same pinning as [`resolve_bare`].
pub(crate) fn resolve(project: &Project, args: &LsArgs) -> anyhow::Result<(Bundle, Applied)> {
    super::resolve(project, args, &build_env(project, "table"))
}

pub(crate) fn args(filter: Option<&str>) -> LsArgs {
    LsArgs {
        filter: filter.map(str::to_owned),
        flags: LsFlags {
            format: crate::render::output::OutputFormat::Table,
            json: false,
            limit: None,
            skip: None,
            sort: vec![],
            columns: None,
            interactive: false,
        },
    }
}

pub(crate) fn project(dir: &Path, config_toml: &str) -> crate::util::Project {
    project_with_config(dir, parse_config(config_toml).unwrap())
}

pub(crate) fn project_with_config(dir: &Path, config: mindtape::Config) -> crate::util::Project {
    make_project(dir, &[("x.typ", TASK_SRC)], config)
}

/// Give `dir` a `.git/HEAD` on `branch`, so `on_branch(...)` conditions can
/// resolve against it (mirrors `crates/mindtape/src/git/tests.rs`'s fixture).
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
    // Neither `name` nor `condition` to be known by, so the rule is labelled
    // `rule` — never the empty string, which is now also a source name.
    assert_eq!(applied.rule.as_deref(), Some("rule"));
    assert_eq!(applied.filter_from, "rule");
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
name = "focus"
import-views = ["inbox"]
filter = 'priority > 5'
sort = ["priority:desc"]
"#,
    );
    let (bundle, applied) = resolve_bare(&project).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
    assert_eq!(bundle.sort, vec!["priority:desc".to_owned()]);
    // Attributed to the rule, not to `manual`: `manual` is the CLI's label
    // alone, so a listing mixing the two can name each.
    assert_eq!(applied.filter_from, "focus");
    assert_eq!(applied.sort_from, "focus");
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
fn explicit_filter_narrows_the_rows_and_keeps_the_rules_presentation() {
    // The whole point of the split: a filter answers "which tasks", so the
    // matched rule goes on answering "how do I read them".
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
name = "release"
filter = "priority > 1"
sort = ["due:asc"]
columns = ["due", "title"]
"#,
    );
    let (bundle, applied) = resolve(&project, &args(Some("priority > 9"))).unwrap();
    assert_eq!(bundle.filter, "priority > 9");
    assert_eq!(bundle.sort, vec!["due:asc".to_owned()]);
    assert_eq!(bundle.columns, vec!["due".to_owned(), "title".to_owned()]);
    assert_eq!(applied.rule.as_deref(), Some("release"));
    assert_eq!(applied.filter_from, "manual");
    assert_eq!(applied.sort_from, "release");
    assert_eq!(applied.columns_from, "release");
    assert!(!applied.rule_superseded);
}

#[test]
fn explicit_filter_keeps_the_default_views_presentation() {
    // No rule involved: the default view is the same kind of standing
    // presentation choice and survives a filter the same way.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.default]
filter = "terminal"
sort = ["due:asc"]
"#,
    );
    let (bundle, applied) = resolve(&project, &args(Some("priority > 9"))).unwrap();
    assert_eq!(bundle.filter, "priority > 9");
    assert_eq!(bundle.sort, vec!["due:asc".to_owned()]);
    assert_eq!(applied.rule, None);
    assert_eq!(applied.filter_from, "manual");
    assert_eq!(applied.sort_from, "default");
}

#[test]
fn a_rule_contributing_only_a_filter_is_reported_superseded() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
name = "release"
filter = "priority > 1"
"#,
    );
    let (bundle, applied) = resolve(&project, &args(Some("priority > 9"))).unwrap();
    assert_eq!(bundle.filter, "priority > 9");
    assert_eq!(applied.rule.as_deref(), Some("release"));
    assert!(applied.rule_superseded);
}

#[test]
fn explicit_view_ref_overrides_the_rule_per_facet() {
    // A named view is the caller choosing a whole reading, so it takes every
    // facet it sets — and leaves the ones it does not to the rule.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[views.special]
filter = "priority > 5"
sort = ["due:asc"]

[[ls.rules]]
name = "release"
sort = ["priority:desc"]
columns = ["status", "title"]
"#,
    );
    let (bundle, applied) = resolve(&project, &args(Some("@special"))).unwrap();
    assert_eq!(bundle.filter, "priority > 5");
    assert_eq!(bundle.sort, vec!["due:asc".to_owned()]);
    assert_eq!(
        bundle.columns,
        vec!["status".to_owned(), "title".to_owned()]
    );
    assert_eq!(applied.filter_from, "special");
    assert_eq!(applied.sort_from, "special");
    assert_eq!(applied.columns_from, "release");
    assert!(!applied.rule_superseded);
}

#[test]
fn cli_sort_flag_leaves_the_rules_columns_alone_under_a_filter() {
    // Every layer is per facet: taking over the sort says nothing about the
    // columns, even with a filter argument on the same command line.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
name = "release"
sort = ["due:asc"]
columns = ["status", "title"]
"#,
    );
    let mut cli_args = args(Some("priority > 9"));
    cli_args.flags.sort = vec!["priority:desc".to_owned()];
    let (bundle, applied) = resolve(&project, &cli_args).unwrap();
    assert_eq!(bundle.sort, vec!["priority:desc".to_owned()]);
    assert_eq!(
        bundle.columns,
        vec!["status".to_owned(), "title".to_owned()]
    );
    assert_eq!(applied.sort_from, "manual");
    assert_eq!(applied.columns_from, "release");
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

#[test]
fn an_empty_filter_widens_to_every_task_and_keeps_the_presentation() {
    // `mt ls ''` is the documented "show me everything" — an explicit
    // match-all filter, not an absent argument, so it still overrides the
    // filter facet while the rule's sort and columns stand.
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        r#"
[[ls.rules]]
name = "release"
filter = 'status != "done"'
sort = ["due:asc"]
"#,
    );
    let (bundle, applied) = resolve(&project, &args(Some(""))).unwrap();
    assert_eq!(bundle.filter, "");
    assert_eq!(bundle.sort, vec!["due:asc".to_owned()]);
    assert_eq!(applied.filter_from, "manual");
    assert_eq!(applied.sort_from, "release");
}
