//! A legend entry with `note = "required"`: `mt flip` refuses to enter it
//! without a non-blank note.
#![allow(clippy::unwrap_used)]

use mindtape::parse_config;

use super::check_note_required;
use super::run_with;
use super::tests::{ALPHA, flip_args, read, setup};
use crate::diag::plain;

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
