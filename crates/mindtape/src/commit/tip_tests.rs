#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::check_tip;
use crate::commit::tasks::ProjectState;
use crate::config::{Config, parse_config};
use crate::model::Status;
use crate::test_support::bare_task;

const RULE: &str = "[[commits.rules]]\nname = \"finish\"\nscope = \"tip\"\n\
     assert = '!(status == \"wip\")'\nmessage = \"run `mt done`\"\nseverity = \"error\"\n";

fn state(config: &Config, tasks: &[(&str, &str)]) -> ProjectState {
    let tasks = tasks
        .iter()
        .map(|(id, status)| {
            let mut task = bare_task(id);
            task.status = Some(Status {
                name: (*status).to_owned(),
                date: None,
                note: None,
            });
            task
        })
        .collect();
    ProjectState::new(tasks, config, std::path::Path::new(""), Vec::new())
}

#[test]
fn a_named_task_left_wip_breaches_with_its_stem() {
    let config = parse_config(RULE).unwrap();
    let tip = state(&config, &[("parser", "wip")]);
    let breaches = check_tip(["parser"], &tip, &config);
    assert_eq!(breaches.len(), 1);
    assert_eq!(breaches[0].task.as_deref(), Some("parser"));
    assert_eq!(breaches[0].label, "commits.rules.finish");
    assert_eq!(breaches[0].message, "run `mt done`");
}

#[test]
fn a_named_task_that_is_done_passes() {
    let config = parse_config(RULE).unwrap();
    let tip = state(&config, &[("parser", "done")]);
    assert!(check_tip(["parser"], &tip, &config).is_empty());
}

#[test]
fn only_a_named_task_is_checked() {
    let config = parse_config(RULE).unwrap();
    let tip = state(&config, &[("parser", "wip"), ("lexer", "wip")]);
    let breaches = check_tip(["lexer"], &tip, &config);
    assert_eq!(breaches.len(), 1);
    assert_eq!(breaches[0].task.as_deref(), Some("lexer"));
}

#[test]
fn a_stem_named_twice_is_reported_once() {
    let config = parse_config(RULE).unwrap();
    let tip = state(&config, &[("parser", "wip")]);
    assert_eq!(check_tip(["parser", "parser"], &tip, &config).len(), 1);
}

#[test]
fn a_task_the_tip_no_longer_holds_is_skipped() {
    let config = parse_config(RULE).unwrap();
    let tip = state(&config, &[]);
    assert!(check_tip(["parser"], &tip, &config).is_empty());
}

#[test]
fn when_gates_which_named_tasks_the_rule_reads() {
    let config = parse_config(&format!("{RULE}when = 'priority > 5'\n")).unwrap();
    let tip = state(&config, &[("parser", "wip")]);
    assert!(
        check_tip(["parser"], &tip, &config).is_empty(),
        "a task without that priority is outside the rule"
    );
}

#[test]
fn rules_of_other_scopes_do_not_run_at_the_tip() {
    let config = parse_config(
        "[[commits.rules]]\nassert = 'has(refs)'\n\
         [[commits.rules]]\nscope = \"task\"\nassert = 'named'\n",
    )
    .unwrap();
    let tip = state(&config, &[("parser", "wip")]);
    assert!(check_tip(["parser"], &tip, &config).is_empty());
}

#[test]
fn a_rule_reading_the_working_tree_is_skipped() {
    let config =
        parse_config("[[commits.rules]]\nscope = \"tip\"\nassert = 'changed(\"staged\") == 0'\n")
            .unwrap();
    let tip = state(&config, &[("parser", "wip")]);
    assert!(check_tip(["parser"], &tip, &config).is_empty());
}
