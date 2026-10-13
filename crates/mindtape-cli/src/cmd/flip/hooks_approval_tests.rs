//! The override marker a waived gate leaves in the status note, and the
//! approval step that holds back a config's commands until `mt allow`.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir;
use std::path::Path;

use super::tests::{Mode, run_in};
use super::{HookOutcome, note_with_overrides};
use crate::diag::plain;

// --- the override marker -----------------------------------------------

#[test]
fn note_is_untouched_when_no_gate_was_waived() {
    assert_eq!(
        note_with_overrides(Some("why"), &[]),
        Some("why".to_owned())
    );
    assert_eq!(note_with_overrides(None, &[]), None);
}

#[test]
fn a_waived_gate_is_appended_to_the_note() {
    let forced = vec!["just check".to_owned()];
    assert_eq!(
        note_with_overrides(Some("shipping anyway"), &forced),
        Some("shipping anyway — forced past failing gate: \"just check\"".to_owned())
    );
}

#[test]
fn a_waived_gate_stands_alone_without_a_note() {
    let forced = vec!["just check".to_owned(), "cargo test".to_owned()];
    assert_eq!(
        note_with_overrides(None, &forced),
        Some("forced past failing gate: \"just check\", \"cargo test\"".to_owned())
    );
    // A blank note is not a note.
    let one = vec!["just check".to_owned()];
    assert_eq!(
        note_with_overrides(Some("  "), &one),
        Some("forced past failing gate: \"just check\"".to_owned())
    );
}

// --- approval ----------------------------------------------------------

const MARK_GATE: &str = "[[on.flip]]\nwhen = 'into(\"done\")'\n\
     actions = [{ run = 'touch mark', gate = true }]\n";

/// Hooks for `alpha` → `done` before the user ran `mt allow`.
fn run_unallowed(dir: &Path, config_text: &str, force: bool) -> (HookOutcome, String) {
    let mode = Mode {
        show: false,
        force,
        allowed: false,
    };
    run_in(dir, config_text, "done", mode)
}

#[test]
fn an_unallowed_gate_refuses_without_running() {
    let dir = tempdir();
    let (outcome, out) = run_unallowed(dir.path(), MARK_GATE, false);
    assert_eq!(
        plain(&outcome.refusal.unwrap()),
        "error: gate not allowed: 'touch mark'\n\
         hint: read the config's commands, then run mt allow\n\
         hint: or pass --force to skip the gate; the status note records it\n"
    );
    assert_eq!(out, "skipped: touch mark (not allowed)\n");
    assert!(!dir.path().join("mark").exists());
    assert!(mindtape::allow::held_back().contains(&"touch mark".to_owned()));
}

#[test]
fn force_waives_an_unallowed_gate_and_records_it() {
    let dir = tempdir();
    let (outcome, _) = run_unallowed(dir.path(), MARK_GATE, true);
    assert!(outcome.refusal.is_none());
    assert_eq!(outcome.forced, vec!["touch mark".to_owned()]);
    assert!(!dir.path().join("mark").exists());
}

#[test]
fn an_unallowed_run_without_gate_is_skipped_and_the_walk_goes_on() {
    let dir = tempdir();
    let config = "[[on.flip]]\nwhen = 'into(\"done\")'\n\
                  actions = [{ run = 'touch mark' }, { run = 'touch other', gate = true }]\n";
    let (outcome, out) = run_unallowed(dir.path(), config, false);
    assert!(
        out.starts_with("skipped: touch mark (not allowed)\n"),
        "{out}"
    );
    assert!(out.contains("skipped: touch other"), "{out}");
    assert!(outcome.refusal.is_some(), "the gate after it still refuses");
    assert!(!dir.path().join("mark").exists());
}

#[test]
fn a_runless_gate_needs_no_approval() {
    let dir = tempdir();
    let config = "[[on.flip]]\nwhen = 'into(\"done\")'\n\
                  actions = [{ gate = true, message = \"no\" }]\n";
    let (outcome, out) = run_unallowed(dir.path(), config, false);
    assert!(plain(&outcome.refusal.unwrap()).starts_with("error: gate refused"));
    assert!(!out.contains("skipped"), "{out}");
}
