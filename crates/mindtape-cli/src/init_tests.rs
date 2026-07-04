#![allow(clippy::unwrap_used)]

use super::{FULL_CONFIG, SLIM_CONFIG, scaffold_config};

/// An existing `.mindtape/config.toml` survives `scaffold_config` without
/// `force`, regardless of which template would otherwise be written.
fn assert_skips_existing_without_force(template: &str) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "# existing",
    )
    .unwrap();
    scaffold_config(dir.path(), false, template).unwrap();
    let content =
        std::fs::read_to_string(dir.path().join(".mindtape").join("config.toml")).unwrap();
    assert_eq!(content, "# existing");
}

/// `scaffold_config` overwrites an existing `.mindtape/config.toml` with
/// `force`, writing exactly `template`.
fn assert_overwrites_with_force(template: &str) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape").join("config.toml"), "# old").unwrap();
    scaffold_config(dir.path(), true, template).unwrap();
    let content =
        std::fs::read_to_string(dir.path().join(".mindtape").join("config.toml")).unwrap();
    assert_eq!(content, template);
}

#[test]
fn scaffold_creates_config_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(dir.path(), false, SLIM_CONFIG).unwrap();
    assert!(dir.path().join(".mindtape").join("config.toml").exists());
}

#[test]
fn scaffold_content_is_non_empty() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(dir.path(), false, SLIM_CONFIG).unwrap();
    let content =
        std::fs::read_to_string(dir.path().join(".mindtape").join("config.toml")).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn scaffold_skips_existing_without_force() {
    assert_skips_existing_without_force(SLIM_CONFIG);
}

#[test]
fn scaffold_overwrites_with_force() {
    assert_overwrites_with_force(SLIM_CONFIG);
}

/// `--force` behaviour is unchanged for the `--full` template too.
#[test]
fn full_scaffold_skips_existing_without_force() {
    assert_skips_existing_without_force(FULL_CONFIG);
}

/// `--force` behaviour is unchanged for the `--full` template too.
#[test]
fn full_scaffold_overwrites_with_force() {
    assert_overwrites_with_force(FULL_CONFIG);
}

/// Regression: the fully-expanded (`--full`) scaffold still parses and
/// validates through the embedded-floor `parse_config`, unchanged.
#[test]
fn full_config_parses_and_validates() {
    let config = mindtape::parse_config(FULL_CONFIG).unwrap();
    config.validate().unwrap();
}

/// The slim scaffold — just `[tasks] roots` plus a comment — parses and
/// validates on its own, with the floor supplying everything else.
#[test]
fn slim_config_parses_and_validates() {
    let config = mindtape::parse_config(SLIM_CONFIG).unwrap();
    config.validate().unwrap();
}

/// Regression: `MindTape`'s own dogfood config (which fully overrides the
/// status legend and priority scale) still parses and validates through the
/// embedded-floor `parse_config`, unchanged.
#[test]
fn dogfood_config_parses_and_validates() {
    const DOGFOOD: &str = include_str!("../../../.mindtape/config.toml");
    let config = mindtape::parse_config(DOGFOOD).unwrap();
    config.validate().unwrap();
}
