#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::{BodyRules, Candidate, check_candidate, check_candidates, harvest_candidate};
use crate::config::Config;
use crate::content_gate::compile_content_rules;
use crate::gate::compile_rules;
use crate::loader::load_project;
use crate::model::Task;
use crate::parse_config;
use crate::validate::Violation;

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
        BodyRules::default(),
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
        BodyRules::default(),
    )
    .unwrap();

    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].task_id, "build");
    assert_eq!(violations[0].field, "status");
    assert!(!dir.path().join(rel).exists());
}

/// [`check_candidate`] against `existing`, for a `"wip"` candidate at
/// `tasks/build.typ` under the floor config — the call
/// [`filters_out_violations_of_the_existing_tasks`] and
/// [`replaces_an_existing_task_at_the_same_path`] both make, against
/// different `existing` sets.
fn check_wip_candidate_for_build(dir: &Path, config: &Config, existing: &[Task]) -> Vec<Violation> {
    let source = marker("wip");
    check_candidate(
        dir,
        config,
        existing,
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: &source,
        },
        BodyRules::default(),
    )
    .unwrap()
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

    let violations = check_wip_candidate_for_build(dir.path(), &config, &existing);

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

/// A task marker with `depends-on` edges — the batch fixture, since an edge is
/// the cheapest rule that cannot be answered for one candidate alone.
fn linked(deps: &[&str]) -> String {
    let links: String = deps
        .iter()
        .map(|dep| format!("(task: {dep:?}, kind: \"depends-on\", note: none),"))
        .collect();
    format!(
        "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Build\",
    status: (name: \"wip\", date: none, note: none),
    links: ({links}),
  ),
  extras: (:),
))
"
    )
}

/// The reason a batch is one call and not a loop: `a` depends on `b`, and `b`
/// exists only as the other candidate. Together the edge resolves.
#[test]
fn resolves_an_edge_between_two_candidates_in_one_batch() {
    let dir = setup();
    let a = linked(&["b.typ"]);
    let b = linked(&[]);
    let batch = [
        Candidate {
            rel: Path::new("tasks/a.typ"),
            source: &a,
        },
        Candidate {
            rel: Path::new("tasks/b.typ"),
            source: &b,
        },
    ];

    let results = check_candidates(dir.path(), &floor(), &[], &batch, BodyRules::default());

    assert_eq!(results.len(), 2);
    let checked = results[0].as_ref().unwrap();
    assert_eq!(checked.task.id, "a");
    assert!(checked.violations.is_empty(), "{:?}", checked.violations);
}

/// The same candidate checked alone cannot see its peer — the negative half of
/// the pair above, so "checked together" is pinned as load-bearing rather than
/// merely true.
#[test]
fn an_edge_to_a_batch_peer_fails_when_checked_alone() {
    let dir = setup();
    let a = linked(&["b.typ"]);

    let violations = check_candidate(
        dir.path(),
        &floor(),
        &[],
        Candidate {
            rel: Path::new("tasks/a.typ"),
            source: &a,
        },
        BodyRules::default(),
    )
    .unwrap();

    assert!(!violations.is_empty());
}

/// One entry that does not evaluate is its own `Err` and does not cost the rest
/// their verdict: a batch reports per entry, so a writer can land the good ones.
#[test]
fn a_candidate_that_fails_to_evaluate_does_not_sink_the_batch() {
    let dir = setup();
    let good = marker("wip");
    let batch = [
        Candidate {
            rel: Path::new("tasks/broken.typ"),
            source: "= no marker here\n",
        },
        Candidate {
            rel: Path::new("tasks/good.typ"),
            source: &good,
        },
    ];

    let results = check_candidates(dir.path(), &floor(), &[], &batch, BodyRules::default());

    assert!(results[0].is_err());
    let checked = results[1].as_ref().unwrap();
    assert_eq!(checked.task.id, "good");
    assert!(checked.violations.is_empty(), "{:?}", checked.violations);
}

/// A candidate overwriting a task already at its path is checked as a rewrite:
/// the old copy drops out, so the two never collide on the shared id.
#[test]
fn replaces_an_existing_task_at_the_same_path() {
    let dir = setup();
    std::fs::write(dir.path().join("tasks/build.typ"), marker("bogus")).unwrap();
    let config = floor();
    let existing = load_project(dir.path(), &config).tasks;

    let violations = check_wip_candidate_for_build(dir.path(), &config, &existing);

    assert!(violations.is_empty(), "{violations:?}");
}

