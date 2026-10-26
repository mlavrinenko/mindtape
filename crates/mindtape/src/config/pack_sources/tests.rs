#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::read_raw;
use crate::config::ConfigError;
use crate::config::layers::load_config_layered_from;
use crate::test_support::{tempdir, tempdir_with_files as project, write_file};

const SOURCE: &str =
    "[pack-sources.acme]\ngit = \"https://example.com/acme/packs\"\nref = \"v1\"\n";

const LOCK: &str = "[acme]\ngit = \"https://example.com/acme/packs\"\nref = \"v1\"\n\
    commit = \"3f2c1a9e0b7d4c55e1a2b3c4d5e6f708192a3b4c\"\n\
    sha256 = \"9c41aa00bb11cc22dd33ee44ff5566778899aabbccddeeff0011223344556677\"\n";

const STALE: &str = "pack source lock is stale: 'acme'";

/// Load `root` with no ancestor walk past it and `xdg` as the user layer.
fn load(root: &Path, xdg: Option<&Path>) -> Result<crate::config::Config, ConfigError> {
    load_config_layered_from(root, Some(root), xdg).map(|(config, _, _)| config)
}

/// The error loading a project of `files` gives, with `xdg_defaults` as the
/// user layer's `mindtape/defaults.toml` when given.
fn load_err(files: &[(&str, &str)], xdg_defaults: Option<&str>) -> ConfigError {
    let root = project(files);
    let xdg = tempdir();
    if let Some(text) = xdg_defaults {
        write_file(xdg.path(), "mindtape/defaults.toml", text);
    }
    match load(root.path(), Some(xdg.path())) {
        Ok(_) => panic!("expected the config to fail to load"),
        Err(err) => err,
    }
}

/// The message and hint of the invalid config `files` make.
fn refusal(files: &[(&str, &str)], xdg_defaults: Option<&str>) -> (String, Option<&'static str>) {
    match load_err(files, xdg_defaults) {
        ConfigError::Invalid { message, hint, .. } => (message, hint),
        other => panic!("expected an invalid config, got {other:?}"),
    }
}

/// The message of a project of `config` and `lock`.
fn refused(config: &str, lock: &str) -> String {
    let files = [
        (".mindtape/config.toml", config),
        (".mindtape/packs.lock", lock),
    ];
    refusal(&files, None).0
}

#[test]
fn a_locked_source_loads_into_the_config() {
    let root = project(&[
        (".mindtape/config.toml", SOURCE),
        (".mindtape/packs.lock", LOCK),
    ]);
    let config = load(root.path(), None).unwrap();
    let acme = &config.pack_sources["acme"];
    assert_eq!(acme.git, "https://example.com/acme/packs");
    assert_eq!(acme.reference, "v1");
}

#[test]
fn a_source_takes_only_git_and_ref() {
    let config = format!("{SOURCE}branch = \"main\"\n");
    let err = load_err(&[(".mindtape/config.toml", &config)], None);
    assert!(
        err.to_string().contains("unknown field `branch`"),
        "{err:?}"
    );
}

#[test]
fn a_source_name_is_refused_unless_extends_can_spell_it() {
    let reserved = refused(
        &SOURCE.replace("acme", "mindtape"),
        &LOCK.replace("acme", "mindtape"),
    );
    assert_eq!(
        reserved,
        "pack-sources: the name is taken by the packs built into mt: 'mindtape'"
    );
    let pathlike = refused(&SOURCE.replace("acme]", "\"a/b\"]"), "");
    assert_eq!(
        pathlike,
        "pack-sources: a name takes letters, digits, '-' and '_': 'a/b'"
    );
}

#[test]
fn only_the_project_layer_declares_a_source() {
    let (xdg, hint) = refusal(&[], Some(SOURCE));
    assert!(xdg.starts_with("pack-sources: declared outside the project config: '"));
    assert!(xdg.ends_with("defaults.toml'"), "{xdg}");
    assert!(hint.is_some_and(|hint| hint.contains(".mindtape/config.toml")));
    let (ancestor, _) = refusal(&[(".mindtape/defaults.toml", SOURCE)], None);
    assert!(
        ancestor.starts_with("pack-sources: declared outside"),
        "{ancestor}"
    );
}

#[test]
fn a_pack_may_not_declare_a_source() {
    let files = [
        (".mindtape/config.toml", "extends = [\"./team.toml\"]\n"),
        ("team.toml", SOURCE),
    ];
    let (message, _) = refusal(&files, None);
    assert!(
        message.ends_with("may not declare: 'pack-sources'"),
        "{message}"
    );
}

