#![allow(clippy::unwrap_used)]

use super::write_defaults;

#[test]
fn write_defaults_creates_file_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mindtape");
    write_defaults(&target, false).unwrap();
    assert!(target.join("defaults.toml").exists());
}

#[test]
fn write_defaults_content_matches_embedded_floor() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mindtape");
    write_defaults(&target, false).unwrap();
    let content = std::fs::read_to_string(target.join("defaults.toml")).unwrap();
    assert_eq!(content, mindtape::embedded_defaults());
}

#[test]
fn write_defaults_skips_existing_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mindtape");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("defaults.toml"), "# existing").unwrap();
    write_defaults(&target, false).unwrap();
    let content = std::fs::read_to_string(target.join("defaults.toml")).unwrap();
    assert_eq!(content, "# existing");
}

#[test]
fn write_defaults_overwrites_with_force() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mindtape");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("defaults.toml"), "# old").unwrap();
    write_defaults(&target, true).unwrap();
    let content = std::fs::read_to_string(target.join("defaults.toml")).unwrap();
    assert_eq!(content, mindtape::embedded_defaults());
}

/// Regression: the seeded floor text parses and validates on its own — the
/// same guarantee `init_tests.rs` asserts for the slim/full project scaffolds.
#[test]
fn seeded_defaults_parses_and_validates() {
    let config = mindtape::parse_config(mindtape::embedded_defaults()).unwrap();
    config.validate().unwrap();
}
