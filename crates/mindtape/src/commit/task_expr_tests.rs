#![allow(clippy::unwrap_used)]

use super::{TaskScope, desugar_commit_into, task_expr_problem};
use crate::commit::CommitRecord;
use crate::commit::expr::CommitExpr;
use crate::commit::tasks::{CommitStates, ProjectState};
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::test_support::bare_task;

fn config() -> Config {
    parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap()
}

fn at(status: &str) -> Task {
    let mut task = bare_task("parser");
    task.status = Some(Status {
        name: status.to_owned(),
        date: Some("2026-06-01".to_owned()),
        note: None,
    });
    task
}

/// `parser` wip at `HEAD`, closed in the index, and absent from the working
/// tree.
fn states(config: &Config) -> CommitStates {
    CommitStates {
        head: ProjectState::new(vec![at("wip")], config, Vec::new()),
        staged: ProjectState::new(vec![at("done")], config, Vec::new()),
        worktree: Some(ProjectState::new(Vec::new(), config, Vec::new())),
    }
}

fn commit(refs: &[&str]) -> CommitRecord {
    CommitRecord {
        refs: refs.iter().map(|stem| (*stem).to_owned()).collect(),
        ..CommitRecord::default()
    }
}

fn holds(expr: &str, commit: &CommitRecord) -> bool {
    let config = config();
    let states = states(&config);
    let scope = TaskScope::new("parser", &states, &config);
    CommitExpr::parse(&desugar_commit_into(expr))
        .unwrap()
        .matches_task(commit, &scope)
        .unwrap()
}

#[test]
fn each_wrapper_reads_its_own_state() {
    let none = commit(&[]);
    assert!(holds("head(status == \"wip\")", &none));
    assert!(holds(
        "staged(status == \"done\") && staged(terminal)",
        &none
    ));
    assert!(holds(
        "head(exists) && staged(exists) && !worktree(exists)",
        &none
    ));
    assert!(holds("head(status) != staged(status)", &none));
    assert!(holds("staged(status) == \"done\"", &none));
    assert!(holds("staged(count(terminal) == 1)", &none));
}

#[test]
fn into_is_a_flip_between_head_and_staged() {
    let none = commit(&[]);
    assert!(holds("into(\"done\")", &none));
    assert!(!holds("into(\"wip\")", &none));
    assert_eq!(
        desugar_commit_into("into(\"done\") && named"),
        "(staged(status == \"done\") && !head(status == \"done\")) && named"
    );
}

#[test]
fn named_reads_the_trailer() {
    assert!(holds("named", &commit(&["parser"])));
    assert!(!holds("named()", &commit(&["lexer"])));
}

#[test]
fn a_missing_working_tree_is_an_error_not_a_guess() {
    let config = config();
    let states = CommitStates {
        worktree: None,
        ..states(&config)
    };
    let scope = TaskScope::new("parser", &states, &config);
    let err = CommitExpr::parse("worktree(exists)")
        .unwrap()
        .matches_task(&commit(&[]), &scope)
        .unwrap_err();
    assert!(err.to_string().contains("no working tree"), "{err}");
}

#[test]
fn a_bare_task_field_is_refused_naming_the_wrappers() {
    let config = config();
    for bare in ["status == \"done\"", "terminal", "exists", "has(estimate)"] {
        let problem = task_expr_problem(bare, &config).unwrap();
        assert!(problem.contains("staged("), "{bare}: {problem}");
    }
}

#[test]
fn a_well_formed_task_expression_passes_the_dry_run() {
    let config = config();
    for good in [
        "into(\"done\") || into(\"wip\")",
        "named && (!tasks_only || staged(has_tag(\"decision\")))",
        "worktree(status) != staged(status)",
        "staged(changed(\"staged\", \"src/**\") <= 3)",
        "head(ready) || head(blocked)",
    ] {
        assert_eq!(task_expr_problem(good, &config), None, "{good}");
    }
}

#[test]
fn a_malformed_wrapper_or_status_is_refused() {
    let config = config();
    assert!(
        task_expr_problem("into(\"dnoe\")", &config)
            .unwrap()
            .contains("dnoe")
    );
    assert!(task_expr_problem("staged(status, title)", &config).is_some());
    assert!(task_expr_problem("staged(bogus == 1)", &config).is_some());
}

#[test]
fn a_commit_rule_refuses_every_task_reading() {
    for task_only in ["named", "staged(terminal)", "head(status) == \"wip\""] {
        let problem =
            crate::commit::expr::commit_expr_problem(task_only, &crate::config::Config::default())
                .unwrap();
        assert!(
            problem.contains("scope = \"task\""),
            "{task_only}: {problem}"
        );
    }
}

#[test]
fn an_absent_task_stand_in_carries_its_stem_as_id() {
    let none = commit(&[]);
    assert!(holds("worktree(id) == \"parser\"", &none));
}

#[test]
fn changed_passes_the_dry_run_even_where_git_cannot_answer() {
    let mut config = config();
    let dir = tempfile::tempdir().unwrap();
    config.git.set_root(dir.path());
    assert_eq!(
        task_expr_problem("staged(changed(\"staged\", \"src/**\") <= 3)", &config),
        None,
        "the load-time dry run must not read git"
    );
}
