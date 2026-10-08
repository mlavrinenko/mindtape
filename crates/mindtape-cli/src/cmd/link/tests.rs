//! Tests for `mt link`'s two forms end to end, over an in-process project:
//! argument parsing, the write, `--replace`, `--dry-run`, and the exit status
//! a per-entry failure carries.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use clap::Parser;

use super::{LinkArgs, change_summary, run_with};
use crate::cmd::link::plan::Staged;
use crate::util::Project;
use mindtape::test_support::bare_task;

use mindtape::test_support::PRELUDE;

const ALPHA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

const BETA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Beta\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

const GAMMA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Gamma\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

const CONFIG: &str = "[[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// Wrapper so `LinkArgs` can be parsed on its own, without the whole `Cli`.
#[derive(Parser)]
#[command(name = "link", no_binary_name = true)]
struct Harness {
    #[command(flatten)]
    args: LinkArgs,
}

fn parse(argv: &[&str]) -> LinkArgs {
    Harness::try_parse_from(argv).unwrap().args
}

fn setup(dir: &Path) -> Project {
    crate::test_util::make_project_with_lib(
        dir,
        PRELUDE,
        &[
            ("alpha.typ", ALPHA),
            ("beta.typ", BETA),
            ("gamma.typ", GAMMA),
        ],
        CONFIG,
    )
}

fn reload(dir: &Path) -> Project {
    let config = mindtape::parse_config(CONFIG).unwrap();
    let outcome = mindtape::load_project(dir, &config);
    Project {
        root: dir.to_path_buf(),
        config,
        outcome,
    }
}

fn source(dir: &Path, stem: &str) -> String {
    std::fs::read_to_string(dir.join("tasks").join(format!("{stem}.typ"))).unwrap()
}

#[test]
fn a_single_edge_lands() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    run_with(&project, &parse(&["alpha", "depends-on", "beta"])).unwrap();
    assert!(
        source(dir.path(), "alpha").contains(r#"depends-on("beta.typ")"#),
        "{}",
        source(dir.path(), "alpha")
    );
}

#[test]
fn several_targets_land_in_one_call() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    run_with(&project, &parse(&["alpha", "depends-on", "beta", "gamma"])).unwrap();
    assert!(
        source(dir.path(), "alpha").contains(r#"depends-on("beta.typ", "gamma.typ")"#),
        "{}",
        source(dir.path(), "alpha")
    );
}

/// The property the whole command is built around: an adjacency map is safe to
/// re-apply as it grows, so a second run over the same input writes nothing.
#[test]
fn a_second_identical_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    let args = parse(&["alpha", "depends-on", "beta"]);
    run_with(&project, &args).unwrap();
    let after_first = source(dir.path(), "alpha");

    run_with(&reload(dir.path()), &args).unwrap();
    assert_eq!(source(dir.path(), "alpha"), after_first);
}

#[test]
fn merging_keeps_the_edges_already_there() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    run_with(&project, &parse(&["alpha", "depends-on", "beta"])).unwrap();
    run_with(
        &reload(dir.path()),
        &parse(&["alpha", "related", "gamma", "-m", "same theme"]),
    )
    .unwrap();
    let out = source(dir.path(), "alpha");
    assert!(out.contains(r#"depends-on("beta.typ")"#), "{out}");
    assert!(out.contains(r#"related("gamma.typ")[same theme]"#), "{out}");
}

#[test]
fn dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    let before = source(dir.path(), "alpha");
    run_with(
        &project,
        &parse(&["alpha", "depends-on", "beta", "--dry-run"]),
    )
    .unwrap();
    assert_eq!(source(dir.path(), "alpha"), before);
}

#[test]
fn an_unknown_kind_fails_the_run_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    let before = source(dir.path(), "alpha");
    let err = run_with(&project, &parse(&["alpha", "depends-onn", "beta"])).unwrap_err();
    let err = crate::diag::plain(&err);
    assert!(
        err.starts_with("error: tasks/alpha.typ: unknown link kind: 'depends-onn'\n"),
        "{err}"
    );
    assert!(
        !err.contains("had problems"),
        "a single target gets no summary: {err}"
    );
    assert_eq!(source(dir.path(), "alpha"), before);
}

#[test]
fn a_missing_target_fails_the_run_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    let before = source(dir.path(), "alpha");
    assert!(run_with(&project, &parse(&["alpha", "depends-on", "nope"])).is_err());
    assert_eq!(source(dir.path(), "alpha"), before);
}

#[test]
fn related_without_a_note_fails_the_run() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    assert!(run_with(&project, &parse(&["alpha", "related", "beta"])).is_err());
}

