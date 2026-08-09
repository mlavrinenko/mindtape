//! Tests for the `flip` command: status rewriting, shortcut parsing, batch.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use clap::Parser;
use mindtape::parse_config;

use super::{FlipArgs, Shortcut, check_note_required, check_transition, run_with, status_edit};
use crate::util::Project;

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

pub(super) const ALPHA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

const BETA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Beta\",\n  status: blocked(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

pub(super) const GAMMA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Water plants\",\n  status: wip(2026, 6, 1),\n  recur: every(weeks: 1),\n)\n\n== Summary\n\nBody.\n";

pub(super) const STATELESS: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Stateless\",\n)\n\n== Summary\n\nBody.\n";

/// A legend declaring `blocked` (not part of the embedded floor's status
/// legend), so it must be written to disk, not just held in memory: `run_with`
/// revalidates by reloading config from `.mindtape/config.toml` after a flip.
pub(super) const LEGEND_CONFIG: &str = "[[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"blocked\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A legend with a transition graph where `blocked` is declared but
/// unreachable from the initial state (`wip` → `done` only), so flipping a task
/// to it must be caught by `run_with`'s post-flip revalidation.
pub(super) const REACHABLE_CONFIG: &str = "\
[[tasks.status]]\nvalue = \"wip\"\nto = [\"done\"]\n\
[[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
[[tasks.status]]\nvalue = \"blocked\"\n";

/// A legend whose `broken"done` value carries a double quote: flipping to it
/// renders the generic `status("broken"done", …)` — a Typst syntax error — so
/// the rewrite lands on disk as a file that fails to reload, and the
/// post-flip `revalidate` must report that load error against the touched
/// file.
pub(super) const BROKEN_RELOAD_CONFIG: &str = "\
[[tasks.status]]\nvalue = \"wip\"\n\
[[tasks.status]]\nvalue = \"broken\\\"done\"\n\
[[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

/// A file that fails to evaluate (an unclosed content block), used as a
/// pre-existing broken task that no flip touches.
const BROKEN_FILE: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Broken\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody [\n";

/// Project rooted at `dir` with an in-tree `lib/` mindtape package override (so
/// `@local/mindtape` resolves without a global install), a real
/// `.mindtape/config.toml` holding `config_text`, and `files` under `tasks/`.
/// The config is written to disk (not just parsed in memory) because
/// `run_with`'s post-flip revalidation reloads it from `.mindtape/config.toml`;
/// without a matching file, that reload would see the embedded floor's legend
/// instead of `config_text`'s.
pub(super) fn setup(dir: &Path, files: &[(&str, &str)], config_text: &str) -> Project {
    crate::test_util::make_project_with_lib(dir, PRELUDE, files, config_text)
}

pub(super) fn flip_args(status: &str, tasks: &[&str]) -> FlipArgs {
    FlipArgs {
        status: Some(status.to_owned()),
        tasks: tasks.iter().map(|s| (*s).to_owned()).collect(),
        note: None,
        list: false,
        force: false,
        quiet: false,
    }
}

/// A legend with a transition graph: wip → {blocked, done}, blocked → wip, and
/// terminal done reopening to wip. `initial` is wip; every value is reachable.
const TRANSITION_CONFIG: &str = "[[tasks.status]]\nvalue = \"wip\"\nto = [\"blocked\", \"done\"]\n\
     [[tasks.status]]\nvalue = \"blocked\"\nto = [\"wip\"]\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\nto = [\"wip\"]\n";

pub(super) fn read(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join("tasks").join(name)).unwrap()
}

