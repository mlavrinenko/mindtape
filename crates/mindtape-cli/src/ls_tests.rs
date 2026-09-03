#![allow(clippy::unwrap_used)]
use std::path::Path;

use mindtape::numeric::NumValue;
use mindtape::test_support::{TASK_SRC as TASK, bare_task};
use mindtape::{Config, Readiness, Task};

use mindtape::sort::sort_by_specs;

use super::{LsArgs, resolve_and_select, run_with};
use crate::output::OutputFormat;

fn task(id: &str, priority: f64) -> Task {
    let mut t = bare_task(id);
    t.priority = Some(NumValue::Num(priority));
    t
}

#[test]
fn sort_by_specs_orders_priority_numerically() {
    // Ascending: lower number first. Lexicographic "8" < "2" is false,
    // so this only passes with numeric ordering.
    let mut tasks = vec![task("b", 8.0), task("a", 2.0)];
    sort_by_specs(
        &mut tasks,
        &["priority:asc".to_owned()],
        &Config::default(),
        &Readiness::empty(),
    )
    .unwrap();
    assert_eq!(tasks.first().unwrap().id, "a");
}

#[test]
fn sort_by_specs_rejects_unknown_field() {
    let mut tasks = vec![task("a", 1.0)];
    assert!(
        sort_by_specs(
            &mut tasks,
            &["no_such_field".to_owned()],
            &Config::default(),
            &Readiness::empty(),
        )
        .is_err()
    );
}

fn make_project(dir: &Path, files: &[(&str, &str)]) -> crate::util::Project {
    crate::test_util::make_project(dir, files, Config::default())
}

fn default_args() -> LsArgs {
    LsArgs {
        filter: None,
        flags: crate::ls::LsFlags {
            format: OutputFormat::Table,
            json: false,
            limit: None,
            sort: vec![],
            columns: None,
            interactive: false,
        },
    }
}

fn config_with_views() -> Config {
    mindtape::parse_config(
        "[views.hot]\nfilter = 'priority > 5'\nsort = ['priority:desc']\n\
         columns = ['status', 'title']\n\
         [views.broken]\nfilter = 'blocked'\n",
    )
    .unwrap()
}

fn view_project(dir: &Path) -> crate::util::Project {
    crate::test_util::make_project(dir, &[("x.typ", TASK)], config_with_views())
}

#[test]
fn run_with_table_format_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    assert!(run_with(&default_args(), project).is_ok());
}

#[test]
fn run_with_json_flag_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let mut args = default_args();
    args.flags.json = true;
    assert!(run_with(&args, project).is_ok());
}

#[test]
fn run_with_csv_format_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let mut args = default_args();
    args.flags.format = OutputFormat::Csv;
    assert!(run_with(&args, project).is_ok());
}

#[test]
fn run_with_limit_truncates() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
    let mut args = default_args();
    args.flags.limit = Some(1);
    assert!(run_with(&args, project).is_ok());
}

#[test]
fn run_with_invalid_filter_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let args = LsArgs {
        filter: Some("!!bad!!".to_owned()),
        ..default_args()
    };
    assert!(run_with(&args, project).is_err());
}

#[test]
fn run_with_invalid_sort_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let mut args = default_args();
    args.flags.sort = vec!["no_such_field".to_owned()];
    assert!(run_with(&args, project).is_err());
}

#[test]
fn empty_filter_lists_everything() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let args = LsArgs {
        filter: Some(String::new()),
        ..default_args()
    };
    assert!(run_with(&args, project).is_ok());
}

#[test]
fn bundle_view_reference_runs() {
    let dir = tempfile::tempdir().unwrap();
    let args = LsArgs {
        filter: Some("@hot".to_owned()),
        ..default_args()
    };
    assert!(run_with(&args, view_project(dir.path())).is_ok());
}

