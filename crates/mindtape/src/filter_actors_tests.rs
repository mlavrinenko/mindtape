//! `assignee` / `role` filter tests, in a child module so they share
//! `sample()` / `hit()` without pushing `filter_tests.rs` over the linecop cap.
#![allow(clippy::unwrap_used)]

use crate::config::Config;
use crate::config_actors::{Actor, ActorKind, Role};
use crate::model::Task;

use super::{Filter, sample};

fn actors_config() -> Config {
    Config {
        roles: vec![
            Role {
                name: "reviewer".to_owned(),
            },
            Role {
                name: "author".to_owned(),
            },
        ],
        actors: vec![
            Actor {
                name: "ada".to_owned(),
                kind: ActorKind::Human,
                roles: vec!["reviewer".to_owned(), "author".to_owned()],
            },
            Actor {
                name: "gpt".to_owned(),
                kind: ActorKind::Llm,
                roles: vec!["author".to_owned()],
            },
        ],
        ..Config::default()
    }
}

fn hit_with(expr: &str, task: &Task, config: &Config) -> bool {
    Filter::parse(expr).unwrap().matches(task, config).unwrap()
}

#[test]
fn assignee_compares_by_actor_name() {
    let config = actors_config();
    let mut task = sample();
    task.assignee = Some("ada".to_owned());
    assert!(hit_with("assignee == \"ada\"", &task, &config));
    assert!(!hit_with("assignee != \"ada\"", &task, &config));
    assert!(!hit_with("assignee == \"eve\"", &task, &config));
    assert!(hit_with("has(assignee)", &task, &config));
    assert!(hit_with("contains(\"ad\", assignee)", &task, &config));
}

#[test]
fn unset_assignee_is_missing() {
    let config = actors_config();
    let task = sample();
    assert!(!hit_with("assignee == \"ada\"", &task, &config));
    assert!(hit_with("assignee != \"ada\"", &task, &config));
    assert!(hit_with("miss(assignee)", &task, &config));
}

#[test]
fn role_resolves_the_assignees_declared_roles() {
    let config = actors_config();
    let mut task = sample();
    task.assignee = Some("ada".to_owned());
    assert!(hit_with("role == \"reviewer\"", &task, &config));
    assert!(hit_with("role == \"author\"", &task, &config));
    assert!(!hit_with("role == \"maintainer\"", &task, &config));
    assert!(hit_with("role != \"maintainer\"", &task, &config));
    task.assignee = Some("gpt".to_owned());
    assert!(hit_with("role == \"author\"", &task, &config));
    assert!(!hit_with("role == \"reviewer\"", &task, &config));
    assert!(hit_with("role != \"reviewer\"", &task, &config));
}

#[test]
fn role_is_missing_for_unassigned_or_undeclared_assignees() {
    let config = actors_config();
    let unassigned = sample();
    assert!(!hit_with("role == \"reviewer\"", &unassigned, &config));
    assert!(hit_with("role != \"reviewer\"", &unassigned, &config));
    let mut free_text = sample();
    free_text.assignee = Some("eve".to_owned());
    assert!(!hit_with("role == \"reviewer\"", &free_text, &config));
    assert!(hit_with("role != \"reviewer\"", &free_text, &config));
    let mut no_registry = sample();
    no_registry.assignee = Some("ada".to_owned());
    assert!(!hit_with(
        "role == \"reviewer\"",
        &no_registry,
        &Config::default()
    ));
}
