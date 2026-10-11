//! End-to-end tests for `mt config`'s own dispatch: the bare (no-subcommand)
//! default, project discovery failing outside a project, and a malformed
//! `config.toml`. `cmd/config_tests.rs`'s unit tests call `render_effective`/
//! `render_origin` directly with an explicit root, bypassing `run()`'s own
//! `cwd`-driven `util::resolve_root` and its `ConfigAction` dispatch — these
//! need a real `cwd` and a real (or missing/broken) project on disk.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

fn scaffold(dir: &Path, config_toml: &str) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), config_toml).unwrap();
}

#[test]
fn bare_config_defaults_to_show() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), "[project]\ncode = \"demo\"\n");

    let bare = support::mt(dir.path())
        .arg("config")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let explicit = support::mt(dir.path())
        .args(["config", "show"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(
        bare, explicit,
        "omitting the subcommand must behave exactly like `show`"
    );
    assert!(
        String::from_utf8_lossy(&bare).contains("code = \"demo\""),
        "{}",
        String::from_utf8_lossy(&bare)
    );
}

#[test]
fn config_show_outside_a_project_fails_naming_the_reason() {
    let dir = tempfile::tempdir().unwrap();

    let output = support::mt(dir.path()).arg("config").assert().failure();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr).into_owned();
    assert!(
        stderr.contains("no .mindtape/ directory here or above"),
        "{stderr}"
    );
}

#[test]
fn malformed_config_toml_fails_show_with_context() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), "[project\nbroken toml");

    let output = support::mt(dir.path())
        .args(["config", "show"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr).into_owned();
    assert!(stderr.contains("config.toml"), "{stderr}");
}

#[test]
fn malformed_config_toml_fails_origin_with_context() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), "[project\nbroken toml");

    let output = support::mt(dir.path())
        .args(["config", "show", "--origin"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr).into_owned();
    assert!(stderr.contains("config.toml"), "{stderr}");
}
