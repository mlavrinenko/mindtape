#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::{Candidate, check_candidate, harvest_candidate};
use crate::config::Config;
use crate::loader::load_project;
use crate::parse_config;

/// The embedded floor config — `Config::default()` carries an empty status
/// legend, under which no status name is out of bounds and half these tests
/// would assert nothing.
fn floor() -> Config {
    parse_config("").unwrap()
}

/// A task marker source with the given status name, written the way the loader
/// tests do — a bare `#metadata(...)`, so no `@local/mindtape` package has to
/// be installed for these to run.
fn marker(status: &str) -> String {
    format!(
        "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Build\",
    status: (name: {status:?}, date: datetime(year: 2026, month: 6, day: 1), note: none),
  ),
  extras: (:),
))
"
    )
}

fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    crate::test_support::write_bare_project(dir.path());
    dir
}

/// The whole point: a candidate is evaluated and validated with nothing on
/// disk at its destination, and evaluating it puts nothing there either.
#[test]
fn checks_a_candidate_without_creating_its_file() {
    let dir = setup();
    let rel = Path::new("tasks/build.typ");
    let source = marker("wip");

    let violations = check_candidate(
        dir.path(),
        &floor(),
        &[],
        Candidate {
            rel,
            source: &source,
        },
    )
    .unwrap();

    assert!(violations.is_empty(), "{violations:?}");
    assert!(
        !dir.path().join(rel).exists(),
        "checking a candidate must not write it"
    );
}

/// A status outside the legend is caught before the write, and the violation
/// names the candidate — the case that used to leave a broken file behind.
#[test]
fn reports_a_violation_against_the_unwritten_candidate() {
    let dir = setup();
    let rel = Path::new("tasks/build.typ");
    let source = marker("bogus");

    let violations = check_candidate(
        dir.path(),
        &floor(),
        &[],
        Candidate {
            rel,
            source: &source,
        },
    )
    .unwrap();

    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].task_id, "build");
    assert_eq!(violations[0].field, "status");
    assert!(!dir.path().join(rel).exists());
}

/// Violations on the tasks already on disk are the project's problem, not this
/// write's: only the candidate's own are returned.
#[test]
fn filters_out_violations_of_the_existing_tasks() {
    let dir = setup();
    std::fs::write(dir.path().join("tasks/old.typ"), marker("bogus")).unwrap();
    let config = floor();
    let existing = load_project(dir.path(), &config).tasks;
    assert_eq!(existing.len(), 1);

    let source = marker("wip");
    let violations = check_candidate(
        dir.path(),
        &config,
        &existing,
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: &source,
        },
    )
    .unwrap();

    assert!(violations.is_empty(), "{violations:?}");
}

/// The candidate is evaluated *at its destination*, so the folder-derived tags
/// and nesting depth are the ones the written file would carry — not the ones
/// of wherever a staging copy might have sat.
#[test]
fn derives_tags_and_depth_from_the_destination_path() {
    let dir = setup();
    let config = parse_config(
        r#"
[[tasks.roots]]
path = "tasks"
name = "mt"
derive-tags = true

[tasks.derive-tags]
collect = ["segments"]
"#,
    )
    .unwrap();
    let source = marker("wip");

    let task = harvest_candidate(
        dir.path(),
        &config,
        Candidate {
            rel: Path::new("tasks/backend/auth/login.typ"),
            source: &source,
        },
    )
    .unwrap();

    assert_eq!(task.id, "login");
    assert_eq!(
        task.derived_tags,
        vec!["mt".to_owned(), "backend".to_owned(), "auth".to_owned()]
    );
    assert_eq!(task.depth, 2);
}

/// A source that evaluates but emits no task marker is a source failure, and
/// says so — there is no file to blame it on.
#[test]
fn errors_when_the_source_harvests_no_marker() {
    let dir = setup();
    let err = harvest_candidate(
        dir.path(),
        &floor(),
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: "= Just notes\n",
        },
    )
    .unwrap_err();

    assert_eq!(err.file, Path::new("tasks/build.typ"));
    assert!(err.message.contains("no task marker"), "{}", err.message);
}

/// A source Typst cannot evaluate is reported as a load error rather than
/// panicking or being mistaken for a clean parse.
#[test]
fn errors_when_the_source_fails_to_evaluate() {
    let dir = setup();
    let err = harvest_candidate(
        dir.path(),
        &floor(),
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: "#import \"@local/nope:9.9.9\": *\n",
        },
    )
    .unwrap_err();

    assert_eq!(err.file, Path::new("tasks/build.typ"));
    assert!(!err.message.is_empty());
}

/// A candidate overwriting a task already at its path is checked as a rewrite:
/// the old copy drops out, so the two never collide on the shared id.
#[test]
fn replaces_an_existing_task_at_the_same_path() {
    let dir = setup();
    std::fs::write(dir.path().join("tasks/build.typ"), marker("bogus")).unwrap();
    let config = floor();
    let existing = load_project(dir.path(), &config).tasks;

    let source = marker("wip");
    let violations = check_candidate(
        dir.path(),
        &config,
        &existing,
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: &source,
        },
    )
    .unwrap();

    assert!(violations.is_empty(), "{violations:?}");
}