#[test]
fn flips_status_by_slug() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], LEGEND_CONFIG);
    assert_eq!(project.outcome.tasks.len(), 1, "fixture must load");

    run_with(&project, &flip_args("done", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

/// A task body carrying deep leading indentation (the `mt add`/surgical-edit
/// drift) is canonicalized on flip when `[format]` is enabled (the default).
#[test]
fn flip_formats_body_drift_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let drift = format!(
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(\n  \
         title: \"Drifty\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\n{}drift line\n",
        " ".repeat(40)
    );
    let project = setup(dir.path(), &[("drifty.typ", &drift)], LEGEND_CONFIG);

    run_with(&project, &flip_args("done", &["drifty"])).unwrap();

    let body = read(dir.path(), "drifty.typ");
    assert!(
        body.contains("status: done("),
        "status not flipped: {body:?}"
    );
    assert!(
        body.contains("\ndrift line\n"),
        "body indentation not canonicalized: {body:?}"
    );
}

/// With `[format] enabled = false` the flip is a raw byte splice: the status
/// changes but the body's drift is left exactly as authored.
#[test]
fn flip_preserves_drift_when_format_disabled() {
    let dir = tempfile::tempdir().unwrap();
    let indent = " ".repeat(40);
    let drift = format!(
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(\n  \
         title: \"Drifty\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\n{indent}drift line\n"
    );
    let config = format!("{LEGEND_CONFIG}[format]\nenabled = false\n");
    let project = setup(dir.path(), &[("drifty.typ", &drift)], &config);

    run_with(&project, &flip_args("done", &["drifty"])).unwrap();

    let body = read(dir.path(), "drifty.typ");
    assert!(
        body.contains("status: done("),
        "status not flipped: {body:?}"
    );
    assert!(
        body.contains(&format!("{indent}drift line")),
        "drift should survive with formatting off: {body:?}"
    );
}

#[test]
fn unknown_status_errors() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], LEGEND_CONFIG);
    assert!(run_with(&project, &flip_args("nope", &["alpha"])).is_err());
}

#[test]
fn batch_applies_valid_and_reports_unresolved() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(
        dir.path(),
        &[("alpha.typ", ALPHA), ("beta.typ", BETA)],
        LEGEND_CONFIG,
    );
    // One good slug, one nonexistent: the good one still flips, overall Err.
    let result = run_with(&project, &flip_args("done", &["alpha", "ghost"]));
    assert!(result.is_err(), "missing target must surface");
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

#[test]
fn restamp_keeps_same_status() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("wip", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: wip("));
}

#[test]
fn list_verbs_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[], LEGEND_CONFIG);
    let mut args = flip_args("done", &[]);
    args.status = None;
    args.list = true;
    assert!(run_with(&project, &args).is_ok());
}

#[test]
fn note_flag_writes_content_block() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], LEGEND_CONFIG);
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

// --- transition graph -----------------------------------------------------

fn status(name: &str) -> mindtape::Status {
    mindtape::Status {
        name: name.to_owned(),
        date: None,
        note: None,
    }
}

#[test]
fn transition_unconstrained_allows_any_move() {
    // LEGEND_CONFIG declares no `to`, so the graph is inactive: any edge passes.
    let legend = parse_config(LEGEND_CONFIG).unwrap().tasks.status;
    assert!(check_transition(&legend, Some(&status("wip")), "done").is_ok());
}

#[test]
fn transition_legal_edge_ok() {
    let legend = parse_config(TRANSITION_CONFIG).unwrap().tasks.status;
    assert!(check_transition(&legend, Some(&status("wip")), "done").is_ok());
    assert!(check_transition(&legend, Some(&status("done")), "wip").is_ok()); // reopen
}

#[test]
fn transition_illegal_edge_err() {
    let legend = parse_config(TRANSITION_CONFIG).unwrap().tasks.status;
    // blocked → done is not an edge (blocked only reopens to wip).
    let err = check_transition(&legend, Some(&status("blocked")), "done").unwrap_err();
    assert!(err.contains("illegal transition"), "{err}");
    assert!(err.contains("allowed from `blocked`: wip"), "{err}");
}

#[test]
fn transition_self_restamp_always_allowed() {
    let legend = parse_config(TRANSITION_CONFIG).unwrap().tasks.status;
    // wip → wip, even though wip is not in its own `to` list.
    assert!(check_transition(&legend, Some(&status("wip")), "wip").is_ok());
}

