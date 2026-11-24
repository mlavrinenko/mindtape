//! Tests for the `[[on.flip]]` after-the-write phase: when it runs, what it
//! prints, and what a failing command does to the flip.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir;
use std::path::Path;

use super::tests::run_transition;
use super::{AFTER_ADVICE, HookOutcome, Hooks};
use crate::cmd::flip::batch::Landed;

// after the write

/// A hook whose `after` list holds `action`, on a flip into `done`.
fn after_action(action: &str) -> String {
    format!(
        "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{{ show = \"x\" }}]\n\
         after = [{action}]\n"
    )
}

/// A hook whose `after` command is `run`.
fn after_hook(run: &str) -> String {
    after_action(&format!("{{ run = {run:?} }}"))
}

/// Run `config_text`'s hooks for `alpha` → `target`, then the after phase for a
/// batch that landed `landed`, returning the outcome and everything printed.
fn run_after(
    dir: &Path,
    config_text: &str,
    target: &str,
    landed: &[&str],
    allowed: bool,
) -> (HookOutcome, String) {
    let project = crate::test_util::hook_project(dir, config_text);
    let hooks = Hooks::new(&project, true);
    hooks.allowed.set(allowed).unwrap();
    let mut out: Vec<u8> = Vec::new();
    let outcome = run_transition(&hooks, &project, target, false, &mut out);
    let mut batch = Landed::default();
    for id in landed {
        let mut task = project.outcome.tasks.first().unwrap().clone();
        task.id = (*id).to_owned();
        batch.record(&task);
    }
    hooks
        .after(&outcome.matched, &batch, target, &mut out)
        .unwrap();
    (outcome, String::from_utf8(out).unwrap())
}

/// [`run_after`] for a batch that landed `landed` at `done`, in a fresh
/// tempdir the caller keeps alive.
fn after(
    config_text: &str,
    landed: &[&str],
    allowed: bool,
) -> (tempfile::TempDir, HookOutcome, String) {
    let dir = tempdir();
    let (outcome, out) = run_after(dir.path(), config_text, "done", landed, allowed);
    (dir, outcome, out)
}

/// A hook's `after` actions run once for the batch, however many targets it
/// landed.
#[test]
fn after_runs_once_for_the_batch() {
    let config = after_hook("echo after>> marks.txt");
    let (dir, _, out) = after(&config, &["alpha", "beta"], true);
    assert!(out.contains("$ echo after>> marks.txt"), "{out}");
    let marks = std::fs::read_to_string(dir.path().join("marks.txt")).unwrap();
    assert_eq!(marks.trim(), "after", "one run for two targets: {marks}");
}

/// A hook whose `when` matched no landed target skips its after actions.
#[test]
fn after_skips_a_hook_that_did_not_match() {
    let config = after_hook("echo after>> marks.txt").replace("into(\"done\")", "into(\"wip\")");
    let (dir, _, out) = after(&config, &["alpha"], true);
    assert_eq!(out, "");
    assert!(!dir.path().join("marks.txt").exists());
}

/// A failing after command warns and never refuses: the write already landed.
#[test]
fn a_failing_after_command_warns_and_does_not_refuse() {
    let (_dir, outcome, out) = after(&after_hook("exit 3"), &["alpha"], true);
    assert!(outcome.refusal.is_none(), "{out}");
    assert!(out.contains("warning: exit 3 (exit 3)"), "{out}");
    assert!(out.contains(AFTER_ADVICE), "{out}");
}

/// The config's `message` replaces the default advice on a failing after
/// command, as it does on a failing pre-write one.
#[test]
fn an_after_message_replaces_the_advice() {
    let config = "[[on.flip]]\nwhen = 'into(\"done\")'\nactions = [{ show = \"x\" }]\n\
                  after = [{ run = \"exit 3\", message = \"refresh the arms\" }]\n";
    let (_dir, _, out) = after(config, &["alpha"], true);
    assert!(out.contains("refresh the arms"), "{out}");
    assert!(!out.contains(AFTER_ADVICE), "{out}");
}

/// An after command waits for `mt allow` like every other `run`, and joins the
/// held-back list the one warning reads.
#[test]
fn an_after_command_is_held_back_until_allowed() {
    let config = after_hook("echo after>> marks.txt");
    let (dir, outcome, out) = after(&config, &["alpha"], false);
    assert!(outcome.refusal.is_none(), "{out}");
    assert!(
        out.contains("skipped: echo after>> marks.txt (not allowed)"),
        "{out}"
    );
    assert!(!dir.path().join("marks.txt").exists());
    assert_eq!(
        mindtape::allow::take_held(dir.path()),
        vec!["echo after>> marks.txt".to_owned()]
    );
}

/// An after `show` presents like a pre-write one: it prints after the write,
/// and it runs nothing, so `mt allow` does not gate it.
#[test]
fn an_after_show_prints_after_the_write() {
    let config = after_action("{ show = \"refreshed the arms\" }");
    let (_dir, _, out) = after(&config, &["alpha"], true);
    assert!(out.contains("refreshed the arms"), "{out}");
}

/// A command that cannot spawn warns like a failing one. The write already
/// landed, and the flip still exits zero.
#[test]
fn an_after_command_that_cannot_spawn_warns() {
    let config = after_action("{ run = [\"no-such-program-for-mt-tests\"] }");
    let (_dir, outcome, out) = after(&config, &["alpha"], true);
    assert!(outcome.refusal.is_none(), "{out}");
    assert!(out.contains("could not run"), "{out}");
}
