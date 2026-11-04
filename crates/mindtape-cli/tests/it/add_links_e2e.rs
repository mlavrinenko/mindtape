//! End-to-end tests for `mt add`'s `links` slot: the YAML list of typed edges,
//! and the `--set` refusal that points at `-f`. The slot engine's rendering is
//! unit-tested in `mindtape`; these prove the CLI path writes a task that
//! `mt check` accepts.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::write_file;

const LINKS_TEMPLATE: &str = mindtape::typst_task!(
    r#"(
  title: slot("title", example: "Linked"),
  links: slot("links", kind: "links", default: ()),
)

== Summary
"#
);

/// A `links` slot fills from `-f` as a list of typed edges, and the filed
/// task carries the same dicts the DSL helpers write.
#[test]
fn end_to_end_files_a_links_slot_from_yaml() {
    let dir = support::scaffold();
    write_file(dir.path(), ".mindtape/templates/links.typ", LINKS_TEMPLATE);
    write_file(
        dir.path(),
        "tasks/origin.typ",
        mindtape::typst_task!(r#"(title: "Origin", status: done(2026, 1, 1))"#),
    );

    support::mt(dir.path())
        .args(["add", "-t", "links", "--name", "linked", "-f", "-"])
        .write_stdin("title: Linked\nlinks: [{kind: related, task: origin.typ, note: why}]\n")
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/linked.typ")).unwrap();
    assert!(
        written.contains("links: ((task: \"origin.typ\", kind: \"related\", note: \"why\"),)"),
        "{written}"
    );
    support::check_ok(dir.path());
}

/// One `--set` token is a string, so a `links` slot refuses it and points at
/// `-f`, where a list of mappings has a shape.
#[test]
fn a_links_slot_refuses_a_set_token() {
    let dir = support::scaffold();
    write_file(dir.path(), ".mindtape/templates/links.typ", LINKS_TEMPLATE);

    let output = support::mt(dir.path())
        .args([
            "add",
            "-t",
            "links",
            "--name",
            "bad",
            "--set",
            "title=Bad",
            "--set",
            "links=origin.typ",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8_lossy(&output);
    assert!(stderr.contains("slot expects links"), "{stderr}");
    assert!(stderr.contains("-f"), "{stderr}");
    assert!(!dir.path().join("tasks/bad.typ").exists());
}
