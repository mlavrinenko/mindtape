#![allow(clippy::unwrap_used)]

use super::{CommitExpr, commit_expr_problem};
use crate::commit::CommitRecord;
use crate::config::Config;
use crate::parse_config;

fn commit() -> CommitRecord {
    CommitRecord {
        subject: "docs: fix a typo".to_owned(),
        refs: vec!["parser".to_owned()],
        files: vec!["docs/guide.md".to_owned(), "src/lib.rs".to_owned()],
        renamed_from: Vec::new(),
        tasks_only: false,
        in_range: false,
        tasks: Vec::new(),
        trailers: vec!["Changelog: none".to_owned(), "Refs: parser".to_owned()],
    }
}

fn eval(expr: &str) -> bool {
    CommitExpr::parse(expr)
        .unwrap()
        .matches(&commit(), &Config::default())
        .unwrap()
}

fn problem_of(expr: &str) -> Option<String> {
    commit_expr_problem(expr, &Config::default()).map(|problem| problem.text)
}

/// `src/old.rs` renamed to `src/new.rs`, beside an edit to `docs/a.md`.
fn renamed() -> CommitRecord {
    CommitRecord {
        files: vec![
            "docs/a.md".to_owned(),
            "src/new.rs".to_owned(),
            "src/old.rs".to_owned(),
        ],
        renamed_from: vec!["src/old.rs".to_owned()],
        ..CommitRecord::default()
    }
}

#[test]
fn trailers_are_read_as_whole_lines() {
    assert!(eval("matches(trailers, \"^Changelog: none$\")"));
    assert!(!eval("matches(trailers, \"^Changelog: yes\")"));
    assert!(eval("has(trailers)"));
    assert!(
        CommitExpr::parse("miss(trailers)")
            .unwrap()
            .matches(&CommitRecord::default(), &Config::default())
            .unwrap()
    );
}

#[test]
fn a_changelog_rule_reads_files_and_trailers_together() {
    let expr = "!matches(files, \"^crates/\") || matches(files, \"^CHANGELOG[.]md$\") \
        || matches(trailers, \"^Changelog: none$\")";
    assert!(eval(expr));
}

#[test]
fn count_files_counts_a_rename_once_as_its_destination() {
    let holds = |expr: &str| {
        CommitExpr::parse(expr)
            .unwrap()
            .matches(&renamed(), &Config::default())
            .unwrap()
    };
    assert!(holds("count(files, \"src/**\") == 1"));
    assert!(holds("count(files) == 2"));
    // The source side no longer counts, so a rename into an excluded path is free.
    assert!(holds("count(files, \"!src/new.rs\") == 1"));
    // `has` and `matches` still see both sides.
    assert!(holds("matches(files, \"^src/old[.]rs$\")"));
}

#[test]
fn a_named_path_set_is_read_where_a_glob_is() {
    let config = parse_config(
        "[dicts.source]\ncase = \"exact\"\nvalues = [\"src/**\", \"!**/*test*.rs\"]\n",
    )
    .unwrap();
    let holds = |expr: &str, commit: &CommitRecord| {
        CommitExpr::parse(expr)
            .unwrap()
            .matches(commit, &config)
            .unwrap()
    };
    assert!(holds("count(files, \"@source\") == 1", &renamed()));
    assert!(holds("count(files, \"@source\") == 1", &commit()));
    assert!(commit_expr_problem("count(files, \"@nosuch\") == 1", &config).is_some());
    let folded = parse_config("[dicts.source]\nvalues = [\"src/**\"]\n").unwrap();
    let problem = commit_expr_problem("count(files, \"@source\") == 1", &folded).unwrap();
    assert!(problem.contains("exact"), "{problem}");
}

#[test]
fn in_range_is_true_only_for_a_commit_read_from_a_range() {
    let eval_in = |expr: &str, in_range: bool| {
        let record = CommitRecord {
            in_range,
            ..commit()
        };
        CommitExpr::parse(expr)
            .unwrap()
            .matches(&record, &Config::default())
            .unwrap()
    };
    assert!(eval_in("in_range", true) && eval_in("in_range()", true));
    assert!(!eval_in("in_range", false) && !eval_in("in_range()", false));
    assert!(eval_in("!in_range", false));
    assert!(problem_of("in_range").is_none());
    let refused = problem_of("status == \"wip\"").unwrap();
    assert!(refused.contains("tasks_only, in_range;"), "{refused}");
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
        assert!(problem_of(expr).is_some(), "{expr}");
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
    let holds = |expr: &str| {
        CommitExpr::parse(expr)
            .unwrap()
            .matches(&empty, &Config::default())
            .unwrap()
    };
    assert!(holds("miss(refs) && miss(files) && miss(subject)"));
}

#[test]
fn a_task_field_is_refused_naming_what_a_commit_rule_reads() {
    let problem = problem_of("status == \"wip\"").unwrap();
    assert!(problem.contains("status"), "{problem}");
    assert!(
        problem.contains("subject, refs, files, trailers, tasks_only"),
        "{problem}"
    );
    assert!(problem_of("terminal").is_some());
    assert!(problem_of("has(title)").is_some());
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
        assert!(problem_of(bad).is_some(), "{bad} should be refused");
    }
    assert!(problem_of("!tasks_only || has(refs)").is_none());
}

#[test]
fn a_non_state_function_side_is_a_literal_not_a_state_wrapper() {
    // `now()` is not one of the state wrappers, so a comparison side treats it
    // as a written-out value and refuses it there — it is never handed to a
    // task scope as if it were `head`/`staged`/`worktree`.
    let problem = problem_of("subject == now()").unwrap();
    assert!(problem.contains("expected a literal"), "{problem}");
    assert!(!problem.contains("reads a task"), "{problem}");
}

#[test]
fn a_comparison_without_two_operands_is_refused() {
    for bad in ["== 1", "1 ==", "subject ==", "(==)"] {
        let problem = problem_of(bad).unwrap();
        assert!(problem.contains("incomplete"), "{bad}: {problem}");
    }
}
