//! Unit tests for `mt mv`'s argument shaping: what counts as a new stem, and
//! where the renamed file lands. The end-to-end behaviour — the move, the
//! inbound rewrite, the refused collision — is in `tests/mv_e2e.rs`, which
//! drives the real binary against a real repository.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use super::{new_stem, renamed};

#[test]
fn a_trailing_typ_is_dropped_rather_than_doubled() {
    assert_eq!(new_stem("ship-it.typ").unwrap(), "ship-it");
    assert_eq!(new_stem("ship-it").unwrap(), "ship-it");
}

#[test]
fn an_empty_stem_is_refused() {
    assert!(new_stem("").is_err());
    // `.typ` alone is a bare extension, so dropping it leaves nothing.
    assert!(new_stem(".typ").is_err());
}

#[test]
fn a_path_separator_is_refused_in_both_spellings() {
    assert!(new_stem("sub/ship-it").is_err());
    assert!(new_stem("sub\\ship-it").is_err());
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
