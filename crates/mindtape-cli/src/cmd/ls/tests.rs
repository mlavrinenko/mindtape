#![allow(clippy::unwrap_used)]
use mindtape::test_support::tempdir;
use std::path::Path;

use mindtape::model::numeric::NumValue;
use mindtape::test_support::{TASK_SRC as TASK, bare_task};
use mindtape::{Config, Readiness, Task};

use mindtape::sort::sort_by_specs;

use super::{LsArgs, next_command_hint, resolve_and_select, run_with};
use crate::render::output::OutputFormat;

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
        flags: crate::cmd::ls::LsFlags {
            format: OutputFormat::Table,
            json: false,
            limit: None,
            skip: None,
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
    let dir = tempdir();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    assert!(run_with(&default_args(), project).is_ok());
}

#[test]
fn run_with_json_flag_succeeds() {
    let dir = tempdir();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let mut args = default_args();
    args.flags.json = true;
    assert!(run_with(&args, project).is_ok());
}

#[test]
fn run_with_csv_format_succeeds() {
    let dir = tempdir();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let mut args = default_args();
    args.flags.format = OutputFormat::Csv;
    assert!(run_with(&args, project).is_ok());
}

#[test]
fn run_with_limit_truncates() {
    let dir = tempdir();
    let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
    let mut args = default_args();
    args.flags.limit = Some(1);
    assert!(run_with(&args, project).is_ok());
}

#[test]
fn run_with_invalid_filter_returns_err() {
    let dir = tempdir();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let args = LsArgs {
        filter: Some("!!bad!!".to_owned()),
        ..default_args()
    };
    assert!(run_with(&args, project).is_err());
}

#[test]
fn run_with_invalid_sort_returns_err() {
    let dir = tempdir();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let mut args = default_args();
    args.flags.sort = vec!["no_such_field".to_owned()];
    assert!(run_with(&args, project).is_err());
}

#[test]
fn empty_filter_lists_everything() {
    let dir = tempdir();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    let args = LsArgs {
        filter: Some(String::new()),
        ..default_args()
    };
    assert!(run_with(&args, project).is_ok());
}

#[test]
fn bundle_view_reference_runs() {
    let dir = tempdir();
    let args = LsArgs {
        filter: Some("@hot".to_owned()),
        ..default_args()
    };
    assert!(run_with(&args, view_project(dir.path())).is_ok());
}

#[test]
fn unknown_bundle_view_errors() {
    let dir = tempdir();
    let args = LsArgs {
        filter: Some("@ghost".to_owned()),
        ..default_args()
    };
    assert!(run_with(&args, view_project(dir.path())).is_err());
}

#[test]
fn columns_flag_selects_named_columns() {
    let dir = tempdir();
    let mut args = default_args();
    args.flags.columns = Some("status,title".to_owned());
    assert!(run_with(&args, view_project(dir.path())).is_ok());
}

#[test]
fn columns_at_view_runs() {
    let dir = tempdir();
    let mut args = default_args();
    args.flags.columns = Some("@hot".to_owned());
    assert!(run_with(&args, view_project(dir.path())).is_ok());
}

#[test]
fn unknown_column_errors() {
    let dir = tempdir();
    let mut args = default_args();
    args.flags.columns = Some("bogus".to_owned());
    assert!(run_with(&args, view_project(dir.path())).is_err());
}

#[test]
fn sort_at_view_expands() {
    let dir = tempdir();
    let mut args = default_args();
    args.flags.sort = vec!["@hot".to_owned()];
    assert!(run_with(&args, view_project(dir.path())).is_ok());
}

#[test]
fn default_view_drives_bare_listing() {
    // A `[views.default]` view supplies the filter for a bare `mt ls`.
    // TASK is `wip` (non-terminal), so a `terminal` default view hides it.
    let dir = tempdir();
    let config = mindtape::parse_config("[views.default]\nfilter = 'terminal'\n").unwrap();
    let project = crate::test_util::make_project(dir.path(), &[("x.typ", TASK)], config);
    let listing = resolve_and_select(&default_args(), project).unwrap();
    assert!(listing.tasks.is_empty());
}

