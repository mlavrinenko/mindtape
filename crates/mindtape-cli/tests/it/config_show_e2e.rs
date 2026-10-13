//! End-to-end tests for `mt config`'s own dispatch: the bare (no-subcommand)
//! default, project discovery failing outside a project, and a malformed
//! `config.toml`. `cmd/config_tests.rs`'s unit tests call `render_effective`/
//! `render_origin` directly with an explicit root, bypassing `run()`'s own
//! `cwd`-driven `util::resolve_root` and its `ConfigAction` dispatch — these
//! need a real `cwd` and a real (or missing/broken) project on disk.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::{tempdir, tempdir_with_config};

use crate::support;

#[test]
fn bare_config_defaults_to_show() {
    let dir = tempdir_with_config("[project]\ncode = \"demo\"\n");

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
    let dir = tempdir();

    let output = support::mt(dir.path()).arg("config").assert().failure();
    let stderr = support::stderr_of(&output);
    assert!(
        stderr.contains("no .mindtape/ directory here or above"),
        "{stderr}"
    );
}

#[test]
fn malformed_config_toml_fails_show_with_context() {
    let dir = tempdir_with_config("[project\nbroken toml");

    let output = support::mt(dir.path())
        .args(["config", "show"])
        .assert()
        .failure();
    let stderr = support::stderr_of(&output);
    assert!(stderr.contains("config.toml"), "{stderr}");
}

#[test]
fn malformed_config_toml_fails_origin_with_context() {
    let dir = tempdir_with_config("[project\nbroken toml");

    let output = support::mt(dir.path())
        .args(["config", "show", "--origin"])
        .assert()
        .failure();
    let stderr = support::stderr_of(&output);
    assert!(stderr.contains("config.toml"), "{stderr}");
}
