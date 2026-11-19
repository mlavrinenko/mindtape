//! Tests for the `[[on.rm]]` action runner: what each kind emits, when it falls
//! silent, and what a gate does to the removal.
#![allow(clippy::unwrap_used)]

use std::collections::HashSet;
use std::path::Path;

use mindtape::test_support::tempdir;

use super::{Hooks, verdict};
use crate::diag::plain;

/// Run every hook `config_text` declares for `alpha`, returning everything the
/// runner printed, or the refusal that stopped it.
fn run(
    dir: &Path,
    config_text: &str,
    show: bool,
    allowed: bool,
) -> Result<String, crate::diag::Diag> {
    let project = crate::test_util::hook_project(dir, config_text);
    let eligible: HashSet<String> = ["alpha".to_owned()].into_iter().collect();
    let readiness = mindtape::Readiness::compute(&project.outcome.tasks, &project.config);
    let hooks = Hooks::new(&project, &eligible, &readiness, show);
    hooks.allowed.set(allowed).unwrap();
    let mut out: Vec<u8> = Vec::new();
    hooks
        .run(&mut out)
        .map(|()| String::from_utf8(out).unwrap())
}

/// [`run`] with the config's commands allowed and presentation on.
fn shown(dir: &Path, config_text: &str) -> String {
    run(dir, config_text, true, true).unwrap()
}

/// The refusal `config_text` produces for `alpha`, with the commands allowed.
fn refusal(dir: &Path, config_text: &str) -> String {
    plain(&run(dir, config_text, false, true).unwrap_err())
}

const SHOW_HOOK: &str = "[[on.rm]]\nwhen = 'status == \"wip\"'\n\
                         actions = [{ show = \"pick one thing\" }]\n";

#[test]
fn show_prints_its_text() {
    let dir = tempdir();
    assert_eq!(shown(dir.path(), SHOW_HOOK), "pick one thing\n");
}

/// A `when` that misses the removed task runs nothing.
#[test]
fn a_missed_when_runs_nothing() {
    let dir = tempdir();
    let config = "[[on.rm]]\nwhen = 'status == \"done\"'\n\
                  actions = [{ show = \"pick one thing\" }]\n";
    assert_eq!(shown(dir.path(), config), "");
}

const LIST_HOOK: &str = "[[on.rm]]\nwhen = ''\n\
                         actions = [{ list = 'status == \"wip\"', title = \"Still open\" }]\n";

#[test]
fn list_prints_the_matching_tasks_under_its_title() {
    let dir = tempdir();
    let out = shown(dir.path(), LIST_HOOK);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("Still open"));
    assert_eq!(
        lines.filter(|line| line.contains("Alpha")).count(),
        1,
        "{out}"
    );
    assert!(!out.contains("Beta"), "{out}");
}

#[test]
fn list_says_so_when_nothing_matches() {
    let dir = tempdir();
    let config = "[[on.rm]]\nwhen = ''\n\
                  actions = [{ list = \"status == \\\"done\\\"\" }]\n";
    assert_eq!(shown(dir.path(), config), "  (no matching tasks)\n");
}

/// A `list` filter that does not parse is an error, not a silent skip.
#[test]
fn a_bad_list_filter_is_an_error() {
    let dir = tempdir();
    let config = "[[on.rm]]\nwhen = ''\nactions = [{ list = \"no_such_field == 1\" }]\n";
    let text = plain(&run(dir.path(), config, true, true).unwrap_err());
    assert!(text.contains("unknown field"), "{text}");
}

/// An action that doesn't declare a kind is an error, not a silent skip.
#[test]
fn an_action_without_a_kind_is_an_error() {
    let dir = tempdir();
    let config = "[[on.rm]]\nwhen = ''\nactions = [{ title = \"x\" }]\n";
    assert!(refusal(dir.path(), config).contains("on.rm action"));
}

const PASSING_RUN: &str = "[[on.rm]]\nwhen = ''\nactions = [{ run = \"echo cleaned\" }]\n";

/// A piped stdout suppresses every kind's presentation, and a gate still runs.
#[test]
fn presentation_is_silent_when_suppressed() {
    let dir = tempdir();
    for config in [SHOW_HOOK, LIST_HOOK, PASSING_RUN] {
        assert_eq!(
            run(dir.path(), config, false, true).unwrap(),
            "",
            "{config}"
        );
    }
}