#[test]
fn resolve_and_select_applies_view_columns() {
    let dir = tempdir();
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
    let dir = tempdir();
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
    let dir = tempdir();
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
        Some(crate::render::output::Truncation {
            shown: 2,
            total: 3,
            skip: 0,
        })
    );
}

#[test]
fn a_limit_that_hides_nothing_reports_no_truncation() {
    let dir = tempdir();
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
    let dir = tempdir();
    let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
    assert!(
        resolve_and_select(&default_args(), project)
            .unwrap()
            .truncated
            .is_none()
    );
}

// --- --skip (`truth/tasks/add-skip-and-a-next-command-hint-to-ls-paging.typ`)

#[test]
fn skip_drops_rows_after_the_sort() {
    let dir = tempdir();
    let project = make_project(
        dir.path(),
        &[("a.typ", TASK), ("b.typ", TASK), ("c.typ", TASK)],
    );
    let mut args = default_args();
    args.flags.skip = Some(1);
    let listing = resolve_and_select(&args, project).unwrap();
    // Default order tie-breaks on id ascending, so `--skip 1` drops "a".
    assert_eq!(
        listing
            .tasks
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        vec!["b", "c"]
    );
    // Every remaining row is shown, so there is nothing left to page to.
    assert!(listing.truncated.is_none());
}

#[test]
fn skip_and_limit_compose_into_one_page() {
    let dir = tempdir();
    let project = make_project(
        dir.path(),
        &[
            ("a.typ", TASK),
            ("b.typ", TASK),
            ("c.typ", TASK),
            ("d.typ", TASK),
            ("e.typ", TASK),
        ],
    );
    let mut args = default_args();
    args.flags.skip = Some(1);
    args.flags.limit = Some(2);
    let listing = resolve_and_select(&args, project).unwrap();
    assert_eq!(
        listing
            .tasks
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        vec!["b", "c"]
    );
    assert_eq!(
        listing.truncated,
        Some(crate::render::output::Truncation {
            shown: 2,
            total: 5,
            skip: 1,
        })
    );
}

#[test]
fn skip_beyond_the_match_count_yields_an_empty_page() {
    let dir = tempdir();
    let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
    let mut args = default_args();
    args.flags.skip = Some(50);
    let listing = resolve_and_select(&args, project).unwrap();
    assert!(listing.tasks.is_empty());
    assert!(listing.truncated.is_none());
}

// --- next-command hint -------------------------------------------------

#[test]
fn next_command_hint_reproduces_the_filter_and_appends_skip() {
    let mut args = default_args();
    args.filter = Some("priority > 5".to_owned());
    args.flags.limit = Some(20);
    assert_eq!(
        next_command_hint(&args, 20),
        "mt ls 'priority > 5' -n 20 --skip 20"
    );
}

#[test]
fn next_command_hint_advances_past_an_existing_skip() {
    // The hint never restates the run's own `--skip` — `next_skip` (computed
    // from it by `Truncation::next_skip`) already carries it forward.
    let mut args = default_args();
    args.flags.limit = Some(20);
    args.flags.skip = Some(20);
    assert_eq!(next_command_hint(&args, 40), "mt ls -n 20 --skip 40");
}

#[test]
fn next_command_hint_carries_json_sort_and_columns() {
    let mut args = default_args();
    args.flags.limit = Some(5);
    args.flags.json = true;
    args.flags.sort = vec!["priority:desc".to_owned()];
    args.flags.columns = Some("status,title".to_owned());
    assert_eq!(
        next_command_hint(&args, 5),
        "mt ls --json -n 5 --sort 'priority:desc' --columns 'status,title' --skip 5"
    );
}

#[test]
fn next_command_hint_omits_the_filter_token_when_none_was_given() {
    // A bare `mt ls` (no filter argument at all) must page as a bare `mt ls`,
    // not grow a `''` that changes which default view applies.
    let mut args = default_args();
    args.flags.limit = Some(10);
    assert_eq!(next_command_hint(&args, 10), "mt ls -n 10 --skip 10");
}
