//! The unknown-status and unknown-command errors of `mt flip`
//! (`cmd/flip/verbs.rs`): the file stays untouched, and a near miss earns a
//! nearest-match suggestion.
#![allow(clippy::unwrap_used)]

use mindtape::parse_config;

use super::run_with;
use super::tests::{ALPHA, LEGEND_CONFIG, flip_args, read, setup};
use crate::diag::plain;

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