#[test]
fn apply_reads_an_adjacency_map_from_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    let map = dir.path().join("links.yaml");
    std::fs::write(
        &map,
        "alpha:\n  depends-on: [beta, gamma]\nbeta:\n  related:\n    - target: gamma\n      note: pair\n",
    )
    .unwrap();
    run_with(&project, &parse(&["--apply", map.to_str().unwrap()])).unwrap();
    assert!(
        source(dir.path(), "alpha").contains(r#"depends-on("beta.typ", "gamma.typ")"#),
        "{}",
        source(dir.path(), "alpha")
    );
    assert!(
        source(dir.path(), "beta").contains(r#"related("gamma.typ")[pair]"#),
        "{}",
        source(dir.path(), "beta")
    );
}

/// The rm-style contract: one bad entry is reported and the run exits
/// non-zero, but every other entry still lands.
#[test]
fn a_failing_entry_does_not_hold_back_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    let map = dir.path().join("links.yaml");
    std::fs::write(
        &map,
        "alpha:\n  nonsense: [beta]\nbeta:\n  depends-on: [gamma]\n",
    )
    .unwrap();
    let err = run_with(&project, &parse(&["--apply", map.to_str().unwrap()])).unwrap_err();
    assert_eq!(
        crate::diag::plain(&err),
        "error: linked 1 of 2 tasks; 1 failed\n"
    );
    assert!(!source(dir.path(), "alpha").contains("links:"));
    assert!(source(dir.path(), "beta").contains(r#"depends-on("gamma.typ")"#));
}

#[test]
fn replace_makes_the_map_the_complete_edge_set() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    run_with(&project, &parse(&["alpha", "depends-on", "beta"])).unwrap();

    let map = dir.path().join("links.yaml");
    std::fs::write(&map, "alpha:\n  parent: gamma\n").unwrap();
    run_with(
        &reload(dir.path()),
        &parse(&["--apply", map.to_str().unwrap(), "--replace"]),
    )
    .unwrap();

    let out = source(dir.path(), "alpha");
    assert!(out.contains(r#"parent("gamma.typ")"#), "{out}");
    assert!(!out.contains("depends-on"), "{out}");
}

#[test]
fn replace_requires_apply() {
    // `--replace` on the positional form has no complete set to be about: the
    // verb upserts one edge, so silently clearing the others would be a
    // different command wearing the same name.
    assert!(Harness::try_parse_from(["alpha", "depends-on", "beta", "--replace"]).is_err());
}

#[test]
fn apply_and_a_positional_task_are_mutually_exclusive() {
    assert!(Harness::try_parse_from(["alpha", "depends-on", "beta", "--apply", "m.yaml"]).is_err());
}

#[test]
fn the_positional_form_needs_a_kind() {
    assert!(Harness::try_parse_from(["alpha"]).is_err());
}

/// A map carries a note per target, so one `-m` could not say which target it
/// meant. Refused rather than silently ignored.
#[test]
fn a_note_flag_and_apply_are_mutually_exclusive() {
    assert!(Harness::try_parse_from(["--apply", "m.yaml", "-m", "why"]).is_err());
}

#[test]
fn an_empty_map_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    let map = dir.path().join("links.yaml");
    std::fs::write(&map, "").unwrap();
    run_with(&project, &parse(&["--apply", map.to_str().unwrap()])).unwrap();
}

/// A `--replace` that drops edges says so; a change that drops none does not.
/// Inverting the `removed == 0` test would move the "(dropping N)" note to
/// exactly the runs with nothing to drop.
#[test]
fn change_summary_names_dropped_edges_only_when_some_drop() {
    let mut staged = Staged {
        stem: "alpha".to_owned(),
        abs: std::path::PathBuf::from("/tmp/alpha.typ"),
        task: bare_task("alpha"),
        updated: String::new(),
        added: Vec::new(),
        removed: 0,
    };
    assert!(
        !change_summary(&staged).contains("dropping"),
        "no drop, no note: {}",
        change_summary(&staged)
    );

    staged.removed = 2;
    let summary = change_summary(&staged);
    assert!(summary.contains("dropping 2 edge"), "{summary}");
}

const WRITTEN_CHILD: &str = "_MT_LINK_WRITTEN_CHILD";

/// Each landed task is reported as `linked <stem>: ...`. Replacing the report
/// with `()` would write the file silently.
#[test]
fn run_with_reports_each_written_task() {
    if std::env::var(WRITTEN_CHILD).is_err() {
        let out =
            crate::test_util::run_self_test("run_with_reports_each_written_task", WRITTEN_CHILD);
        crate::test_util::assert_child_ok(&out);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("linked alpha"),
            "a landed write must be reported: {stdout}"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    run_with(&project, &parse(&["alpha", "depends-on", "beta"])).unwrap();
}

/// Once a batch lands, the post-write pass reloads the config from disk; a
/// reload that fails is surfaced rather than swallowed. Inverting the
/// emptiness guard would skip that pass for exactly the runs that wrote
/// something.
#[test]
fn a_landed_batch_runs_the_post_write_reload() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    // Break the on-disk config after the project was loaded, so only the
    // post-write reload can notice.
    std::fs::write(dir.path().join(".mindtape/config.toml"), "[[[not toml\n").unwrap();
    let result = run_with(&project, &parse(&["alpha", "depends-on", "beta"]));
    assert!(
        result.is_err(),
        "the post-write reload must run after a landed batch: {result:?}"
    );
}

/// A written edge is checked against the whole project after the batch lands,
/// so a rule only the reloaded set can see — a `depends-on` cycle — is still
/// reported rather than left for the next `mt check`.
#[test]
fn a_cycle_created_by_the_batch_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path());
    let map = dir.path().join("links.yaml");
    std::fs::write(
        &map,
        "alpha:\n  depends-on: [beta]\nbeta:\n  depends-on: [alpha]\n",
    )
    .unwrap();
    let err = run_with(&project, &parse(&["--apply", map.to_str().unwrap()])).unwrap_err();
    assert_eq!(err.to_string(), "linked 1 of 2 tasks; 1 failed");
}
