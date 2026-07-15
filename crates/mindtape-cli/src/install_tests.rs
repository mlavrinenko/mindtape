#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::{write_defaults, write_schema};

/// A file installer under test: `(dir, force) -> Result`.
type Writer = fn(&Path, bool) -> anyhow::Result<()>;

/// `writer` creates `dir/name` with exactly `expected` when absent.
fn assert_writes_content(name: &str, writer: Writer, expected: &str) {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mindtape");
    writer(&target, false).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        expected
    );
}

/// An existing `dir/name` survives `writer` without `force`.
fn assert_skips_existing(name: &str, writer: Writer) {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mindtape");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join(name), "# existing").unwrap();
    writer(&target, false).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        "# existing"
    );
}

/// `force` overwrites an existing `dir/name` with exactly `expected`.
fn assert_overwrites(name: &str, writer: Writer, expected: &str) {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mindtape");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join(name), "# old").unwrap();
    writer(&target, true).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        expected
    );
}

#[test]
fn write_defaults_creates_file_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mindtape");
    write_defaults(&target, false).unwrap();
    assert!(target.join("defaults.toml").exists());
}

#[test]
fn write_defaults_content_matches_embedded_floor() {
    assert_writes_content(
        "defaults.toml",
        write_defaults,
        mindtape::embedded_defaults(),
    );
}

#[test]
fn write_defaults_skips_existing_without_force() {
    assert_skips_existing("defaults.toml", write_defaults);
}

#[test]
fn write_defaults_overwrites_with_force() {
    assert_overwrites(
        "defaults.toml",
        write_defaults,
        mindtape::embedded_defaults(),
    );
}

#[test]
fn write_schema_content_matches_generated_schema() {
    assert_writes_content(
        "config.schema.json",
        write_schema,
        &mindtape::config_schema_json(),
    );
}

#[test]
fn write_schema_skips_existing_without_force() {
    assert_skips_existing("config.schema.json", write_schema);
}

#[test]
fn write_schema_overwrites_with_force() {
    assert_overwrites(
        "config.schema.json",
        write_schema,
        &mindtape::config_schema_json(),
    );
}

/// Regression: the seeded floor text parses and validates on its own — the
/// same guarantee `init_tests.rs` asserts for the slim/full project scaffolds.
#[test]
fn seeded_defaults_parses_and_validates() {
    let config = mindtape::parse_config(mindtape::embedded_defaults()).unwrap();
    config.validate().unwrap();
}
