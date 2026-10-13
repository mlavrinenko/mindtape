//! Tests for the `flip` command: status rewriting, shortcut parsing, batch.
//! The post-flip revalidation cluster lives in the `cmd/flip/tests_revalidate.rs`
//! sibling; `cmd/flip/tests_output.rs` reads this module's fixtures for its
//! child-process tests.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use clap::Parser;
use mindtape::parse_config;

use super::{
    FlipArgs, Shortcut, check_note_required, check_transition, hooks_enabled, run_with, status_edit,
};
use crate::diag::plain;
use crate::util::Project;

use mindtape::test_support::PRELUDE;

pub(super) const ALPHA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Alpha\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n";

pub(super) const BETA: &str = "#import \"@local/mindtape:0.2.0\": *\n\
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

/// Project rooted at `dir` with an in-tree `lib/` mindtape package override (so
/// `@local/mindtape` resolves without a global install), a real
/// `.mindtape/config.toml` holding `config_text`, and `files` under `tasks/`.
/// The config is written to disk (not just parsed in memory) because
/// `run_with`'s post-flip revalidation reloads it from `.mindtape/config.toml`;
/// without a matching file, that reload would see the embedded floor's legend
/// instead of `config_text`'s.
pub(super) fn setup(files: &[(&str, &str)], config_text: &str) -> (tempfile::TempDir, Project) {
    crate::test_util::make_project_with_lib(PRELUDE, files, config_text)
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
    let (dir, project) = setup(&[("alpha.typ", ALPHA)], LEGEND_CONFIG);
    assert_eq!(project.outcome.tasks.len(), 1, "fixture must load");

    run_with(&project, &flip_args("done", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

/// A task body carrying deep leading indentation (the `mt add`/surgical-edit
/// drift) is canonicalized on flip when `[format]` is enabled (the default).
#[test]
fn flip_formats_body_drift_by_default() {
    let drift = format!(
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(\n  \
         title: \"Drifty\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\n{}drift line\n",
        " ".repeat(40)
    );
    let (dir, project) = setup(&[("drifty.typ", &drift)], LEGEND_CONFIG);

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
    let indent = " ".repeat(40);
    let drift = format!(
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(\n  \
         title: \"Drifty\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\n{indent}drift line\n"
    );
    let config = format!("{LEGEND_CONFIG}[format]\nenabled = false\n");
    let (dir, project) = setup(&[("drifty.typ", &drift)], &config);

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
    let (_dir, project) = setup(&[("alpha.typ", ALPHA)], LEGEND_CONFIG);
    assert!(run_with(&project, &flip_args("nope", &["alpha"])).is_err());
}

#[test]
fn batch_applies_valid_and_reports_unresolved() {
    let (dir, project) = setup(&[("alpha.typ", ALPHA), ("beta.typ", BETA)], LEGEND_CONFIG);
    // One good slug, one nonexistent: the good one still flips, overall Err.
    let result = run_with(&project, &flip_args("done", &["alpha", "ghost"]));
    assert!(result.is_err(), "missing target must surface");
    assert!(read(dir.path(), "alpha.typ").contains("status: done("));
}

#[test]
fn restamp_keeps_same_status() {
    let (dir, project) = setup(&[("alpha.typ", ALPHA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("wip", &["alpha"])).unwrap();
    assert!(read(dir.path(), "alpha.typ").contains("status: wip("));
}

#[test]
fn list_verbs_succeeds() {
    let (_dir, project) = setup(&[], LEGEND_CONFIG);
    let mut args = flip_args("done", &[]);
    args.status = None;
    args.list = true;
    assert!(run_with(&project, &args).is_ok());
}

#[test]
fn note_flag_writes_content_block() {
    let (dir, project) = setup(&[("alpha.typ", ALPHA)], LEGEND_CONFIG);
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
    assert_eq!(
        plain(&err),
        "error: illegal transition: 'blocked → done'\n\
         hint: or pass --force to override\n\
         note: allowed next: blocked → wip\n"
    );
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
    assert_eq!(
        plain(&err),
        "error: illegal transition: 'done → wip'\n\
         hint: or pass --force to override\n\
         note: no transitions are declared from: 'done'\n"
    );
}

#[test]
fn flip_rejects_illegal_transition_and_keeps_file() {
    // BETA is `blocked`; blocked → done is not a declared edge.
    let (dir, project) = setup(&[("beta.typ", BETA)], TRANSITION_CONFIG);
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
    // BETA is `blocked`; blocked → wip is a declared reopen edge.
    let (dir, project) = setup(&[("beta.typ", BETA)], TRANSITION_CONFIG);
    run_with(&project, &flip_args("wip", &["beta"])).unwrap();
    assert!(read(dir.path(), "beta.typ").contains("status: wip("));
}

#[test]
fn flip_to_terminal_preserves_recur_field() {
    // `mt done` on a recurring task stays a pure close: it never strips
    // `recur`, so `mt recur` stays callable on the now-done task.
    let (dir, project) = setup(&[("gamma.typ", GAMMA)], LEGEND_CONFIG);
    run_with(&project, &flip_args("done", &["gamma"])).unwrap();
    let out = read(dir.path(), "gamma.typ");
    assert!(out.contains("status: done("));
    assert!(out.contains("recur: every(weeks: 1)"));
}

#[test]
fn flip_force_bypasses_the_graph() {
    let (dir, project) = setup(&[("beta.typ", BETA)], TRANSITION_CONFIG);
    let mut args = flip_args("done", &["beta"]);
    args.force = true;
    run_with(&project, &args).unwrap();
    assert!(read(dir.path(), "beta.typ").contains("status: done("));
}

#[test]
fn unknown_status_does_not_touch_file() {
    let before = ALPHA.to_owned();
    let (dir, project) = setup(&[("a.typ", ALPHA)], LEGEND_CONFIG);
    assert!(run_with(&project, &flip_args("nope", &["a"])).is_err());
    assert_eq!(before, read(dir.path(), "a.typ"));
}

// --- suggest wiring for the unknown-status / shortcut-verb-miss messages
// (`truth/tasks/adopt-nearest-match-suggestions-at-the-remaining-unknown-name-errors.typ`) --

#[test]
fn unknown_status_near_miss_offers_a_suggestion() {
    let (_dir, project) = setup(&[("a.typ", ALPHA)], LEGEND_CONFIG);
    let msg = plain(&run_with(&project, &flip_args("donee", &["a"])).unwrap_err());
    assert_eq!(
        msg,
        "error: unknown status: 'donee'\nhint: did you mean done?\n"
    );
}

#[test]
fn unknown_status_far_miss_omits_the_suggestion() {
    let (_dir, project) = setup(&[("a.typ", ALPHA)], LEGEND_CONFIG);
    let msg = plain(&run_with(&project, &flip_args("zzzzzzzz", &["a"])).unwrap_err());
    assert_eq!(
        msg,
        "error: unknown status: 'zzzzzzzz'\nhint: run mt flip --list to see the statuses\n"
    );
}

#[test]
fn unknown_command_message_near_miss_offers_a_suggestion() {
    let legend = parse_config(LEGEND_CONFIG).unwrap().tasks.status;
    let msg = plain(&super::unknown_command("donee", &legend));
    assert_eq!(
        msg,
        "error: unknown command: 'donee'\nhint: did you mean done?\n"
    );
}

#[test]
fn unknown_command_message_far_miss_omits_the_suggestion() {
    let legend = parse_config(LEGEND_CONFIG).unwrap().tasks.status;
    let msg = plain(&super::unknown_command("zzzzzzzz", &legend));
    assert_eq!(
        msg,
        "error: unknown command: 'zzzzzzzz'\n\
         hint: run mt --help for commands, or mt flip --list for status verbs\n"
    );
}

#[test]
fn unknown_command_message_with_an_empty_legend_omits_the_suggestion() {
    let legend = mindtape::config::Legend::default();
    let msg = plain(&super::unknown_command("anything", &legend));
    assert!(!msg.contains("did you mean"), "{msg}");
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
    assert_eq!(
        plain(&err),
        "error: status requires a note: 'blocked'\n\
         hint: pass one with -m/--note\n"
    );
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
    let (dir, project) = setup(&[("alpha.typ", ALPHA)], NOTE_REQUIRED_CONFIG);
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
    let (dir, project) = setup(&[("alpha.typ", ALPHA)], NOTE_REQUIRED_CONFIG);
    let mut args = flip_args("blocked", &["alpha"]);
    args.note = Some("waiting on upstream".to_owned());
    run_with(&project, &args).unwrap();
    let body = read(dir.path(), "alpha.typ");
    assert!(body.contains("status: blocked("), "{body}");
    assert!(body.contains("waiting on upstream"), "{body}");
}

#[test]
fn hooks_need_a_terminal_and_a_non_quiet_run() {
    assert!(hooks_enabled(false, true), "quiet off, tty on: hooks run");
    assert!(!hooks_enabled(true, true), "quiet suppresses hooks");
    assert!(!hooks_enabled(false, false), "a pipe suppresses hooks");
}
