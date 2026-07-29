//! End-to-end tests for the `install` command that shell out to the `mt`
//! binary, exercising real app-directory resolution
//! (`mindtape::config_home()` / `mindtape::data_home()`).
//!
//! Staged through `$MINDTAPE_CONFIG_HOME`/`$MINDTAPE_DATA_HOME` rather than the
//! XDG pair: `dirs` reads the environment on Unix only, so an XDG-injected
//! directory is not where a Windows `mt install` writes and every assertion
//! below would be about a path nothing touched.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

/// Point the config and data homes at fresh tempdirs under `home` and run
/// `mt <args>` with them, returning the command's `Assert`.
fn run_install(home: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .env(mindtape::CONFIG_HOME_ENV, home.join("config"))
        .env(mindtape::DATA_HOME_ENV, home.join("data"))
        .args(args)
        .assert()
}

fn defaults_path(home: &Path) -> PathBuf {
    home.join("config/mindtape/defaults.toml")
}

fn schema_path(home: &Path) -> PathBuf {
    home.join("config/mindtape/config.schema.json")
}

fn lib_typst_toml(home: &Path) -> PathBuf {
    home.join("data/typst/packages/local/mindtape/0.2.0/typst.toml")
}

fn lib_prelude(home: &Path) -> PathBuf {
    home.join("data/typst/packages/local/mindtape/0.2.0/prelude.typ")
}

/// Seed an existing XDG `defaults.toml` (sentinel content) under `home`,
/// returning its path.
fn seed_existing_defaults(home: &Path) -> PathBuf {
    let path = defaults_path(home);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "# existing").unwrap();
    path
}

/// Bare `mt install` writes the Typst package files, the XDG global
/// `defaults.toml`, and the JSON Schema.
#[test]
fn install_writes_lib_defaults_and_schema() {
    let home = tempfile::tempdir().unwrap();
    run_install(home.path(), &["install"]).success();

    assert!(lib_typst_toml(home.path()).exists());
    assert!(lib_prelude(home.path()).exists());
    assert!(defaults_path(home.path()).exists());
    assert!(schema_path(home.path()).exists());

    let content = std::fs::read_to_string(defaults_path(home.path())).unwrap();
    assert_eq!(content, mindtape::embedded_defaults());
    let schema = std::fs::read_to_string(schema_path(home.path())).unwrap();
    assert_eq!(schema, mindtape::config_schema_json());
}

/// `--no-lib` writes the XDG defaults and schema, skipping the Typst package.
#[test]
fn no_lib_skips_only_the_package() {
    let home = tempfile::tempdir().unwrap();
    run_install(home.path(), &["install", "--no-lib"]).success();

    assert!(defaults_path(home.path()).exists());
    assert!(schema_path(home.path()).exists());
    assert!(!lib_typst_toml(home.path()).exists());
}

/// `--no-defaults` writes the Typst package and schema, skipping the defaults.
#[test]
fn no_defaults_skips_only_the_defaults() {
    let home = tempfile::tempdir().unwrap();
    run_install(home.path(), &["install", "--no-defaults"]).success();

    assert!(lib_typst_toml(home.path()).exists());
    assert!(schema_path(home.path()).exists());
    assert!(!defaults_path(home.path()).exists());
}

/// `--no-schema` writes the lib and defaults but skips the JSON Schema.
#[test]
fn no_schema_skips_only_the_schema() {
    let home = tempfile::tempdir().unwrap();
    run_install(home.path(), &["install", "--no-schema"]).success();

    assert!(lib_typst_toml(home.path()).exists());
    assert!(defaults_path(home.path()).exists());
    assert!(!schema_path(home.path()).exists());
}

/// An existing `config.schema.json` survives a bare `mt install`, and `--force`
/// rewrites it.
#[test]
fn schema_force_interplay() {
    let home = tempfile::tempdir().unwrap();
    let path = schema_path(home.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{}").unwrap();

    run_install(home.path(), &["install", "--no-lib", "--no-defaults"]).success();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{}");

    run_install(
        home.path(),
        &["install", "--no-lib", "--no-defaults", "--force"],
    )
    .success();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        mindtape::config_schema_json()
    );
}

/// An existing XDG `defaults.toml` survives a bare `mt install`.
#[test]
fn existing_defaults_preserved_without_force() {
    let home = tempfile::tempdir().unwrap();
    let path = seed_existing_defaults(home.path());

    run_install(home.path(), &["install"]).success();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# existing");
}

/// `--force` overwrites an existing XDG `defaults.toml`.
#[test]
fn existing_defaults_overwritten_with_force() {
    let home = tempfile::tempdir().unwrap();
    let path = seed_existing_defaults(home.path());

    run_install(home.path(), &["install", "--force"]).success();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        mindtape::embedded_defaults()
    );
}

/// The seeded `defaults.toml` parses and validates through `parse_config` —
/// it is the floor text verbatim, which always merges cleanly with itself.
#[test]
fn seeded_defaults_parses_through_parse_config() {
    let home = tempfile::tempdir().unwrap();
    run_install(home.path(), &["install"]).success();

    let content = std::fs::read_to_string(defaults_path(home.path())).unwrap();
    let config = mindtape::parse_config(&content).unwrap();
    config.validate().unwrap();
}

/// The hidden, deprecated `install-lib` alias still installs the lib only,
/// equivalent to `mt install --no-defaults`.
#[test]
fn install_lib_alias_installs_lib_only() {
    let home = tempfile::tempdir().unwrap();
    run_install(home.path(), &["install-lib"]).success();

    assert!(lib_typst_toml(home.path()).exists());
    assert!(lib_prelude(home.path()).exists());
    assert!(!defaults_path(home.path()).exists());
    assert!(!schema_path(home.path()).exists());
}

/// `install-lib` is hidden from `--help` (deprecated in favour of `install`).
#[test]
fn install_lib_hidden_from_help() {
    let home = tempfile::tempdir().unwrap();
    let assert = run_install(home.path(), &["--help"]).success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(output.contains("install"), "{output}");
    assert!(!output.contains("install-lib"), "{output}");
}
