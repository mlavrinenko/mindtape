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