#[test]
fn transition_unset_status_is_exempt() {
    let legend = parse_config(TRANSITION_CONFIG).unwrap().tasks.status;
    assert!(check_transition(&legend, None, "done").is_ok());
}

#[test]
fn transition_sink_message_names_terminal() {
    // A legend where done is a strict sink (no `to`), so leaving it is illegal.
    let cfg = "[[tasks.status]]\nvalue = \"wip\"\nto = [\"done\"]\n\
        [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";
    let legend = parse_config(cfg).unwrap().tasks.status;
    let err = check_transition(&legend, Some(&status("done")), "wip").unwrap_err();
    assert!(err.contains("terminal here"), "{err}");
}

#[test]
fn flip_rejects_illegal_transition_and_keeps_file() {
    let dir = tempfile::tempdir().unwrap();
    // BETA is `blocked`; blocked → done is not a declared edge.
    let project = setup(dir.path(), &[("beta.typ", BETA)], TRANSITION_CONFIG);
    assert_eq!(project.outcome.tasks.len(), 1, "fixture must load");
    let result = run_with(&project, &flip_args("done", &["beta"]));
    assert!(result.is_err(), "illegal transition must surface");
    assert!(
        read(dir.path(), "beta.typ").contains("status: blocked("),
        "file must be left untouched"
    );
}

#[test]
fn flip_allows_legal_transition() {
    let dir = tempfile::tempdir().unwrap();
    // BETA is `blocked`; blocked → wip is a declared reopen edge.
    let project = setup(dir.path(), &[("beta.typ", BETA)], TRANSITION_CONFIG);
    run_with(&project, &flip_args("wip", &["beta"])).unwrap();
    assert!(read(dir.path(), "beta.typ").contains("status: wip("));
}

#[test]
fn flip_to_terminal_preserves_recur_field() {
    // `mt done` on a recurring task stays a pure close: it never strips
    // `recur`, so `mt recur` stays callable on the now-done task.
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("gamma.typ", GAMMA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("done", &["gamma"])).unwrap();
    let out = read(dir.path(), "gamma.typ");
    assert!(out.contains("status: done("));
    assert!(out.contains("recur: every(weeks: 1)"));
}

#[test]
fn flip_force_bypasses_the_graph() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("beta.typ", BETA)], TRANSITION_CONFIG);
    let mut args = flip_args("done", &["beta"]);
    args.force = true;
    run_with(&project, &args).unwrap();
    assert!(read(dir.path(), "beta.typ").contains("status: done("));
}

#[test]
fn unknown_status_does_not_touch_file() {
    let dir = tempfile::tempdir().unwrap();
    let before = ALPHA.to_owned();
    let project = setup(dir.path(), &[("a.typ", ALPHA)], LEGEND_CONFIG);
    assert!(run_with(&project, &flip_args("nope", &["a"])).is_err());
    assert_eq!(before, read(dir.path(), "a.typ"));
}

/// The post-flip revalidation reloads the project and reports violations on the
/// touched task. The returned error only carries the count; the violation text
/// itself goes to stderr, so `flip_tests_output.rs` asserts that half.
#[test]
fn revalidate_catches_unreachable_status() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("s.typ", STATELESS)], REACHABLE_CONFIG);
    let err = run_with(&project, &flip_args("blocked", &["s"]))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("1 task(s) had problems"),
        "expected the revalidation failure count: {err}"
    );
}

/// A flip whose rewrite breaks the file (the status value renders into a Typst
/// syntax error) surfaces through the post-flip reload: `revalidate` appends
/// the load error against the touched file to the problem list. The returned
/// error only carries the count; the message itself goes to stderr (see
/// `flip_tests_output.rs`).
#[test]
fn revalidate_reports_load_error_on_touched_file() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], BROKEN_RELOAD_CONFIG);
    let err = run_with(&project, &flip_args("broken\"done", &["alpha"]))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("1 task(s) had problems"),
        "expected the load-failure count: {err}"
    );
}

