#![allow(clippy::unwrap_used)]

use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;
use std::path::Path;

use super::{
    HOOKS, completions_dir, write_completions, write_defaults, write_schema, xdg_schema_path,
};

/// A file installer under test: `(dir, force) -> Result`.
type Writer = fn(&Path, bool) -> crate::diag::Result<()>;

/// `writer` creates `dir/name` with exactly `expected` when absent.
fn assert_writes_content(name: &str, writer: Writer, expected: &str) {
    let dir = tempdir();
    let target = dir.path().join("mindtape");
    writer(&target, false).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        expected
    );
}

/// An existing `dir/name` survives `writer` without `force`.
fn assert_skips_existing(name: &str, writer: Writer) {
    let dir = tempdir();
    let target = dir.path().join("mindtape");
    std::fs::create_dir_all(&target).unwrap();
    write_file(&target, name, "# existing");
    writer(&target, false).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        "# existing"
    );
}

/// `force` overwrites an existing `dir/name` with exactly `expected`.
fn assert_overwrites(name: &str, writer: Writer, expected: &str) {
    let dir = tempdir();
    let target = dir.path().join("mindtape");
    std::fs::create_dir_all(&target).unwrap();
    write_file(&target, name, "# old");
    writer(&target, true).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join(name)).unwrap(),
        expected
    );
}

#[test]
fn write_defaults_creates_file_when_absent() {
    let dir = tempdir();
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

/// `xdg_schema_path` returns a path ending with `config.schema.json`,
/// not a `Default::default()` (empty) `PathBuf`.
#[test]
fn xdg_schema_path_ends_with_config_schema_json() {
    let path = xdg_schema_path().unwrap();
    assert!(path.ends_with("config.schema.json"));
}

/// Regression: the seeded floor text parses and validates on its own — the
/// same guarantee `cmd/init/tests.rs` asserts for the slim/full project scaffolds.
#[test]
fn seeded_defaults_parses_and_validates() {
    let config = mindtape::parse_config(mindtape::embedded_defaults()).unwrap();
    config.validate().unwrap();
}

/// `completions_dir` composes `<config home>/mindtape/completions`, not an
/// empty `Default::default()` path.
#[test]
fn completions_dir_ends_with_the_completions_folder() {
    let path = completions_dir().unwrap();
    assert_eq!(
        path.file_name().and_then(std::ffi::OsStr::to_str),
        Some("completions"),
        "{path:?}"
    );
    assert!(path.to_string_lossy().contains("mindtape"), "{path:?}");
}

/// `write_completions` actually writes every hook `HOOKS` declares — a no-op
/// leaves a shell that installed completions pointing at nothing.
#[test]
fn write_completions_writes_a_hook_per_shell() {
    let dir = tempdir();
    write_completions(dir.path(), false).unwrap();
    for (shell, hook) in HOOKS {
        assert_eq!(
            std::fs::read_to_string(dir.path().join(shell)).unwrap(),
            *hook,
            "{shell}"
        );
    }
}

/// Env marker gating the inner side of the `run` completion tests below.
const RUN_COMPLETIONS_INNER: &str = "_MT_INSTALL_RUN_COMPLETIONS";

/// Re-run the calling test in a child with the config/data homes pointed at
/// `home`.
fn run_install_inner(home: &Path) {
    let out = mindtape::test_support::rerun_isolated_with(RUN_COMPLETIONS_INNER, |child| {
        child
            .env(mindtape::CONFIG_HOME_ENV, home.join("config"))
            .env(mindtape::DATA_HOME_ENV, home.join("data"));
    });
    assert!(out.is_some(), "only the parent runs the child");
}

/// A bare `install` writes the completion hooks. Pins the `!` in
/// `if !args.no_completions`: the inverted guard skips them.
#[test]
fn run_writes_completion_hooks_by_default() {
    if std::env::var(RUN_COMPLETIONS_INNER).is_ok() {
        super::run(&super::InstallArgs {
            no_lib: true,
            no_defaults: true,
            no_schema: true,
            no_completions: false,
            force: false,
        })
        .unwrap();
        return;
    }
    let home = tempdir();
    run_install_inner(home.path());
    assert!(
        home.path()
            .join("config/mindtape/completions/bash")
            .exists()
    );
}

/// `--no-completions` is the only thing that skips them.
#[test]
fn run_with_no_completions_writes_no_hooks() {
    if std::env::var(RUN_COMPLETIONS_INNER).is_ok() {
        super::run(&super::InstallArgs {
            no_lib: true,
            no_defaults: true,
            no_schema: true,
            no_completions: true,
            force: false,
        })
        .unwrap();
        return;
    }
    let home = tempdir();
    run_install_inner(home.path());
    assert!(
        !home
            .path()
            .join("config/mindtape/completions/bash")
            .exists()
    );
}
