#![allow(clippy::unwrap_used)]

use super::scaffold_config;

#[test]
fn scaffold_creates_config_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(dir.path(), false).unwrap();
    assert!(dir.path().join(".mindtape").join("config.toml").exists());
}

#[test]
fn scaffold_content_is_non_empty() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_config(dir.path(), false).unwrap();
    let content =
        std::fs::read_to_string(dir.path().join(".mindtape").join("config.toml")).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn scaffold_skips_existing_without_force() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "# existing",
    )
    .unwrap();
    scaffold_config(dir.path(), false).unwrap();
    let content =
        std::fs::read_to_string(dir.path().join(".mindtape").join("config.toml")).unwrap();
    assert_eq!(content, "# existing");
}

#[test]
fn scaffold_overwrites_with_force() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape").join("config.toml"), "# old").unwrap();
    scaffold_config(dir.path(), true).unwrap();
    let content =
        std::fs::read_to_string(dir.path().join(".mindtape").join("config.toml")).unwrap();
    assert_ne!(content, "# old");
}

/// Regression: the generic `mt init` scaffold still parses and validates
/// through the embedded-floor `parse_config`, unchanged.
#[test]
fn default_config_parses_and_validates() {
    let config = mindtape::parse_config(super::DEFAULT_CONFIG).unwrap();
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
