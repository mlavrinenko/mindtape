#![allow(clippy::unwrap_used)]

use super::{EvalArgs, locate, run};

const TASK: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

/// `mt eval <file>` with the path spelled as the task reference.
fn eval(file: &std::path::Path, json: bool) -> anyhow::Result<()> {
    run(&EvalArgs {
        task: file.to_string_lossy().into_owned().into(),
        json,
    })
}

#[test]
fn eval_valid_file_table_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, TASK).unwrap();
    assert!(eval(&file, false).is_ok());
}

#[test]
fn eval_valid_file_json_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, TASK).unwrap();
    assert!(eval(&file, true).is_ok());
}

#[test]
fn eval_missing_file_returns_err() {
    // A path under a fresh tempdir that was never written: nonexistent by
    // construction on every platform, unlike a hardcoded `/tmp/...`.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("no_such_file.typ");
    assert!(eval(&file, false).is_err());
}

#[test]
fn eval_file_without_marker_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.typ");
    std::fs::write(&file, "= Just notes\n\nNo marker here.\n").unwrap();
    assert!(eval(&file, false).is_err());
}

#[test]
fn build_world_missing_file_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("no_such_file.typ");
    assert!(super::build_world(&file).is_err());
}

/// A `@local` package on disk: manifest plus the entrypoint it names. The
/// stub emits a `mindtape.task` marker directly, so a task importing it
/// harvests without the real prelude.
fn write_pkg(root: &std::path::Path, dir_rel: &str, name: &str) {
    let pkg = root.join(dir_rel);
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(
        pkg.join("typst.toml"),
        // A version no install would ship, so these stay hermetic whatever the
        // machine has under its own local-package directory.
        format!("[package]\nname = \"{name}\"\nversion = \"99.99.99\"\nentrypoint = \"lib.typ\"\n"),
    )
    .unwrap();
    std::fs::write(
        pkg.join("lib.typ"),
        "#let mk(t) = [#metadata((marker: \"mindtape.task\", fields: (title: t), extras: (:))) <mindtape>]\n",
    )
    .unwrap();
}

/// Write a task file importing `@local/mindtape` under `root`, returning it.
fn write_importing_task(root: &std::path::Path) -> std::path::PathBuf {
    let file = root.join("tasks").join("t.typ");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(
        &file,
        "#import \"@local/mindtape:99.99.99\": *\n#mk(\"Hi\")\n",
    )
    .unwrap();
    file
}

#[test]
fn eval_registers_an_in_tree_lib_named_mindtape() {
    let dir = tempfile::tempdir().unwrap();
    write_pkg(dir.path(), "lib", "mindtape");
    let file = write_importing_task(dir.path());
    assert!(eval(&file, true).is_ok());
}

#[test]
fn eval_does_not_hijack_a_foreign_lib_package() {
    // `<root>/lib` is some other project's Typst package, and the project
    // points `@local/mindtape` elsewhere. Keying the override off the mere
    // existence of `lib/typst.toml` resolved the import to `lib/` and failed
    // on its declared name; the manifest name is what decides.
    let dir = tempfile::tempdir().unwrap();
    write_pkg(dir.path(), "lib", "other");
    write_pkg(dir.path(), "vendor/mindtape", "mindtape");
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "[[typst.local-packages]]\nname = \"mindtape\"\npath = \"vendor/mindtape\"\n",
    )
    .unwrap();
    let file = write_importing_task(dir.path());
    assert!(eval(&file, true).is_ok());
}

// --- locate ---
//
// `locate` treats a path-spelled reference as a path (so it bails when the file
// is absent) and only falls through to project resolution for a bare stem or
// `#` handle. The two spellings below each take one side of
// `contains(['/', '\\']) || ends_with(".typ")`, so a `||` -> `&&` mutant —
// which lets both fall through to `resolve_task` and answers with a different
// error — fails the exact-message assertion.

#[test]
fn locate_refuses_a_missing_path_with_a_separator() {
    let err = locate("sub/nope").unwrap_err();
    assert!(
        err.to_string().contains("no such file: sub/nope"),
        "{err:#}"
    );
}

#[test]
fn locate_refuses_a_missing_typ_path() {
    let err = locate("nope.typ").unwrap_err();
    assert!(
        err.to_string().contains("no such file: nope.typ"),
        "{err:#}"
    );
}