/// A pre-existing broken file elsewhere in the project is not the flip's
/// doing, so `revalidate` must not report its load error: only errors on the
/// touched file are appended to the problem list, and the batch stays Ok.
#[test]
fn revalidate_ignores_load_errors_on_untouched_files() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(
        dir.path(),
        &[("alpha.typ", ALPHA), ("broken.typ", BROKEN_FILE)],
        LEGEND_CONFIG,
    );
    run_with(&project, &flip_args("done", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

/// A task whose authored status is unreachable under the declared graph is a
/// pre-existing condition: when that task is not part of the batch, its
/// revalidation violation must not fail the flip of an unrelated task.
#[test]
fn revalidate_ignores_violations_on_untouched_tasks() {
    let dir = tempfile::tempdir().unwrap();
    // BETA's authored `blocked` is unreachable under REACHABLE_CONFIG; only
    // alpha is flipped, so the violation must stay out of the problem list.
    let project = setup(
        dir.path(),
        &[("alpha.typ", ALPHA), ("beta.typ", BETA)],
        REACHABLE_CONFIG,
    );
    run_with(&project, &flip_args("done", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

#[test]
fn verbs_csv_includes_legend_values_in_error() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("a.typ", ALPHA)], LEGEND_CONFIG);
    let msg = run_with(&project, &flip_args("nonexistent", &["a"]))
        .unwrap_err()
        .to_string();
    assert!(msg.contains("wip") && msg.contains("blocked"));
}

// --- note requirement ------------------------------------------------------

/// `blocked` declares `note = "required"`; `wip`/`done` are left at the
/// `Optional` default.
const NOTE_REQUIRED_CONFIG: &str = "[[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"blocked\"\nnote = \"required\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

#[test]
fn note_required_ok_when_target_has_no_requirement() {
    let legend = parse_config(NOTE_REQUIRED_CONFIG).unwrap().tasks.status;
    assert!(check_note_required(&legend, "wip", None).is_ok());
}

#[test]
fn note_required_err_without_a_note() {
    let legend = parse_config(NOTE_REQUIRED_CONFIG).unwrap().tasks.status;
    let err = check_note_required(&legend, "blocked", None).unwrap_err();
    assert!(err.contains("`blocked` requires a note"), "{err}");
    assert!(err.contains("-m/--note"), "{err}");
}

#[test]
fn note_required_err_with_a_blank_note() {
    let legend = parse_config(NOTE_REQUIRED_CONFIG).unwrap().tasks.status;
    assert!(check_note_required(&legend, "blocked", Some("   ")).is_err());
}

#[test]
fn note_required_ok_with_a_filled_note() {
    let legend = parse_config(NOTE_REQUIRED_CONFIG).unwrap().tasks.status;
    assert!(check_note_required(&legend, "blocked", Some("waiting on review")).is_ok());
}

#[test]
fn flip_refuses_note_required_status_without_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], NOTE_REQUIRED_CONFIG);
    let result = run_with(&project, &flip_args("blocked", &["alpha"]));
    assert!(
        result.is_err(),
        "flip into a note-required status without a note must fail"
    );
    assert!(
        read(dir.path(), "alpha.typ").contains("status: wip("),
        "file must be left untouched"
    );
}

#[test]
fn flip_accepts_note_required_status_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup(dir.path(), &[("alpha.typ", ALPHA)], NOTE_REQUIRED_CONFIG);
    let mut args = flip_args("blocked", &["alpha"]);
    args.note = Some("waiting on upstream".to_owned());
    run_with(&project, &args).unwrap();
    let body = read(dir.path(), "alpha.typ");
    assert!(body.contains("status: blocked("), "{body}");
    assert!(body.contains("waiting on upstream"), "{body}");
}
