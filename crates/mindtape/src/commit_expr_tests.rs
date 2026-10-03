#![allow(clippy::unwrap_used)]

use super::{CommitExpr, commit_expr_problem};
use crate::commit_msg::CommitRecord;

fn commit() -> CommitRecord {
    CommitRecord {
        subject: "docs: fix a typo".to_owned(),
        refs: vec!["parser".to_owned()],
        files: vec!["docs/guide.md".to_owned(), "src/lib.rs".to_owned()],
        tasks_only: false,
        tasks: Vec::new(),
    }
}

fn eval(expr: &str) -> bool {
    CommitExpr::parse(expr).unwrap().matches(&commit()).unwrap()
}

#[test]
fn fields_read_the_commit() {
    assert!(eval("has(refs) && has(files) && has(subject)"));
    assert!(!eval("miss(refs)"));
    assert!(!eval("tasks_only") && !eval("tasks_only()"));
    assert!(eval("subject == \"docs: fix a typo\""));
    assert!(eval("subject != \"feat\" && subject > \"a\""));
    assert!(eval(""));
}

#[test]
fn count_files_counts_what_the_globs_admit() {
    assert!(eval("count(files, \"**\") == 2"));
    assert!(eval("count(files) == 2"));
    assert!(eval("count(files, \"!docs/**\") == 1"));
    assert!(eval(
        "count(files, \"src/**\", \"docs/**\", \"!**/*.md\") <= 1"
    ));
    assert!(!eval("count(files, \"nothing/**\") > 0"));
}

#[test]
fn count_files_refuses_other_fields_and_bad_globs() {
    for expr in [
        "count(refs, \"**\") == 1",
        "count() == 1",
        "count(files, \"[\") == 1",
    ] {
        assert!(commit_expr_problem(expr).is_some(), "{expr}");
    }
}

#[test]
fn matches_over_a_list_is_any_entry() {
    assert!(eval("matches(files, \"^docs/\")"));
    assert!(eval("matches(files, \"^src/\")"));
    assert!(!eval("matches(files, \"^services/\")"));
    assert!(eval("matches(refs, \"^pars\")"));
    assert!(eval("matches(subject, \"^docs\")"));
}

#[test]
fn an_empty_commit_has_nothing() {
    let empty = CommitRecord::default();
    let holds = |expr: &str| CommitExpr::parse(expr).unwrap().matches(&empty).unwrap();
    assert!(holds("miss(refs) && miss(files) && miss(subject)"));
}

#[test]
fn a_task_field_is_refused_naming_what_a_commit_rule_reads() {
    let problem = commit_expr_problem("status == \"wip\"").unwrap();
    assert!(problem.contains("status"), "{problem}");
    assert!(
        problem.contains("subject, refs, files, tasks_only"),
        "{problem}"
    );
    assert!(commit_expr_problem("terminal").is_some());
    assert!(commit_expr_problem("has(title)").is_some());
}

#[test]
fn malformed_expressions_are_named() {
    for bad in [
        "has((",
        "refs == \"x\"",
        "subject",
        "matches(files)",
        "matches(files, \"(\")",
        "has(refs, files)",
        "ready()",
        "1 + 2",
    ] {
        assert!(
            commit_expr_problem(bad).is_some(),
            "{bad} should be refused"
        );
    }
    assert!(commit_expr_problem("!tasks_only || has(refs)").is_none());
}

#[test]
fn a_non_state_function_side_is_a_literal_not_a_state_wrapper() {
    // `now()` is not one of the state wrappers, so a comparison side treats it
    // as a written-out value and refuses it there — it is never handed to a
    // task scope as if it were `head`/`staged`/`worktree`.
    let problem = commit_expr_problem("subject == now()").unwrap();
    assert!(problem.contains("expected a literal"), "{problem}");
    assert!(!problem.contains("reads a task"), "{problem}");
}

#[test]
fn a_comparison_without_two_operands_is_refused() {
    for bad in ["== 1", "1 ==", "subject ==", "(==)"] {
        let problem = commit_expr_problem(bad).unwrap();
        assert!(
            problem.contains("comparison needs two operands"),
            "{bad}: {problem}"
        );
    }
}
