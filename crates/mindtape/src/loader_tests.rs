#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::{find_root, load_project};
use crate::config::Config;
use crate::numeric::NumValue;
use crate::parse_config;

const TASK_BODY: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Build\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".mindtape")).unwrap();
    std::fs::create_dir(dir.path().join("tasks")).unwrap();
    dir
}

fn write(root: &Path, rel: &str, body: &str) {
    std::fs::write(root.join(rel), body).unwrap();
}

#[test]
fn loads_tasks_and_skips_non_tasks() {
    let dir = setup();
    write(dir.path(), "tasks/build.typ", TASK_BODY);
    write(
        dir.path(),
        "tasks/notes.typ",
        "= Just notes\n\nNo marker here.\n",
    );

    let outcome = load_project(dir.path(), &Config::default());
    assert_eq!(outcome.errors, Vec::new());
    assert_eq!(outcome.tasks.len(), 1);
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.id, "build");
    assert_eq!(task.title, "Build");
    assert_eq!(task.status.as_ref().unwrap().name, "wip");
    assert_eq!(task.priority, Some(NumValue::Num(8.0)));
}

#[test]
fn loads_multi_unit_estimate_verbatim() {
    let dir = setup();
    write(
        dir.path(),
        "tasks/est.typ",
        "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"E\", estimate: (days: 1, hours: 2)),
  extras: (:),
))
",
    );

    // Units are opaque labels: stored as-is, never converted.
    let outcome = load_project(dir.path(), &Config::default());
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.estimate.get("days"), Some(&1.0));
    assert_eq!(task.estimate.get("hours"), Some(&2.0));
}

#[test]
fn records_eval_errors() {
    let dir = setup();
    write(
        dir.path(),
        "tasks/broken.typ",
        "#import \"@local/nope:9.9.9\": *\n",
    );

    let outcome = load_project(dir.path(), &Config::default());
    assert!(outcome.tasks.is_empty());
    assert_eq!(outcome.errors.len(), 1);
    assert_eq!(
        outcome.errors.first().unwrap().file,
        Path::new("tasks/broken.typ")
    );
}

#[test]
fn injects_folder_derived_tags() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tasks/backend/auth")).unwrap();
    write(dir.path(), "tasks/backend/auth/login.typ", TASK_BODY);
    write(dir.path(), "tasks/top.typ", TASK_BODY);

    let config = parse_config(
        r#"
[[tasks.roots]]
path = "tasks"
name = "mt"
derive-tags = true

[tasks.derive-tags]
collect = ["segments", "breadcrumbs"]
"#,
    )
    .unwrap();
    let outcome = load_project(dir.path(), &config);

    let nested = outcome.tasks.iter().find(|t| t.id == "login").unwrap();
    assert_eq!(
        nested.derived_tags,
        vec![
            "mt".to_owned(),
            "backend".to_owned(),
            "auth".to_owned(),
            "backend:auth".to_owned(),
        ]
    );
    // Authored tags stay untouched; derived live separately.
    assert!(nested.tags.is_empty());

    let top = outcome.tasks.iter().find(|t| t.id == "top").unwrap();
    assert_eq!(top.derived_tags, vec!["mt".to_owned()]);
}

#[test]
fn glob_gates_which_files_derive() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tasks/backend")).unwrap();
    std::fs::create_dir_all(dir.path().join("tasks/frontend")).unwrap();
    write(dir.path(), "tasks/backend/a.typ", TASK_BODY);
    write(dir.path(), "tasks/frontend/b.typ", TASK_BODY);

    let config = parse_config(
        r#"
[[tasks.roots]]
path = "tasks"
name = "mt"
derive-tags = "backend/**"
"#,
    )
    .unwrap();
    let outcome = load_project(dir.path(), &config);

    let inside = outcome.tasks.iter().find(|t| t.id == "a").unwrap();
    assert_eq!(
        inside.derived_tags,
        vec!["mt".to_owned(), "backend".to_owned()]
    );
    let outside = outcome.tasks.iter().find(|t| t.id == "b").unwrap();
    assert!(outside.derived_tags.is_empty());
}

#[test]
fn find_root_locates_mindtape_dir() {
    let dir = setup();
    let child = dir.path().join("tasks");
    assert_eq!(find_root(&child).unwrap(), dir.path());
}

#[test]
fn find_root_returns_none_without_marker() {
    let dir = tempfile::tempdir().unwrap();
    assert!(find_root(dir.path()).is_none());
}

#[test]
fn explicit_id_wins_over_stem() {
    let dir = setup();
    write(
        dir.path(),
        "tasks/build.typ",
        "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"Build\", id: \"custom\"),
  extras: (:),
))
",
    );

    let outcome = load_project(dir.path(), &Config::default());
    assert_eq!(outcome.errors, Vec::new());
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.id, "custom", "explicit id: should win over stem");
}

#[test]
fn stem_used_when_id_absent() {
    let dir = setup();
    write(dir.path(), "tasks/my-task.typ", TASK_BODY);

    let outcome = load_project(dir.path(), &Config::default());
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.id, "my-task");
}

#[test]
fn duplicate_ids_produce_uniqueness_violation() {
    let dir = setup();
    // Two files, one with explicit id matching the other's stem.
    write(
        dir.path(),
        "tasks/alpha.typ",
        "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"Alpha\", id: \"shared\"),
  extras: (:),
))
",
    );
    write(
        dir.path(),
        "tasks/shared.typ",
        "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"Shared\"),
  extras: (:),
))
",
    );

    let outcome = load_project(dir.path(), &Config::default());
    let violations = crate::validate::validate(&outcome.tasks, &Config::default());
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "expected uniqueness violation on duplicate id"
    );
}
