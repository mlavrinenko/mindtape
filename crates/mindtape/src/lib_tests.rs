#![allow(clippy::unwrap_used)]

use crate::config::Config;
use crate::filter::Filter;
use crate::ready::Readiness;
use crate::select;
use crate::test_support::bare_task;

#[test]
fn no_filter_keeps_all_tasks_unchanged() {
    let tasks = vec![bare_task("a"), bare_task("b")];
    let kept = select(tasks.clone(), None, &Config::default(), &Readiness::empty()).unwrap();
    assert_eq!(kept, tasks);
}

#[test]
fn filter_narrows_to_matching_tasks() {
    let tasks = vec![bare_task("a"), bare_task("b")];
    let filter = Filter::parse("id == \"a\"").unwrap();
    let kept = select(
        tasks,
        Some(&filter),
        &Config::default(),
        &Readiness::empty(),
    )
    .unwrap();
    let ids: Vec<_> = kept.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["a"]);
}