#[test]
fn a_source_the_lock_lacks_is_stale() {
    let (message, hint) = refusal(&[(".mindtape/config.toml", SOURCE)], None);
    assert_eq!(message, STALE);
    assert!(hint.is_some_and(|hint| hint.contains("mt packs update")));
}

#[test]
fn a_source_locked_from_another_git_or_ref_is_stale() {
    assert_eq!(
        refused(SOURCE, &LOCK.replace("acme/packs", "acme/other")),
        STALE
    );
    assert_eq!(refused(SOURCE, &LOCK.replace("\"v1\"", "\"v2\"")), STALE);
}

#[test]
fn a_lock_entry_no_source_declares_is_not_stale() {
    let root = project(&[(".mindtape/packs.lock", LOCK)]);
    assert!(load(root.path(), None).is_ok());
}

#[test]
fn a_lock_id_that_is_not_hex_of_its_length_is_refused() {
    let hash = refused(SOURCE, &LOCK.replace("9c41aa00", "../../.."));
    assert!(hash.contains(": acme.sha256: "), "{hash}");
    let commit = refused(SOURCE, &LOCK.replace("3f2c1a9e", "3F2C1A9E"));
    assert!(commit.contains(": acme.commit: "), "{commit}");
}

#[test]
fn a_lock_that_is_not_toml_names_the_file() {
    let files = [
        (".mindtape/config.toml", SOURCE),
        (".mindtape/packs.lock", "[acme\n"),
    ];
    let err = load_err(&files, None);
    assert!(
        matches!(&err, ConfigError::ParseIn { origin, .. } if origin.ends_with("packs.lock")),
        "{err:?}"
    );
}

#[test]
fn a_declared_source_outside_the_project_layer_is_refused() {
    let files = [
        (".mindtape/config.toml", SOURCE),
        (".mindtape/packs.lock", LOCK),
    ];
    let (message, _) = refusal(&files, Some("extends = [\"acme:commits\"]\n"));
    assert_eq!(
        message,
        "extends: only the project config may use pack source: 'acme'"
    );
}

#[test]
fn an_undeclared_source_outside_the_project_layer_is_unknown() {
    let ancestor = (".mindtape/defaults.toml", "extends = [\"acme:commits\"]\n");
    let (message, _) = refusal(&[ancestor], None);
    assert_eq!(message, "extends: unknown pack source: 'acme'");
}

#[test]
fn a_declared_locked_source_is_not_installed_yet() {
    let config = format!("extends = [\"acme:commits\"]\n\n{SOURCE}");
    let files = [
        (".mindtape/config.toml", config.as_str()),
        (".mindtape/packs.lock", LOCK),
    ];
    let (message, hint) = refusal(&files, None);
    assert_eq!(message, "extends: pack source not installed: 'acme'");
    assert!(hint.is_some_and(|hint| hint.contains("mt packs install")));
}

#[test]
fn read_raw_reads_sources_and_lock_without_resolving_extends() {
    let config = format!("extends = [\"mindtape:nope\"]\n\n{SOURCE}");
    let lock = LOCK.replace("\"v1\"", "\"v0\"");
    let root = project(&[
        (".mindtape/config.toml", &config),
        (".mindtape/packs.lock", &lock),
    ]);
    let raw = read_raw(root.path()).unwrap();
    assert_eq!(raw.sources["acme"].reference, "v1");
    assert_eq!(raw.lock.entries["acme"].reference, "v0");
    assert_eq!(raw.stale(), Some("acme"));
}

#[test]
fn read_raw_without_a_config_or_lock_is_empty() {
    let raw = read_raw(tempdir().path()).unwrap();
    assert!(raw.sources.is_empty() && raw.lock.entries.is_empty());
    assert_eq!(raw.stale(), None);
}

#[test]
fn a_lock_or_config_that_cannot_be_read_is_an_io_error() {
    let lock_dir = project(&[(".mindtape/config.toml", SOURCE)]);
    std::fs::create_dir(lock_dir.path().join(".mindtape/packs.lock")).unwrap();
    let err = load(lock_dir.path(), None).err().unwrap();
    assert!(matches!(err, ConfigError::Io(_)), "{err:?}");
    assert!(matches!(read_raw(lock_dir.path()), Err(ConfigError::Io(_))));

    let config_dir = tempdir();
    std::fs::create_dir_all(config_dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(matches!(
        read_raw(config_dir.path()),
        Err(ConfigError::Io(_))
    ));
}
