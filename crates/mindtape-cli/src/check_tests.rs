#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::{Config, LoadOutcome, load_project, parse_config};

use crate::util;

const TASK: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

fn make_project(dir: &Path, files: &[(&str, &str)]) -> util::Project {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
    for (name, body) in files {
        std::fs::write(dir.join("tasks").join(name), body).unwrap();
    }
    let config = Config::default();
    let outcome = load_project(dir, &config);
    util::Project {
        root: dir.to_path_buf(),
        config,
        outcome,
    }
}

fn make_project_with_config(dir: &Path, files: &[(&str, &str)], config: Config) -> util::Project {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
    for (name, body) in files {
        std::fs::write(dir.join("tasks").join(name), body).unwrap();
    }
    let outcome = load_project(dir, &config);
    util::Project {
        root: dir.to_path_buf(),
        config,
        outcome,
    }
}

#[test]
fn valid_project_returns_ok() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("x.typ", TASK)]);
    assert!(super::run_with(&project).is_ok());
}

#[test]
fn empty_project_returns_ok() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[]);
    assert!(super::run_with(&project).is_ok());
}

#[test]
fn load_error_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(
        dir.path(),
        &[("broken.typ", "#import \"@local/nope:9.9.9\": *\n")],
    );
    assert!(super::run_with(&project).is_err());
}

#[test]
fn validation_violation_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n[tasks.priority]\nrequired = true",
    )
    .unwrap();
    // Task has no priority field; config requires it.
    let body = "#metadata((
      marker: \"mindtape.task\",
      fields: (title: \"Y\", status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none)),
      extras: (:),
    ))
    ";
    let project = make_project_with_config(dir.path(), &[("y.typ", body)], config);
    assert!(super::run_with(&project).is_err());
}

#[test]
fn multiple_valid_tasks_reports_count() {
    let dir = tempfile::tempdir().unwrap();
    let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
    let outcome = LoadOutcome {
        tasks: project.outcome.tasks.clone(),
        errors: vec![],
    };
    // Confirm two tasks loaded successfully before asserting Ok.
    assert_eq!(outcome.tasks.len(), 2);
    assert!(super::run_with(&project).is_ok());
}

const TASK_WITH_SUMMARY: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))

= Summary

Some content here.
";

#[test]
fn header_require_rule_passes_when_heading_present() {
    let dir = tempfile::tempdir().unwrap();
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK_WITH_SUMMARY)], config);
    assert!(super::run_with(&project).is_ok());
}

#[test]
fn header_require_rule_fails_when_heading_absent() {
    let dir = tempfile::tempdir().unwrap();
    let config =
        parse_config("[[tasks.body.headers.rules]]\npath = [\"Summary\"]\naction = \"require\"\n")
            .unwrap();
    // TASK has no headings at all
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK)], config);
    assert!(super::run_with(&project).is_err());
}

#[test]
fn invalid_header_rule_regex_fails_with_config_error() {
    let dir = tempfile::tempdir().unwrap();
    let config = parse_config(
        "[[tasks.body.headers.rules]]\npath = [{ match = \"[bad\" }]\naction = \"forbid\"\n",
    )
    .unwrap();
    let project = make_project_with_config(dir.path(), &[("a.typ", TASK)], config);
    assert!(super::run_with(&project).is_err());
}