#[test]
fn a_passing_run_prints_its_output() {
    let dir = tempdir();
    assert_eq!(
        shown(dir.path(), PASSING_RUN),
        "$ echo cleaned\n  cleaned\n"
    );
}

/// A failing `run` without `gate` warns and lets the removal go on.
#[test]
fn a_failing_run_without_gate_only_warns() {
    let dir = tempdir();
    let config = "[[on.rm]]\nwhen = ''\nactions = [{ run = \"exit 1\" }]\n";
    let out = run(dir.path(), config, false, true).unwrap();
    assert!(
        out.contains("warning: exit 1 (exit 1) — fix it, then run mt rm again"),
        "{out}"
    );
}

/// A failing `run` with `gate` refuses the removal, and its `message` is the
/// hint.
#[test]
fn a_failing_run_gate_refuses() {
    let dir = tempdir();
    let config = "[[on.rm]]\nwhen = ''\n\
                  actions = [{ run = \"exit 1\", gate = true, message = \"clean first\" }]\n";
    let text = refusal(dir.path(), config);
    assert!(text.contains("gate failed: 'exit 1': exit 1"), "{text}");
    assert!(text.contains("hint: clean first"), "{text}");
}

/// A run-less gate refuses on `when` alone, with no command to run.
#[test]
fn a_runless_gate_refuses_with_its_message() {
    let dir = tempdir();
    let config = "[[on.rm]]\nwhen = ''\n\
                  actions = [{ gate = true, message = \"clean first\" }]\n";
    let text = refusal(dir.path(), config);
    assert!(text.contains("gate refused"), "{text}");
    assert!(text.contains("hint: clean first"), "{text}");
}

/// A gate that doesn't declare a `message` refuses with no hint of its own.
#[test]
fn a_gate_without_a_message_refuses_with_no_hint() {
    let dir = tempdir();
    let run_gate = "[[on.rm]]\nwhen = ''\nactions = [{ run = \"exit 1\", gate = true }]\n";
    let text = refusal(dir.path(), run_gate);
    assert!(text.contains("gate failed: 'exit 1': exit 1"), "{text}");
    assert!(!text.contains("hint:"), "{text}");

    let runless = "[[on.rm]]\nwhen = ''\nactions = [{ gate = true }]\n";
    let text = refusal(dir.path(), runless);
    assert!(text.contains("gate refused"), "{text}");
    assert!(!text.contains("hint:"), "{text}");
}

/// Until the user allows the config's commands, a gate refuses and a plain
/// `run` skips with a line saying so.
#[test]
fn an_unallowed_gate_refuses_and_a_plain_run_skips() {
    let dir = tempdir();
    let gate = "[[on.rm]]\nwhen = ''\nactions = [{ run = \"exit 0\", gate = true }]\n";
    let text = plain(&run(dir.path(), gate, false, false).unwrap_err());
    assert!(text.contains("gate not allowed"), "{text}");
    assert!(text.contains("hint: read the config's commands"), "{text}");

    let plain_run = "[[on.rm]]\nwhen = ''\nactions = [{ run = \"exit 0\" }]\n";
    let out = run(dir.path(), plain_run, false, false).unwrap();
    assert_eq!(out, "skipped: exit 0 (not allowed)\n");
}

/// A command the platform cannot start is a failure, not a silent skip, and
/// the message quotes the command.
#[test]
fn a_command_that_cannot_spawn_is_an_error() {
    let dir = tempdir();
    let config = "[[on.rm]]\nwhen = ''\n\
                  actions = [{ run = [\"no-such-program-for-mt-tests\"] }]\n";
    let text = refusal(dir.path(), config);
    assert!(text.contains("can't run on.rm command"), "{text}");
    assert!(text.contains("no-such-program-for-mt-tests"), "{text}");
}

/// The verdict words an exit code, or a signal when the command left none.
#[test]
fn the_verdict_words_an_exit_code_or_a_signal() {
    assert_eq!(verdict(Some(1)), "exit 1");
    assert_eq!(verdict(None), "terminated by signal");
}

/// A hook with no actions runs nothing, and the runner still walks the rest.
#[test]
fn a_hook_with_no_actions_runs_nothing() {
    let dir = tempdir();
    assert_eq!(
        shown(dir.path(), "[[on.rm]]\nwhen = ''\nactions = []\n"),
        ""
    );
}
