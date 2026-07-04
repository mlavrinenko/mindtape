//! End-to-end tests for the `add` command that shell out to the `mt` binary.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used)]

use std::path::Path;

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

/// Scaffold a temp project that can evaluate `@local/mindtape` via the in-tree
/// lib override the loader honours.
fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("lib/prelude.typ"), PRELUDE).unwrap();
    std::fs::write(
        dir.join("lib/typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
}

#[test]
fn end_to_end_files_and_validates_a_task() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "wire-add", "--set", "title=Wire add"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-add.typ")).unwrap();
    assert!(written.contains("title: \"Wire add\""), "{written}");
    assert!(written.contains("status: proposed("), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task passes `check`.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();
}

#[test]
fn end_to_end_files_from_positional_title() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "cleanup", "kitchen"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/cleanup-kitchen.typ")).unwrap();
    assert!(written.contains("title: \"cleanup kitchen\""), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");
}

#[test]
fn end_to_end_nests_into_subfolder() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--to", "shopping", "Potatoes"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/shopping/potatoes.typ")).unwrap();
    assert!(written.contains("title: \"Potatoes\""), "{written}");
}

/// Parsed `add --describe=json` schema rows run in `dir`.
fn describe_slots(dir: &Path) -> serde_json::Value {
    let output = assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir)
        .args(["add", "--describe=json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

/// Slot names from `add --describe=json` run in `dir`.
fn describe_slot_names(dir: &Path) -> Vec<String> {
    describe_slots(dir)
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

/// The `kind` reported for slot `name` in `add --describe=json` run in `dir`.
fn describe_slot_kind(dir: &Path, name: &str) -> String {
    describe_slots(dir)
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row.get("name").and_then(serde_json::Value::as_str) == Some(name))
        .and_then(|row| row.get("kind"))
        .and_then(serde_json::Value::as_str)
        .unwrap()
        .to_owned()
}

#[test]
fn describe_json_emits_slot_schema() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    assert!(describe_slot_names(dir.path()).iter().any(|n| n == "title"));
}

/// A config whose `[tasks.priority]` is required: the barebones template omits
/// priority, so `add` must splice it in.
const PRIORITY_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n";

#[test]
fn injects_config_required_priority_slot() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe` surfaces the spliced-in priority slot.
    let names = describe_slot_names(dir.path());
    assert!(names.iter().any(|n| n == "priority"), "{names:?}");

    // Filling it writes a valid task.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "add",
            "--name",
            "with-pri",
            "--set",
            "title=T",
            "--set",
            "priority=7",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-pri.typ")).unwrap();
    assert!(written.contains("priority: 7"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // Omitting a config-required field is the existing missing-slot hard error.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "no-pri", "--set", "title=T"])
        .assert()
        .failure();
}

/// A config whose `[tasks.priority]` is real-scaled and required: the
/// barebones template omits priority, so `add` must splice in a `real`-kind
/// slot, not the `int` it would wrongly emit for an int-scaled field.
///
/// `bands = []` clears the embedded floor's int-contiguous bands (0-2/3-6/7-9),
/// which are non-contiguous under real semantics; a real-scale project either
/// defines real bands or drops them.
const REAL_PRIORITY_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"real\"\nmin = 0\nmax = 9\nrequired = true\nbands = []\n";

#[test]
fn injects_real_scale_priority_slot_and_round_trips_a_decimal() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        REAL_PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe=json` reports the spliced-in priority slot as `real`.
    assert_eq!(describe_slot_kind(dir.path(), "priority"), "real");

    // Filling it with a decimal writes a float literal without truncation.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "add",
            "--name",
            "with-real-pri",
            "--set",
            "title=T",
            "--set",
            "priority=2.5",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-real-pri.typ")).unwrap();
    assert!(written.contains("priority: 2.5"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task with a real priority value passes `check`.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();
}

#[test]
fn collision_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::write(dir.path().join("tasks/dup.typ"), "existing").unwrap();
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "dup", "--set", "title=Dup"])
        .assert()
        .failure();
}
