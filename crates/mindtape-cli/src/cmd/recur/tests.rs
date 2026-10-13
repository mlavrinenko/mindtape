//! Tests for the core recurrence planning helpers: occurrence-chain walking
//! (`occurrence_index`, `successor_of`), date planning (`plan_next`), and the
//! double-spawn guard. The `--advance` chain and cargo-mutants regression
//! tests live in `cmd/recur/tests_advance.rs`, which shares this file's
//! `weekly()` fixture.
#![allow(
    clippy::unwrap_used,
    clippy::many_single_char_names,
    clippy::indexing_slicing
)]

use mindtape::test_support::tempdir_with_dirs;
use std::path::Path;

use mindtape::recur::{PeriodUnit, Recur, RecurConfig, Rule};
use mindtape::test_support::{bare_task, link_edge};

use super::*;
use crate::util::Project;

pub(super) fn weekly(count: Option<u32>, until: Option<&str>) -> Recur {
    Recur {
        rule: Rule::Period {
            unit: PeriodUnit::Weeks,
            value: 1,
            weekdays: vec![],
            start: None,
        },
        anchor: None,
        until: until.map(ToOwned::to_owned),
        count,
        materialize: None,
    }
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn stem_of_extracts_the_file_stem() {
    assert_eq!(stem_of(Path::new("tasks/water-plants.typ")), "water-plants");
}

#[test]
fn occurrence_index_is_one_without_a_predecessor() {
    let task = bare_task("chore");
    let tasks = [task.clone()];
    assert_eq!(occurrence_index(&tasks, &task), 1);
}

#[test]
fn occurrence_index_counts_the_recurred_from_chain() {
    let original = bare_task("chore");
    let mut spawn1 = bare_task("chore-2026-06-08");
    spawn1.links = vec![link_edge("chore.typ", "recurred-from")];
    let tasks = [original.clone(), spawn1.clone()];
    assert_eq!(occurrence_index(&tasks, &spawn1), 2);
}

#[test]
fn successor_of_finds_the_referrer() {
    let original = bare_task("chore");
    let mut spawn1 = bare_task("chore-2026-06-08");
    spawn1.links = vec![link_edge("chore.typ", "recurred-from")];
    let tasks = [original.clone(), spawn1.clone()];
    let found = successor_of(&tasks, &original).unwrap();
    assert_eq!(found.id, "chore-2026-06-08");
}

#[test]
fn successor_of_none_without_a_successor() {
    let original = bare_task("chore");
    let tasks = [original.clone()];
    assert!(successor_of(&tasks, &original).is_none());
}

#[test]
fn plan_next_errors_when_count_is_zero() {
    let recur = weekly(Some(0), None);
    let err = plan_next(&recur, &RecurConfig::default(), date(2026, 6, 18), None).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: series exhausted: count has reached 0\n"
    );
}

#[test]
fn plan_next_errors_past_until() {
    let recur = weekly(None, Some("2026-06-20"));
    // Completion anchor: today 2026-06-18 + 1 week = 2026-06-25, past the
    // 2026-06-20 series bound.
    let err = plan_next(&recur, &RecurConfig::default(), date(2026, 6, 18), None).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: series ended: the next occurrence is past until: '2026-06-25'\n"
    );
}

#[test]
fn plan_next_returns_the_next_date_when_in_bounds() {
    let recur = weekly(Some(3), None);
    let next = plan_next(&recur, &RecurConfig::default(), date(2026, 6, 18), None).unwrap();
    assert_eq!(next, date(2026, 6, 25));
}

#[test]
fn guard_double_spawn_bypassed_by_force() {
    let dir = tempdir_with_dirs(&[".mindtape"]);
    let project = Project {
        root: dir.path().to_path_buf(),
        config: mindtape::Config::default(),
        outcome: mindtape::LoadOutcome::default(),
    };
    // No loaded tasks at all: nothing to find a successor among, so both
    // forced and unforced calls succeed.
    let task = bare_task("chore");
    assert!(guard_double_spawn(&project, &task, false).is_ok());
    assert!(guard_double_spawn(&project, &task, true).is_ok());
}

#[test]
fn guard_double_spawn_refuses_an_existing_successor_unless_forced() {
    let dir = tempdir_with_dirs(&[".mindtape"]);
    let original = bare_task("chore");
    let mut spawn1 = bare_task("chore-2026-06-08");
    spawn1.links = vec![link_edge("chore.typ", "recurred-from")];
    let project = Project {
        root: dir.path().to_path_buf(),
        config: mindtape::Config::default(),
        outcome: mindtape::LoadOutcome {
            tasks: vec![original.clone(), spawn1],
            errors: vec![],
            ..mindtape::LoadOutcome::default()
        },
    };
    let err = guard_double_spawn(&project, &original, false).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: task already has a successor: 'chore'\n\
         hint: pass --force to spawn another\n\
         note: successor: chore-2026-06-08\n"
    );
    assert!(guard_double_spawn(&project, &original, true).is_ok());
}

/// A recurring task's `recur` field round-trips through `Task` for the
/// dispatch tests below.
fn recurring_task(id: &str, materialize: Option<&str>) -> Task {
    let mut task = bare_task(id);
    task.recur = Some(Recur {
        materialize: materialize.map(ToOwned::to_owned),
        ..weekly(None, None)
    });
    task
}

#[test]
fn recurring_task_carries_a_recur_field() {
    let task = recurring_task("chore", Some("mutate"));
    assert_eq!(task.recur.unwrap().materialize.as_deref(), Some("mutate"));
}

#[test]
fn a_policy_that_rejects_a_recur_reports_the_first_problem_and_notes_the_rest() {
    assert!(policy_refusal(&[]).is_none());
    let one = policy_refusal(&["recur is not allowed".to_owned()]).unwrap();
    assert_eq!(crate::diag::plain(&one), "error: recur is not allowed\n");
    let two = policy_refusal(&["first".to_owned(), "second".to_owned()]).unwrap();
    assert_eq!(
        crate::diag::plain(&two),
        "error: first\nnote: also: second\n"
    );
}

#[test]
fn a_file_with_no_parent_names_itself() {
    let err = parent_dir(Path::new("/")).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: task file has no parent directory: '/'\n"
    );
    assert_eq!(parent_dir(Path::new("a/b.typ")).unwrap(), Path::new("a"));
}
