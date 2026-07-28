//! End-to-end tests for `mt import` that shell out to the `mt` binary — the
//! only reliable way to exercise real stdin (unit tests can't fake process
//! stdin). File-arg input is covered here too for parity; the mapping/
//! round-trip/dry-run/failure-cleanup logic itself is unit-tested in
//! `crates/mindtape-cli/src/import_tests.rs` via `import::import_records`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n\
         [[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
    .unwrap();
    support::install_lib(dir);
}

const ONE_TASK_JSON: &str = r#"[{"id":"","file":"","title":"Wire import","status":{"name":"wip","date":"2026-06-01","note":null},"priority":null,"difficulty":null,"kind":null,"estimate":{},"links":[],"tags":[],"derived_tags":[],"due":null,"start":null,"reviewed":null,"body":"","extras":{}}]"#;

#[test]
fn imports_from_stdin() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    support::mt(dir.path())
        .arg("import")
        .write_stdin(ONE_TASK_JSON)
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-import.typ")).unwrap();
    assert!(written.contains("title: \"Wire import\""), "{written}");
    assert!(written.contains("status: wip("), "{written}");
}

#[test]
fn imports_from_a_file_argument() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    let input = dir.path().join("in.json");
    std::fs::write(&input, ONE_TASK_JSON).unwrap();

    support::mt(dir.path())
        .args(["import", input.to_str().unwrap()])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-import.typ")).unwrap();
    assert!(written.contains("title: \"Wire import\""), "{written}");

    // The filed task passes `check`.
    support::mt(dir.path()).arg("check").assert().success();
}
