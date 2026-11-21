//! End-to-end tests for `mt config`'s own dispatch: the bare (no-subcommand)
//! default, project discovery failing outside a project, and a malformed
//! `config.toml`. `cmd/config_tests.rs`'s unit tests call `render_effective`/
//! `render_origin` directly with an explicit root, bypassing `run()`'s own
//! `cwd`-driven `util::resolve_root` and its `ConfigAction` dispatch. These
//! need a real `cwd` and a real (or missing/broken) project on disk.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

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
fn config_show_json_wraps_the_merged_table() {
    let dir = tempdir_with_config("[project]\ncode = \"demo\"\n");

    let text = support::stdout(dir.path(), &["config", "show", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();

    assert!(
        value["$schema"]
            .as_str()
            .unwrap_or_default()
            .ends_with("/config-show/1.schema.json"),
        "{value}"
    );
    assert_eq!(value["config"]["project"]["code"], "demo");
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

/// A rule that sets no `severity` still reports at a level, and the collected
/// view cannot say which: no layer set the key. `--resolved` prints the level
/// `mt` applies.
#[test]
fn config_show_resolved_prints_the_level_a_rule_reports_at() {
    let dir = tempdir_with_config(
        "[[tasks.rules]]\nname = \"no-full-stop\"\nassert = '!matches(title, \"\\\\.$\")'\n",
    );

    let collected = support::stdout(dir.path(), &["config", "show"]);
    assert_eq!(collected.matches("\nseverity = ").count(), 0, "{collected}");

    let resolved = support::stdout(dir.path(), &["config", "show", "--resolved"]);
    assert_eq!(resolved.matches("\nseverity = ").count(), 1, "{resolved}");
}

#[test]
fn config_show_resolved_json_mirrors_the_toml() {
    let dir = tempdir_with_config("[project]\ncode = \"demo\"\n");

    let text = support::stdout(dir.path(), &["config", "show", "--resolved", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();

    support::assert_schema(&value, "config-show");
    assert_eq!(value["config"]["project"]["code"], "demo");
    assert_eq!(value["config"]["format"]["enabled"], true);
    assert_eq!(
        value["config"]["tasks"]["estimate"]["units"],
        serde_json::json!([]),
        "a section no layer set still prints its defaults: {value}"
    );
}

/// A project built from the resolved output reproduces it byte for byte,
/// which is what "resolves to itself" means for this view.
#[test]
fn config_show_resolved_output_round_trips_as_a_config() {
    let dir = tempdir_with_config(
        "[project]\ncode = \"demo\"\n\n\
         [[tasks.rules]]\nname = \"no-full-stop\"\nassert = '!matches(title, \"\\\\.$\")'\n\n\
         [ls]\nhyperlinks = \"never\"\n",
    );

    let first = support::stdout(dir.path(), &["config", "show", "--resolved"]);
    let rebuilt = tempdir_with_config(&first);
    let second = support::stdout(rebuilt.path(), &["config", "show", "--resolved"]);

    assert_eq!(first, second);
}

#[test]
fn config_show_resolved_conflicts_with_origin_and_defaults() {
    let dir = tempdir_with_config("[project]\ncode = \"demo\"\n");

    for other in ["--origin", "--defaults"] {
        let output = support::mt(dir.path())
            .args(["config", "show", "--resolved", other])
            .assert()
            .failure();
        let stderr = support::stderr_of(&output);
        assert!(stderr.contains("cannot be used with"), "{other}: {stderr}");
    }
}
