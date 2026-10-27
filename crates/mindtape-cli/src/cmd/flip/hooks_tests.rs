//! Tests for the `[[on.flip]]` action runner: what each kind emits, when it
//! falls silent, and what a gate does to the flip.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir;
use std::path::Path;

use mindtape::parse_config;

use super::{HookOutcome, Hooks, Transition, view_sort};
use crate::diag::plain;
use crate::util::Project;

/// A task file in the `#metadata` form, so a fixture needs no Typst package.
fn task_src(title: &str, status: &str) -> String {
    format!(
        "#metadata((\n  marker: \"mindtape.task\",\n  fields: (\n    \
         title: \"{title}\",\n    \
         status: (name: \"{status}\", date: datetime(year: 2026, month: 6, day: 1), note: none),\n  \
         ),\n  extras: (:),\n))\n"
    )
}

/// Two tasks — `alpha` (wip) and `beta` (proposed) — under `config_text`.
fn project(dir: &Path, config_text: &str) -> Project {
    let config = parse_config(config_text).unwrap();
    let alpha = task_src("Alpha", "wip");
    let beta = task_src("Beta", "proposed");
    crate::test_util::make_project_in(dir, &[("alpha.typ", &alpha), ("beta.typ", &beta)], config)
}

/// How a test runs the hooks: whether `show`/`list` print, whether `--force`
/// is on, and whether the user allowed the config's commands.
#[derive(Clone, Copy)]
pub(super) struct Mode {
    pub(super) show: bool,
    pub(super) force: bool,
    pub(super) allowed: bool,
}

/// Run every hook `config_text` declares for `alpha` → `target`, with the
/// config's commands allowed, returning the outcome and everything the runner
/// printed.
fn run(
    dir: &Path,
    config_text: &str,
    target: &str,
    show: bool,
    force: bool,
) -> (HookOutcome, String) {
    let mode = Mode {
        show,
        force,
        allowed: true,
    };
    run_in(dir, config_text, target, mode)
}

/// [`run`] in any [`Mode`].
pub(super) fn run_in(
    dir: &Path,
    config_text: &str,
    target: &str,
    mode: Mode,
) -> (HookOutcome, String) {
    let Mode {
        show,
        force,
        allowed,
    } = mode;
    let project = project(dir, config_text);
    let hooks = Hooks::new(&project, show);
    hooks.allowed.set(allowed).unwrap();
    let task = project
        .outcome
        .tasks
        .iter()
        .find(|task| task.id == "alpha")
        .unwrap()
        .clone();
    let path = dir.join("tasks/alpha.typ");
    let mut out: Vec<u8> = Vec::new();
    let outcome = hooks
        .run(
            &Transition {
                task: &task,
                path: &path,
                target,
                force,
            },
            &mut out,
        )
        .unwrap();
    (outcome, String::from_utf8(out).unwrap())
}

/// [`run`], with the default "visible, not forced" setting.
fn run_visible(dir: &Path, config_text: &str, target: &str) -> (HookOutcome, String) {
    run(dir, config_text, target, true, false)
}

// --- show --------------------------------------------------------------

const SHOW_HOOK: &str = "[[on.flip]]\nwhen = 'into(\"wip\")'\n\
                         actions = [{ show = \"pick one thing\" }]\n";

#[test]
fn show_prints_its_text() {
    let dir = tempdir();
    let (outcome, out) = run_visible(dir.path(), SHOW_HOOK, "wip");
    assert!(out.contains("pick one thing"), "{out}");
    assert!(outcome.refusal.is_none());
}

/// Presentation is suppressed under `--quiet` or a piped stdout.
#[test]
fn show_is_silent_when_suppressed() {
    let dir = tempdir();
    let (_, out) = run(dir.path(), SHOW_HOOK, "wip", false, false);
    assert_eq!(out, "");
}

/// `when` scopes actions to the transitions that match it — here, a different
/// target status fires nothing at all.
#[test]
fn a_non_matching_transition_runs_nothing() {
    let dir = tempdir();
    let (_, out) = run_visible(dir.path(), SHOW_HOOK, "done");
    assert_eq!(out, "");
}

// --- list --------------------------------------------------------------

const LIST_HOOK: &str = "[[on.flip]]\nwhen = 'into(\"wip\")'\n\
     actions = [{ list = 'status == \"wip\"', title = \"Already in flight\" }]\n";

#[test]
fn list_prints_the_matching_tasks_under_its_title() {
    let dir = tempdir();
    let (_, out) = run_visible(dir.path(), LIST_HOOK, "wip");
    assert!(out.contains("Already in flight"), "{out}");
    assert!(out.contains("Alpha"), "{out}");
    assert!(
        !out.contains("Beta"),
        "the filter must narrow the set: {out}"
    );
}

