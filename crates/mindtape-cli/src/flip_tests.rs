//! Tests for the `flip` command: status rewriting, shortcut parsing, batch.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use clap::Parser;
use mindtape::{Config, parse_config};

use super::{FlipArgs, Shortcut, run_with, status_edit};
use crate::util::Project;

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

const ALPHA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

const BETA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Beta\",\n  status: blocked(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

fn legend_config() -> Config {
    parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"blocked\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap()
}

/// Project rooted at `dir` with an in-tree `lib/` mindtape package override (so
/// `@local/mindtape` resolves without a global install) and `files` under `tasks/`.
/// The lib is written before [`crate::test_util::make_project`] loads, so the
/// real prelude resolves the task `task.with(…)` sources.
fn setup(dir: &Path, files: &[(&str, &str)], config: Config) -> Project {
    let lib = dir.join("lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(
        lib.join("typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    std::fs::write(lib.join("prelude.typ"), PRELUDE).unwrap();
    crate::test_util::make_project(dir, files, config)
}

fn flip_args(status: &str, tasks: &[&str]) -> FlipArgs {
    FlipArgs {
        status: Some(status.to_owned()),
        tasks: tasks.iter().map(|s| (*s).to_owned()).collect(),
        note: None,
        list: false,
    }
}

fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join("tasks").join(name)).unwrap()
}

#[test]
fn flips_status_by_slug() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], legend_config());
    assert_eq!(project.outcome.tasks.len(), 1, "fixture must load");

    run_with(&project, &flip_args("done", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

#[test]
fn unknown_status_errors() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], legend_config());
    assert!(run_with(&project, &flip_args("nope", &["alpha"])).is_err());
}

#[test]
fn batch_applies_valid_and_reports_unresolved() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(
        dir.path(),
        &[("alpha.typ", ALPHA), ("beta.typ", BETA)],
        legend_config(),
    );
    // One good slug, one nonexistent: the good one still flips, overall Err.
    let result = run_with(&project, &flip_args("done", &["alpha", "ghost"]));
    assert!(result.is_err(), "missing target must surface");
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

#[test]
fn restamp_keeps_same_status() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], legend_config());
    run_with(&project, &flip_args("wip", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: wip("));
}

#[test]
fn list_verbs_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[], legend_config());
    let mut args = flip_args("done", &[]);
    args.status = None;
    args.list = true;
    assert!(run_with(&project, &args).is_ok());
}

#[test]
fn note_flag_writes_content_block() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], legend_config());
    let mut args = flip_args("blocked", &["alpha"]);
    args.note = Some("waiting on review".to_owned());
    run_with(&project, &args).unwrap();
    assert!(
        read(dir.path(), "alpha.typ").contains("status: blocked(2026, 6, 1)[waiting on review]")
            || read(dir.path(), "alpha.typ").contains("[waiting on review]")
    );
}

#[test]
fn status_edit_shortcut_form() {
    let out = status_edit(ALPHA, "done", (2026, 6, 28), None).unwrap();
    assert!(out.contains("status: done(2026, 6, 28)"), "{out}");
}

#[test]
fn status_edit_verbose_form_for_custom() {
    let out = status_edit(ALPHA, "icebox", (2026, 6, 28), None).unwrap();
    assert!(
        out.contains("status: status(\"icebox\", 2026, 6, 28)"),
        "{out}"
    );
}

#[test]
fn status_edit_errors_without_task_call() {
    assert!(status_edit("not a task file", "done", (2026, 6, 28), None).is_err());
}

#[test]
fn shortcut_parses_status_and_tasks() {
    let parsed = Shortcut::try_parse_from(["done", "1b", "2c"]).unwrap();
    assert_eq!(parsed.args.status.as_deref(), Some("done"));
    assert_eq!(parsed.args.tasks, vec!["1b", "2c"]);
}

#[test]
fn shortcut_parses_note_flag() {
    let parsed = Shortcut::try_parse_from(["wip", "-m", "on it", "1b"]).unwrap();
    assert_eq!(parsed.args.status.as_deref(), Some("wip"));
    assert_eq!(parsed.args.note.as_deref(), Some("on it"));
    assert_eq!(parsed.args.tasks, vec!["1b"]);
}
