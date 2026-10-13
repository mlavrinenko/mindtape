//! Unit tests for `mt mv`'s argument shaping: what counts as a new stem, and
//! where the renamed file lands. The end-to-end behaviour — the move, the
//! inbound rewrite, the refused collision — is in `tests/it/mv_e2e.rs`, which
//! drives the real binary against a real repository.
#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir;
use std::path::{Path, PathBuf};

use super::{new_stem, renamed, verify};
use crate::test_util::make_project;

#[test]
fn a_trailing_typ_is_dropped_rather_than_doubled() {
    assert_eq!(new_stem("ship-it.typ").unwrap(), "ship-it");
    assert_eq!(new_stem("ship-it").unwrap(), "ship-it");
}

/// `new_stem`'s refusal as stderr gets it.
fn refusal(raw: &str) -> String {
    crate::diag::plain(&new_stem(raw).unwrap_err())
}

#[test]
fn an_empty_stem_is_refused() {
    let expected = "error: the new stem is empty\nhint: pass a stem such as new-name\n";
    assert_eq!(refusal(""), expected);
    // `.typ` alone is a bare extension, so dropping it leaves nothing.
    assert_eq!(refusal(".typ"), expected);
}

#[test]
fn a_path_separator_is_refused_in_both_spellings() {
    assert_eq!(
        refusal("sub/ship-it"),
        "error: the new stem has a path separator: 'sub/ship-it'\n\
         hint: mt mv renames a task where it stands\n\
         hint: moving a task between roots or folders changes its derived tags, so it is a decision of its own\n"
    );
    assert!(refusal("sub\\ship-it").starts_with("error: the new stem has a path separator: "));
}

#[test]
fn renaming_keeps_the_folder_and_the_extension() {
    assert_eq!(
        renamed(Path::new("tasks/sub/old.typ"), "new"),
        PathBuf::from("tasks/sub/new.typ")
    );
}

#[test]
fn renaming_a_dotted_stem_does_not_eat_its_own_dots() {
    // `Path::with_extension` would read `.task-260101` as the extension and
    // replace it; only the last dot names one, and the whole stem is the part
    // being replaced.
    assert_eq!(
        renamed(Path::new("tasks/old.task-260101.typ"), "new"),
        PathBuf::from("tasks/new.typ")
    );
}

// --- verify ---
//
// `verify` asks the project whether the batch may land; it must refuse a
// rewrite the project would reject. The renamed task below carries a status
// outside the legend, which `validate` refuses, so `verify` returns an error.
// A stubbed `Ok(())` would let `run` move the file and write the rewrites with
// nothing having checked them.

#[test]
fn verify_refuses_a_rename_the_project_would_reject() {
    let dir = tempdir();
    let config = mindtape::parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap();
    let project = make_project(dir.path(), &[], config);
    let to = dir.path().join("tasks/new.typ");
    let bad = "#metadata((\n  marker: \"mindtape.task\",\n  fields: (\n    title: \"T\",\n    \
               status: (name: \"bogus\", date: none, note: none),\n  ),\n  extras: (:),\n))\n";

    assert!(
        verify(&project, &to, bad, &[]).is_err(),
        "a task the project would reject must not be verified clean"
    );
}