#[test]
fn list_says_so_when_nothing_matches() {
    let dir = tempdir();
    let config = "[[on.flip]]\nwhen = 'into(\"wip\")'\n\
                  actions = [{ list = 'status == \"cancelled\"' }]\n";
    let (_, out) = run_visible(dir.path(), config, "wip");
    assert!(out.contains("no matching tasks"), "{out}");
}

#[test]
fn list_is_silent_when_suppressed() {
    let dir = tempdir();
    let (_, out) = run(dir.path(), LIST_HOOK, "wip", false, false);
    assert_eq!(out, "");
}

#[test]
fn view_sort_follows_a_bare_view_reference() {
    let dir = tempdir();
    let config = "[views.hot]\nfilter = ''\nsort = [\"priority:desc\"]\n";
    let project = project(dir.path(), config);
    assert_eq!(view_sort(&project, "@hot"), vec!["priority:desc"]);
    assert!(view_sort(&project, "@ghost").is_empty());
    assert!(view_sort(&project, "status == \"wip\"").is_empty());
}

// --- run and gates -----------------------------------------------------

/// A gate on the way into a terminal status, spelled with a command that fails.
/// `&&` chains in both `sh` and `cmd`, where `;` does not.
const FAILING_GATE: &str = "[[on.flip]]\nwhen = 'into(\"done\")'\n\
     actions = [{ run = 'echo boom && exit 3', gate = true, message = \"prove it\" }]\n";

