//! `count(<expr>)`: over the whole loaded set whatever narrows the listing,
//! as the left operand of a comparison, once per command, and refused where
//! no loaded set exists.
#![allow(clippy::unwrap_used)]

use crate::config::Config;
use crate::filter::{Filter, FilterError, expr_problem, expr_problem_in_set};
use crate::graph::ready::Readiness;
use crate::model::Task;
use crate::test_support::{dep_config, dep_task};

/// Two tasks in flight, one finished.
fn set() -> Vec<Task> {
    vec![
        dep_task("a", "wip", &[]),
        dep_task("b", "wip", &[]),
        dep_task("c", "done", &[]),
    ]
}

fn config() -> Config {
    dep_config("")
}

/// The ids `expr` keeps out of `tasks`, with the set context built over
/// `tasks` itself.
fn kept(expr: &str, tasks: &[Task], cfg: &Config) -> Vec<String> {
    let readiness = Readiness::compute(tasks, cfg);
    let filter = Filter::parse(expr).unwrap();
    crate::select(tasks.to_vec(), Some(&filter), cfg, &readiness)
        .unwrap()
        .into_iter()
        .map(|task| task.id)
        .collect()
}

#[test]
fn count_ranges_over_the_whole_set_under_a_narrowing_filter() {
    let (tasks, cfg) = (set(), config());
    // The outer filter keeps only `c`; the count still sees `a` and `b`.
    assert_eq!(
        kept(
            r#"status == "done" && count(status == "wip") == 2"#,
            &tasks,
            &cfg
        ),
        ["c"]
    );
    // A listing's rows are never the set: narrowing the input to `c` first
    // and counting over it would read 0, the set context reads 2.
    let readiness = Readiness::compute(&tasks, &cfg);
    let filter = Filter::parse(r#"count(status == "wip") == 2"#).unwrap();
    let narrowed = vec![tasks.last().unwrap().clone()];
    let rows = crate::select(narrowed, Some(&filter), &cfg, &readiness).unwrap();
    assert_eq!(rows.len(), 1);
}

#[test]
fn count_compares_numerically_inside_boolean_expressions() {
    let (tasks, cfg) = (set(), config());
    assert!(kept(r#"count(status == "wip") <= 1"#, &tasks, &cfg).is_empty());
    assert_eq!(kept(r#"count(status == "wip") > 1"#, &tasks, &cfg).len(), 3);
    assert_eq!(
        kept(
            r#"status == "wip" && !(count(terminal) >= 2)"#,
            &tasks,
            &cfg
        ),
        ["a", "b"]
    );
    // An empty match counts zero, not missing.
    assert_eq!(kept(r#"count(id == "gone") == 0"#, &tasks, &cfg).len(), 3);
}

#[test]
fn count_reads_every_field_off_the_counted_task() {
    let (tasks, cfg) = (set(), config());
    // Each outer task would give a different answer if the inner expression
    // read it; every one reads the same 1.
    assert_eq!(kept(r#"count(id == "a") == 1"#, &tasks, &cfg).len(), 3);
}

#[test]
fn count_nests() {
    let (tasks, cfg) = (set(), config());
    // Every task sees two in flight, so the outer count is all three.
    assert_eq!(
        kept(r#"count(count(status == "wip") == 2) == 3"#, &tasks, &cfg).len(),
        3
    );
}

#[test]
fn count_is_computed_once_per_command() {
    let (tasks, cfg) = (set(), config());
    let readiness = Readiness::compute(&tasks, &cfg);
    let wip = Filter::parse(r#"count(status == "wip") <= 1"#).unwrap();
    let again = Filter::parse(r#"has_tag("x") || count(status == "wip") > 0"#).unwrap();
    for task in &tasks {
        wip.matches_with(task, &cfg, &readiness).unwrap();
        again.matches_with(task, &cfg, &readiness).unwrap();
    }
    // Six evaluations, one distinct argument: one pass over the set.
    assert_eq!(readiness.set().passes(), 1);
    let other = Filter::parse("count(terminal) == 1").unwrap();
    other
        .matches_with(tasks.first().unwrap(), &cfg, &readiness)
        .unwrap();
    assert_eq!(readiness.set().passes(), 2);
}

#[test]
fn count_without_a_loaded_set_is_an_error() {
    let (tasks, cfg) = (set(), config());
    let filter = Filter::parse(r#"count(status == "wip") <= 1"#).unwrap();
    let err = filter.matches(tasks.first().unwrap(), &cfg).unwrap_err();
    assert!(err.to_string().contains("reads every task"), "{err}");
}

#[test]
fn count_is_a_number_not_a_predicate() {
    let (tasks, cfg) = (set(), config());
    let readiness = Readiness::compute(&tasks, &cfg);
    for expr in [
        r#"count(status == "wip")"#,
        "count()",
        "count(terminal, ready)",
    ] {
        let err = Filter::parse(expr)
            .unwrap()
            .matches_with(tasks.first().unwrap(), &cfg, &readiness)
            .unwrap_err();
        assert!(
            matches!(err, FilterError::Invalid(_) | FilterError::Call { .. }),
            "{expr}: {err:?}"
        );
    }
}

#[test]
fn load_time_dry_run_accepts_count_only_where_a_set_exists() {
    let (probe, cfg) = (Task::default(), config());
    let source = r#"count(status == "wip") <= 1"#;
    assert_eq!(expr_problem_in_set(source, &probe, &cfg), None);
    assert!(expr_problem(source, &probe, &cfg).is_some());
    // The argument is still checked when the probe set has nothing to count.
    let unknown = expr_problem_in_set("count(nosuch == 1) <= 1", &probe, &cfg).unwrap();
    assert!(unknown.contains("nosuch"), "{unknown}");
}

#[test]
fn a_wip_cap_rule_flags_every_task_over_the_cap() {
    let cfg = crate::parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
         [[tasks.rules]]\nname = \"wip-cap\"\nwhen = 'status == \"wip\"'\n\
         assert = 'count(status == \"wip\") <= 1'\nseverity = \"error\"\n",
    )
    .unwrap();
    assert!(crate::rules::validate_rules(&cfg).is_empty());
    let mut out = Vec::new();
    crate::rules::check_all(&set(), &cfg, &mut out);
    let ids: Vec<&str> = out.iter().map(|v| v.task_id.as_str()).collect();
    assert_eq!(ids, ["a", "b"]);
    // A breach is the cap, never an evaluation error reported as one.
    assert!(
        out.iter().all(|v| v.message.starts_with("must satisfy")),
        "{out:?}"
    );
    let mut under = set();
    under.remove(1);
    let mut out = Vec::new();
    crate::rules::check_all(&under, &cfg, &mut out);
    assert!(out.is_empty(), "{out:?}");
}
