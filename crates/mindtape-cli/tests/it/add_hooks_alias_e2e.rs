//! `mt add` alias routing: a template can fill a core field through a slot
//! with another name, and every explicit source must address that slot.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use super::add_hooks_support::{add, project, read};
use crate::support;

const ALIASED: &str = mindtape::typst_task!(
    r#"(
  title: slot("headline"),
  kind: slot("type"),
)
"#
);

const CLASSIFY: &str =
    "[[on.add]]\nname = \"classify\"\nactions = [{ set = { kind = \"feat\" } }]\n";

/// A project whose default template names the title and kind slots differently.
fn aliased_project(extra: &str) -> tempfile::TempDir {
    let dir = project(extra);
    std::fs::write(dir.path().join(".mindtape/templates/task.typ"), ALIASED).unwrap();
    dir
}

/// The explicit value under a slot alias owns the field, so the `classify`
/// hook setting `kind` keeps its hands off.
fn assert_kind_owned(args: &[&str], path: &str) {
    let dir = aliased_project(CLASSIFY);
    let (_, stderr, ok) = add(dir.path(), args);
    assert!(ok, "{stderr}");
    assert!(stderr.contains("kind keeps the explicit value"), "{stderr}");
    let written = read(dir.path(), path);
    assert!(written.contains("kind: \"bug\""), "{written}");
    assert!(!written.contains("kind: \"feat\""), "{written}");
}

fn assert_refused(args: &[&str], needle: &str) {
    let dir = aliased_project("");
    let (_, stderr, ok) = add(dir.path(), args);
    assert!(!ok, "{stderr}");
    assert!(stderr.contains(needle), "{stderr}");
}

fn assert_wrote(args: &[&str], path: &str, needle: &str) {
    let dir = aliased_project("");
    let (_, stderr, ok) = add(dir.path(), args);
    assert!(ok, "{stderr}");
    let written = read(dir.path(), path);
    assert!(written.contains(needle), "{written}");
}

#[test]
fn an_explicit_value_under_an_aliased_slot_owns_the_field() {
    assert_kind_owned(
        &["add", "--name", "aliased", "--set", "type=bug", "Aliased"],
        "tasks/aliased.typ",
    );
}

#[test]
fn a_flag_fills_an_aliased_slot_and_owns_it_from_a_hook() {
    assert_kind_owned(
        &["add", "--name", "flagged", "--kind", "bug", "Flagged"],
        "tasks/flagged.typ",
    );
}

#[test]
fn a_flag_and_its_aliased_set_refuse_two_spellings() {
    assert_refused(
        &[
            "add",
            "--name",
            "duplicate",
            "--kind",
            "feat",
            "--set",
            "type=bug",
            "Duplicate",
        ],
        "the same field is set twice",
    );
}

#[test]
fn a_positional_title_fills_an_aliased_title_slot_and_mints_the_stem() {
    assert_wrote(
        &["add", "--set", "type=bug", "Alias", "title"],
        "tasks/alias-title.typ",
        "title: \"Alias title\"",
    );
}

#[test]
fn an_aliased_title_set_mints_the_stem() {
    assert_wrote(
        &["add", "--set", "type=bug", "--set", "headline=Alias title"],
        "tasks/alias-title.typ",
        "title: \"Alias title\"",
    );
}

#[test]
fn a_piped_title_fills_an_aliased_slot_and_mints_the_stem() {
    let dir = aliased_project("");
    support::mt(dir.path())
        .args(["add", "--set", "type=bug"])
        .write_stdin("Piped title\n")
        .assert()
        .success();
    let written = read(dir.path(), "tasks/piped-title.typ");
    assert!(written.contains("title: \"Piped title\""), "{written}");
}