#[test]
fn a_passing_gate_allows_the_flip_and_stays_quiet() {
    let dir = tempdir();
    let config =
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{ run = \"true\", gate = true }]\n";
    let (outcome, out) = run_visible(dir.path(), config, "done");
    assert!(outcome.refusal.is_none());
    assert!(outcome.forced.is_empty());
    assert_eq!(out, "", "a passing gate prints nothing: {out}");
}

#[test]
fn a_failing_gate_refuses_and_surfaces_the_output() {
    let dir = tempdir();
    let (outcome, out) = run_visible(dir.path(), FAILING_GATE, "done");
    assert_eq!(
        plain(&outcome.refusal.unwrap()),
        "error: gate failed: 'echo boom && exit 3': exit 3\n\
         hint: prove it\n\
         hint: or pass --force to skip the gate; the status note records it\n"
    );
    assert!(
        out.contains("boom"),
        "the command output must surface: {out}"
    );
}

/// The evidence a refusal rests on is printed even with presentation off —
/// otherwise a scripted flip fails with no reason on screen.
#[test]
fn a_failing_gate_prints_its_output_even_when_suppressed() {
    let dir = tempdir();
    let (outcome, out) = run(dir.path(), FAILING_GATE, "done", false, false);
    assert!(outcome.refusal.is_some());
    assert!(out.contains("boom"), "{out}");
}

#[test]
fn force_waives_a_failing_gate_and_records_it() {
    let dir = tempdir();
    let (outcome, out) = run(dir.path(), FAILING_GATE, "done", true, true);
    assert!(
        outcome.refusal.is_none(),
        "--force must let the flip through"
    );
    assert_eq!(outcome.forced, vec!["echo boom && exit 3".to_owned()]);
    assert!(out.contains("--force"), "{out}");
}

/// Actions run in declaration order, and the first failing gate stops the walk.
#[test]
fn a_failed_gate_skips_the_actions_after_it() {
    let dir = tempdir();
    let config = "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [\
         { show = \"before\" }, \
         { run = \"false\", gate = true }, \
         { show = \"after\" }]\n";
    let (outcome, out) = run_visible(dir.path(), config, "done");
    assert!(outcome.refusal.is_some());
    assert!(out.contains("before"), "{out}");
    assert!(
        !out.contains("after"),
        "a refused gate stops the walk: {out}"
    );
}

/// Without `gate`, a non-zero exit is informational: it warns and allows.
#[test]
fn a_failing_run_without_gate_warns_but_allows() {
    let dir = tempdir();
    let config = "[[on.flip]]\nwhen = 'into(\"done\")'\n\
                  actions = [{ run = 'echo hmm && exit 1' }]\n";
    let (outcome, out) = run_visible(dir.path(), config, "done");
    assert!(outcome.refusal.is_none());
    assert!(out.contains("warning:"), "{out}");
    assert!(out.contains("hmm"), "{out}");
}

/// A successful non-gate run's output is presentation, so it follows `show`.
#[test]
fn a_passing_run_without_gate_prints_only_when_visible() {
    let dir = tempdir();
    let config = "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{ run = 'echo hello' }]\n";
    let (_, shown) = run_visible(dir.path(), config, "done");
    assert!(shown.contains("hello"), "{shown}");
    let (_, hidden) = run(dir.path(), config, "done", false, false);
    assert_eq!(hidden, "");
}

/// Passes when the task file exists. `cmd` reads `%VAR%`, not `$VAR`.
#[cfg(not(windows))]
const FILE_EXISTS: &str = "test -f \"$MINDTAPE_TASK_FILE\"";
/// Passes when the task file exists. `cmd` reads `%VAR%`, not `$VAR`.
#[cfg(windows)]
const FILE_EXISTS: &str = "if exist %MINDTAPE_TASK_FILE% (exit 0) else (exit 1)";
/// Prints `<stem>/<status>` and fails, so the output shows.
#[cfg(not(windows))]
const ECHO_STEM_STATUS: &str = "echo \"$MINDTAPE_TASK_STEM/$MINDTAPE_TASK_STATUS\" && exit 1";
/// Prints `<stem>/<status>` and fails, so the output shows.
#[cfg(windows)]
const ECHO_STEM_STATUS: &str = "echo %MINDTAPE_TASK_STEM%/%MINDTAPE_TASK_STATUS%&& exit 1";

/// The command sees the task it is gating.
#[test]
fn a_command_receives_the_task_path_and_stem() {
    let dir = tempdir();
    let config =
        format!("[[on.flip]]\nwhen = ''\nactions = [{{ run = '{FILE_EXISTS}', gate = true }}]\n");
    let (outcome, out) = run_visible(dir.path(), &config, "done");
    assert!(outcome.refusal.is_none(), "the task file must exist: {out}");
    assert!(out.is_empty(), "a passing gate prints nothing: {out}");

    let echo = format!("[[on.flip]]\nwhen = ''\nactions = [{{ run = '{ECHO_STEM_STATUS}' }}]\n");
    let (_, printed) = run_visible(dir.path(), &echo, "done");
    assert!(printed.contains("alpha/done"), "{printed}");
}

// --- run-less gates ------------------------------------------------------

/// A gate with no `run`: `when` alone decides, unconditionally.
const RUNLESS_GATE: &str = "[[on.flip]]\nwhen = 'into(\"done\")'\n\
     actions = [{ gate = true, message = \"re-read the scope first\" }]\n";

#[test]
fn a_runless_gate_refuses_a_matching_flip_and_prints_its_message() {
    let dir = tempdir();
    let (outcome, out) = run_visible(dir.path(), RUNLESS_GATE, "done");
    assert_eq!(
        plain(&outcome.refusal.unwrap()),
        "error: gate refused\n\
         hint: re-read the scope first\n\
         hint: or pass --force to skip the gate; the status note records it\n"
    );
    assert!(
        out.is_empty(),
        "no command ran, so there is no output to print: {out}"
    );
}

/// A hook whose `when` does not match the transition lets the flip through, run
/// or no run.
#[test]
fn a_runless_gate_lets_a_non_matching_transition_through() {
    let dir = tempdir();
    let (outcome, out) = run_visible(dir.path(), RUNLESS_GATE, "wip");
    assert!(outcome.refusal.is_none());
    assert_eq!(out, "");
}

/// Without a `message`, a run-less gate falls back to the same default advice
/// a `run` gate does.
#[test]
fn a_runless_gate_without_a_message_uses_the_default_advice() {
    let dir = tempdir();
    let config = "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{ gate = true }]\n";
    let (outcome, _) = run_visible(dir.path(), config, "done");
    assert_eq!(
        plain(&outcome.refusal.unwrap()),
        "error: gate refused\n\
         hint: pass --force to skip the gate; the status note records it\n"
    );
}

/// `--force` waives a run-less gate exactly as it waives a failing `run` one,
/// and records something legible in the note — the message stands in for the
/// command there is none of.
#[test]
fn force_waives_a_runless_gate_and_records_its_message() {
    let dir = tempdir();
    let (outcome, out) = run(dir.path(), RUNLESS_GATE, "done", true, true);
    assert!(
        outcome.refusal.is_none(),
        "--force must let the flip through"
    );
    assert_eq!(outcome.forced, vec!["re-read the scope first".to_owned()]);
    assert!(out.contains("--force"), "{out}");
}

#[test]
fn no_hooks_means_no_output_and_no_refusal() {
    let dir = tempdir();
    let (outcome, out) = run_visible(dir.path(), "[tasks]\nroots = [\"tasks\"]\n", "done");
    assert!(outcome.refusal.is_none());
    assert_eq!(out, "");
}

/// An action that resolves to no single kind is a hard error, not a silent
/// skip. Config load rejects it too; this pins the runner's own guard.
#[test]
fn an_unresolvable_action_errors() {
    let config = parse_config("[[on.flip]]\nwhen = ''\nactions = [{ title = \"x\" }]\n").unwrap();
    let alpha = task_src("Alpha", "wip");
    let (dir, project) = crate::test_util::make_project(&[("alpha.typ", &alpha)], config);
    let hooks = Hooks::new(&project, true);
    let task = project.outcome.tasks.first().unwrap().clone();
    let path = dir.path().join("tasks/alpha.typ");
    let err = hooks
        .run(
            &Transition {
                task: &task,
                path: &path,
                target: "done",
                force: false,
            },
            &mut Vec::new(),
        )
        .unwrap_err();
    assert!(err.to_string().contains("declares no kind"), "{err}");
}
