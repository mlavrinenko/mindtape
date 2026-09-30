//! Tests for the pre-write checks: unknown kind, note rules, target
//! resolution — the three refusals that keep an invalid edge off disk.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::link::NoteRule;

use super::{build, note_problem};
use crate::link_edit::{EdgeSpec, Mode};
use crate::link_input::Entry;
use crate::util::Project;

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

const ALPHA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

const BETA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Beta\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

const BASE_CONFIG: &str = "[[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A project declaring a custom kind whose targets sit outside the backlog, so
/// the "no `.typ` appended, no stem resolution" path has something to exercise.
const EXTERNAL_KIND_CONFIG: &str = "[[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.links.kinds]]\nname = \"discharged-by\"\nvalidate = \"url\"\n";

fn setup(dir: &Path, config_text: &str) -> Project {
    crate::test_util::make_project_with_lib(
        dir,
        PRELUDE,
        &[("alpha.typ", ALPHA), ("beta.typ", BETA)],
        config_text,
    )
}

/// Re-evaluate an existing project directory without rewriting its task files
/// — what a second `mt link` invocation sees. [`setup`] would restore the
/// fixtures over the rewrite under test.
fn reload(dir: &Path, config_text: &str) -> Project {
    let config = mindtape::parse_config(config_text).unwrap();
    let outcome = mindtape::load_project(dir, &config);
    Project {
        root: dir.to_path_buf(),
        config,
        outcome,
    }
}

fn entry(stem: &str, kind: &str, target: &str, note: Option<&str>) -> Entry {
    Entry {
        stem: stem.to_owned(),
        edges: vec![EdgeSpec {
            kind: kind.to_owned(),
            target: target.to_owned(),
            note: note.map(ToOwned::to_owned),
        }],
    }
}

/// The single problem a one-entry plan reported, or a panic naming what it did
/// instead.
fn only_problem(plan: &super::Plan) -> &str {
    assert_eq!(
        plan.problems.len(),
        1,
        "expected exactly one problem, staged {} unchanged {}",
        plan.staged.len(),
        plan.unchanged.len()
    );
    plan.problems.first().unwrap().as_str()
}

#[test]
fn a_valid_edge_stages_a_rewrite() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[entry("alpha", "depends-on", "beta", None)],
        Mode::Merge,
    );
    assert!(plan.problems.is_empty(), "{:?}", plan.problems);
    assert_eq!(plan.staged.len(), 1);
    let staged = plan.staged.first().unwrap();
    assert!(
        staged.updated.contains(r#"depends-on("beta.typ")"#),
        "{}",
        staged.updated
    );
}

/// Nothing is written by planning: a staged rewrite lives in memory until the
/// command decides to land it, which is what makes `--dry-run` exact.
#[test]
fn planning_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let path = dir.path().join("tasks/alpha.typ");
    let before = std::fs::read_to_string(&path).unwrap();
    let _ = build(
        &project,
        &[entry("alpha", "depends-on", "beta", None)],
        Mode::Merge,
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn an_unknown_kind_is_refused_with_the_configured_list_and_a_suggestion() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[entry("alpha", "depends-onn", "beta", None)],
        Mode::Merge,
    );
    let problem = only_problem(&plan);
    assert!(problem.contains("not a configured link kind"), "{problem}");
    assert!(problem.contains("configured: depends-on"), "{problem}");
    assert!(problem.contains("did you mean: depends-on?"), "{problem}");
}

#[test]
fn a_custom_kind_from_config_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), EXTERNAL_KIND_CONFIG);
    let plan = build(
        &project,
        &[entry(
            "alpha",
            "discharged-by",
            "https://example.test/pull/42",
            None,
        )],
        Mode::Merge,
    );
    assert!(plan.problems.is_empty(), "{:?}", plan.problems);
    let staged = plan.staged.first().unwrap();
    // No `.typ` appended and no stem resolved: this target is not a task.
    assert!(
        staged
            .updated
            .contains(r#"edge("discharged-by", "https://example.test/pull/42")"#),
        "{}",
        staged.updated
    );
}

