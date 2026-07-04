//! End-to-end tests for the `init` command that shell out to the `mt` binary.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

/// File a task in `dir` via `mt add --name <name> --set title=T <extra_set…>`,
/// then assert `mt ls` and `mt check` both succeed against it.
fn add_then_ls_and_check(dir: &Path, name: &str, extra_set: &[&str]) {
    let mut args = vec!["add", "--name", name, "--set", "title=T"];
    args.extend_from_slice(extra_set);
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir)
        .args(&args)
        .assert()
        .success();

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir)
        .arg("ls")
        .assert()
        .success();

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir)
        .arg("check")
        .assert()
        .success();
}

/// `mt init` writes the slim scaffold, which parses and validates through the
/// embedded floor and supports a real `mt add` / `mt ls` / `mt check` cycle.
#[test]
fn init_slim_scaffold_supports_add_ls_check() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("roots = [\"tasks\"]"), "{config}");
    // Slim: no status legend or priority bands spelled out — those come from
    // the embedded floor.
    assert!(!config.contains("[[tasks.status]]"), "{config}");
    assert!(!config.contains("[tasks.priority]"), "{config}");

    add_then_ls_and_check(dir.path(), "wire-init", &[]);
}

/// `mt init --full` writes the fully-expanded, opinionated starter config,
/// which also parses and validates and supports the same cycle.
#[test]
fn init_full_scaffold_supports_add_ls_check() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--full"])
        .assert()
        .success();

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("[[tasks.status]]"), "{config}");
    assert!(config.contains("[tasks.priority]"), "{config}");

    // The full config's priority is required for non-terminal tasks.
    add_then_ls_and_check(dir.path(), "wire-init-full", &["--set", "priority=5"]);
}

/// `--force` interplay is unchanged: without it, an existing config survives
/// `init`; with it, `init` overwrites regardless of `--full`.
#[test]
fn force_interplay_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape/config.toml"), "# existing").unwrap();

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap(),
        "# existing"
    );

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--full", "--force"])
        .assert()
        .success();
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("[[tasks.status]]"), "{config}");
}