#[test]
fn unknown_bundle_view_errors() {
    let dir = tempfile::tempdir().unwrap();
    let args = LsArgs {
        filter: Some("@ghost".to_owned()),
        ..default_args()
    };
    assert!(run_with(&args, view_project(dir.path())).is_err());
}

#[test]
fn columns_flag_selects_named_columns() {
    let dir = tempfile::tempdir().unwrap();
    let mut args = default_args();
    args.flags.columns = Some("status,title".to_owned());
    assert!(run_with(&args, view_project(dir.path())).is_ok());
}

#[test]
fn columns_at_view_runs() {
    let dir = tempfile::tempdir().unwrap();
    let mut args = default_args();
    args.flags.columns = Some("@hot".to_owned());
    assert!(run_with(&args, view_project(dir.path())).is_ok());
}

#[test]
fn unknown_column_errors() {
    let dir = tempfile::tempdir().unwrap();
    let mut args = default_args();
    args.flags.columns = Some("bogus".to_owned());
    assert!(run_with(&args, view_project(dir.path())).is_err());
}

#[test]
fn sort_at_view_expands() {
    let dir = tempfile::tempdir().unwrap();
    let mut args = default_args();
    args.flags.sort = vec!["@hot".to_owned()];
    assert!(run_with(&args, view_project(dir.path())).is_ok());
}

#[test]
fn default_view_drives_bare_listing() {
    // A `[views.default]` view supplies the filter for a bare `mt ls`.
    // TASK is `wip` (non-terminal), so a `terminal` default view hides it.
    let dir = tempfile::tempdir().unwrap();
    let config = mindtape::parse_config("[views.default]\nfilter = 'terminal'\n").unwrap();
    let project = crate::test_util::make_project(dir.path(), &[("x.typ", TASK)], config);
    let listing = resolve_and_select(&default_args(), project).unwrap();
    assert!(listing.tasks.is_empty());
}

#[test]
fn resolve_and_select_applies_view_columns() {
    let dir = tempfile::tempdir().unwrap();
    let project = view_project(dir.path());
    let args = LsArgs {
        filter: Some("@hot".to_owned()),
        ..default_args()
    };
    let listing = resolve_and_select(&args, project).unwrap();
    assert_eq!(listing.config.ls.columns_order, vec!["status", "title"]);
}

#[test]
fn explicit_empty_filter_bypasses_default_view() {
    // `mt ls ''` is match-all: the default view is ignored entirely.
    let dir = tempfile::tempdir().unwrap();
    let config = mindtape::parse_config("[views.default]\nfilter = 'terminal'\n").unwrap();
    let project = crate::test_util::make_project(dir.path(), &[("x.typ", TASK)], config);
    let args = LsArgs {
        filter: Some(String::new()),
        ..default_args()
    };
    let listing = resolve_and_select(&args, project).unwrap();
    assert_eq!(listing.tasks.len(), 1);
}

#[test]
fn limit_truncates_and_reports_what_it_cut() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(
        dir.path(),
        &[("a.typ", TASK), ("b.typ", TASK), ("c.typ", TASK)],
    );
    let mut args = default_args();
    args.flags.limit = Some(2);
    let listing = resolve_and_select(&args, project).unwrap();
    assert_eq!(listing.tasks.len(), 2);
    // `total` is the match count taken BEFORE the cut — the number the
    // caller would see with a bigger `-n`, not the number printed.
    assert_eq!(
        listing.truncated,
        Some(crate::output::Truncation { shown: 2, total: 3 })
    );
}

#[test]
fn a_limit_that_hides_nothing_reports_no_truncation() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
    let mut args = default_args();
    args.flags.limit = Some(5);
    assert!(
        resolve_and_select(&args, project)
            .unwrap()
            .truncated
            .is_none()
    );
}

#[test]
fn no_limit_reports_no_truncation() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
    assert!(
        resolve_and_select(&default_args(), project)
            .unwrap()
            .truncated
            .is_none()
    );
}
