//! End-to-end tests for the `derive` action of `[[on.add]]`: a Typst function
//! over the sanitized draft writes a field or chooses the destination.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use super::add_hooks_support::{DERIVE_DIR, add, project, read};
#[test]
fn a_derived_list_replaces_the_field_and_a_later_hook_sees_it() {
    let dir = project(
        r#"
[[on.add]]
name = "classify"
actions = [{ derive = { typst = "add.typ", function = "classify-tags", field = "tags" } }]

[[on.add]]
name = "regression"
when = 'has_tag("derived")'
actions = [{ append = { tags = ["regression"] } }]
"#,
    );
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "derived", "Derive the tags"]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.contains("on.add classify: derive tags via add.typ:classify-tags"),
        "{stderr}"
    );
    let written = read(dir.path(), "tasks/derived.typ");
    assert!(
        written.contains("tags: (\"derived\", \"from-title\", \"regression\")"),
        "{written}"
    );
}

#[test]
fn a_derived_dir_chooses_the_destination() {
    let dir = project(DERIVE_DIR);
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "placed", "Placed"]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.contains("on.add classify: dir via add.typ:classify-dir"),
        "{stderr}"
    );
    let written = read(dir.path(), "tasks/bug/placed.typ");
    assert!(written.contains("kind: \"bug\""), "{written}");
    assert!(written.contains("\"derived\""), "{written}");
}

#[test]
fn a_derived_value_the_field_cannot_hold_is_refused() {
    let dir = project(
        r#"
[[on.add]]
name = "bad"
actions = [{ derive = { typst = "add.typ", function = "bad-tags", field = "tags" } }]
"#,
    );
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "bad", "Bad value"]);
    assert!(!ok, "{stderr}");
    assert!(
        stderr.contains("a derive returned a value the field cannot hold"),
        "{stderr}"
    );
    assert!(stderr.contains("tags via add.typ:bad-tags"), "{stderr}");
    assert!(!dir.path().join("tasks/bad.typ").exists());
}

#[test]
fn a_derive_naming_a_missing_function_is_a_config_error() {
    let dir = project(
        r#"
[[on.add]]
name = "ghost"
actions = [{ derive = { typst = "add.typ", function = "ghost", field = "kind" } }]
"#,
    );
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "ghost", "Ghost"]);
    assert!(!ok, "{stderr}");
    assert!(stderr.contains("does not export `ghost`"), "{stderr}");
    assert!(stderr.contains("on.add[0].actions[0].derive"), "{stderr}");
}

#[test]
fn a_derived_kind_fires_a_later_hook() {
    let dir = project(
        r#"
[[on.add]]
name = "classify"
actions = [{ derive = { typst = "add.typ", function = "classify-kind", field = "kind" } }]

[[on.add]]
name = "regression"
when = 'kind == "bug"'
actions = [{ append = { tags = ["regression"] } }]
"#,
    );
    let (_, stderr, ok) = add(dir.path(), &["add", "--name", "fired", "Fired"]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.contains("on.add regression: append tags"),
        "{stderr}"
    );
    let written = read(dir.path(), "tasks/fired.typ");
    assert!(written.contains("kind: \"bug\""), "{written}");
    assert!(written.contains("\"regression\""), "{written}");
}

#[test]
fn describe_names_a_derive_function() {
    let dir = project(DERIVE_DIR);
    let (stdout, stderr, ok) = add(dir.path(), &["add", "--describe"]);
    assert!(ok, "{stderr}");
    assert!(
        stdout.contains("derive tags via add.typ:classify-tags"),
        "{stdout}"
    );
    assert!(stdout.contains("dir via add.typ:classify-dir"), "{stdout}");
}