/// [`marker`], plus literal body text appended after the metadata call. The
/// header and content gates parse headings straight out of the raw source
/// text rather than the harvested marker, so a candidate exercising them
/// needs an actual `==` heading in its source.
fn marker_with_body(status: &str, body: &str) -> String {
    format!("{}{body}", marker(status))
}

/// The Problem this task exists to fix
/// (`truth/tasks/validate-body-rules-before-mt-add-writes-a-task.typ`): a
/// content rule `mt check` already enforces — this repo's own "every TO-BE
/// bullet opens with 🟢" — is broken by one candidate and satisfied by
/// another, so the candidate seam is proven to constrain bullets that violate
/// the rule rather than the section's mere presence. The refused candidate
/// gets the same field and message `mt check` would report, and writes
/// nothing.
#[test]
fn refuses_a_candidate_whose_to_be_bullet_breaks_the_content_rule() {
    let dir = setup();
    let config = parse_config(
        r#"[[tasks.body.content.rules]]
path = ["TO-BE"]
assert = { must-not-match = "(?m)^\\s*- [^🟢]" }
message = "open every TO-BE bullet with 🟢"
"#,
    )
    .unwrap();
    let content = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let body_rules = BodyRules {
        headers: &[],
        content: &content,
    };
    let rel = Path::new("tasks/build.typ");

    let bad = marker_with_body("wip", "\n== TO-BE\n\n- ships the feature\n");
    let violations = check_candidate(
        dir.path(),
        &config,
        &[],
        Candidate { rel, source: &bad },
        body_rules,
    )
    .unwrap();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].field, "body.content.rules[0]");
    assert_eq!(violations[0].message, "open every TO-BE bullet with 🟢");
    assert!(!dir.path().join(rel).exists());

    let good = marker_with_body("wip", "\n== TO-BE\n\n- 🟢 ships the feature\n");
    let violations = check_candidate(
        dir.path(),
        &config,
        &[],
        Candidate { rel, source: &good },
        body_rules,
    )
    .unwrap();
    assert!(violations.is_empty(), "{violations:?}");
}

/// A header-structure rule `mt check` already enforces (a required section) is
/// broken by a candidate carrying no such section, and the candidate seam
/// refuses it naming the field `mt check` would.
#[test]
fn refuses_a_candidate_missing_a_required_section() {
    let dir = setup();
    let config = parse_config(
        r#"[[tasks.body.headers.rules]]
path = ["Summary"]
action = "require"
message = "every task needs a Summary section"
"#,
    )
    .unwrap();
    let headers = compile_rules(&config.tasks.body.headers.rules).unwrap();
    let body_rules = BodyRules {
        headers: &headers,
        content: &[],
    };
    let source = marker_with_body("wip", "\n== Scope\n\nno summary here.\n");

    let violations = check_candidate(
        dir.path(),
        &config,
        &[],
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: &source,
        },
        body_rules,
    )
    .unwrap();

    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].field, "body.headers.rules[0]");
    assert_eq!(violations[0].message, "every task needs a Summary section");
}

/// A content-rule breach already on disk is the existing task's problem, not
/// this candidate's: body rules run over the candidate's own source only, the
/// same property [`filters_out_violations_of_the_existing_tasks`] pins for
/// [`crate::validate::validate`].
#[test]
fn does_not_leak_a_content_violation_from_an_existing_task() {
    let dir = setup();
    let config = parse_config(
        r#"[[tasks.body.content.rules]]
path = ["Summary"]
assert = "non-empty"
"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("tasks/old.typ"),
        marker_with_body("wip", "\n== Summary\n\n"),
    )
    .unwrap();
    let existing = load_project(dir.path(), &config).tasks;
    assert_eq!(existing.len(), 1);

    let content = compile_content_rules(&config.tasks.body.content.rules).unwrap();
    let body_rules = BodyRules {
        headers: &[],
        content: &content,
    };
    let source = marker_with_body("wip", "\n== Summary\n\nfine.\n");

    let violations = check_candidate(
        dir.path(),
        &config,
        &existing,
        Candidate {
            rel: Path::new("tasks/build.typ"),
            source: &source,
        },
        body_rules,
    )
    .unwrap();

    assert!(violations.is_empty(), "{violations:?}");
}