#[test]
fn related_without_a_note_is_refused_naming_the_edge() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[entry("alpha", "related", "beta", None)],
        Mode::Merge,
    );
    let problem = only_problem(&plan);
    assert!(
        problem.contains("`related` edge to `beta.typ`"),
        "{problem}"
    );
    assert!(problem.contains("requires a note"), "{problem}");
}

#[test]
fn related_with_a_note_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[entry("alpha", "related", "beta", Some("same recital"))],
        Mode::Merge,
    );
    assert!(plan.problems.is_empty(), "{:?}", plan.problems);
}

/// A blank note satisfies nothing: `render_edges` drops it, so accepting it
/// here would write exactly the file the note rule exists to reject.
#[test]
fn a_blank_note_does_not_satisfy_a_required_one() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[entry("alpha", "related", "beta", Some("   "))],
        Mode::Merge,
    );
    assert!(only_problem(&plan).contains("requires a note"));
}

#[test]
fn a_missing_target_is_refused_with_the_nearest_task() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[entry("alpha", "depends-on", "betta", None)],
        Mode::Merge,
    );
    let problem = only_problem(&plan);
    assert!(problem.contains("names no task"), "{problem}");
    assert!(problem.contains("did you mean: beta?"), "{problem}");
}

#[test]
fn a_missing_source_task_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[entry("nope", "depends-on", "beta", None)],
        Mode::Merge,
    );
    assert!(only_problem(&plan).starts_with("nope: "));
}

/// A per-entry failure is not a batch failure: the other entries still stage.
#[test]
fn one_bad_entry_does_not_stop_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[
            entry("alpha", "nonsense", "beta", None),
            entry("beta", "depends-on", "alpha", None),
        ],
        Mode::Merge,
    );
    assert_eq!(plan.problems.len(), 1);
    assert_eq!(plan.staged.len(), 1);
    assert_eq!(plan.staged.first().map(|s| s.stem.as_str()), Some("beta"));
}

/// Every refusal for one entry is collected, so a caller fixes them in one
/// pass rather than one re-run per problem.
#[test]
fn every_refusal_for_one_entry_is_reported_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let plan = build(
        &project,
        &[Entry {
            stem: "alpha".to_owned(),
            edges: vec![
                EdgeSpec {
                    kind: "nonsense".to_owned(),
                    target: "beta".to_owned(),
                    note: None,
                },
                EdgeSpec {
                    kind: "related".to_owned(),
                    target: "beta".to_owned(),
                    note: None,
                },
            ],
        }],
        Mode::Merge,
    );
    assert_eq!(plan.problems.len(), 2, "{:?}", plan.problems);
}

#[test]
fn an_edge_already_there_reports_unchanged_rather_than_staging() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), BASE_CONFIG);
    let entries = [entry("alpha", "depends-on", "beta", None)];
    let plan = build(&project, &entries, Mode::Merge);
    let staged = plan.staged.first().unwrap();
    std::fs::write(&staged.abs, &staged.updated).unwrap();

    // Reload, then ask for exactly the same edge again.
    let project = reload(dir.path(), BASE_CONFIG);
    let plan = build(&project, &entries, Mode::Merge);
    assert!(plan.staged.is_empty(), "a re-run must stage nothing");
    assert_eq!(plan.unchanged, vec!["alpha".to_owned()]);
}

// --- note_problem, Forbidden arm ---
//
// The `Forbidden if note.is_some()` guard decides both sides: a forbidden edge
// that carries a note is refused, and one without a note is accepted. A guard
// forced to `false` accepts the noted edge; forced to `true` refuses the bare
// one. Both directions are pinned.

#[test]
fn a_forbidden_edge_with_a_note_is_refused() {
    let spec = EdgeSpec {
        kind: "blocks".to_owned(),
        target: "beta.typ".to_owned(),
        note: Some("why".to_owned()),
    };
    let problem = note_problem(&spec, NoteRule::Forbidden).expect("a note is forbidden here");
    assert!(problem.contains("must not carry a note"), "{problem}");
}

#[test]
fn a_forbidden_edge_without_a_note_is_accepted() {
    let spec = EdgeSpec {
        kind: "blocks".to_owned(),
        target: "beta.typ".to_owned(),
        note: None,
    };
    assert!(note_problem(&spec, NoteRule::Forbidden).is_none());
}
