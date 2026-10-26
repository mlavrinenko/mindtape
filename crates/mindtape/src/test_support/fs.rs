//! Tempdirs, project files, the in-tree Typst package and git seeds.
#![allow(clippy::disallowed_methods)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// The in-tree `lib/prelude.typ`, read once here so no test spells its own
/// path to it.
pub const PRELUDE: &str = include_str!("../../../../lib/prelude.typ");

/// Write the `mindtape` package (its `typst.toml` plus `prelude.typ` from
/// `prelude`) into `dir/lib`. A project resolves it from there only through a
/// `[[typst.local-packages]]` entry, such as [`LIB_ENTRY`] appended to its
/// config. Callers pass the real prelude, [`PRELUDE`].
///
/// # Panics
/// Panics if writing the package files under `dir` fails — a test-only helper,
/// so an IO failure against a fresh tempdir is a test-harness bug, not a case
/// to recover from.
pub fn write_lib_package(dir: &Path, prelude: &str) {
    write_package(&dir.join("lib"), prelude);
}

/// The config entry that resolves `@local/mindtape` from the `lib/` that
/// [`write_lib_package`] writes. Append it to a config text.
pub const LIB_ENTRY: &str = "\n[[typst.local-packages]]\nname = \"mindtape\"\npath = \"lib\"\n";

/// Write the `mindtape` package at version 0.2.0 into `pkg`, which becomes
/// the package directory itself: a project's `lib/`, or the
/// `local/mindtape/0.2.0` an install writes under a package root.
///
/// # Panics
/// Panics if writing the package files fails, a test-harness bug.
#[allow(clippy::unwrap_used)]
pub fn write_package(pkg: &Path, prelude: &str) {
    std::fs::create_dir_all(pkg).unwrap();
    std::fs::write(
        pkg.join("typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    std::fs::write(pkg.join("prelude.typ"), prelude).unwrap();
}

/// A fresh tempdir, removed when the returned guard drops. The setup line
/// nearly every test that touches the disk opens with.
///
/// # Panics
/// Panics if the OS refuses a temp directory, a test-harness bug.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// A fresh [`tempdir`] with each of `dirs` (relative paths) created in it.
///
/// # Panics
/// Panics if creating a directory fails, a test-harness bug.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn tempdir_with_dirs(dirs: &[&str]) -> tempfile::TempDir {
    let dir = tempdir();
    for rel in dirs {
        std::fs::create_dir_all(dir.path().join(rel)).unwrap();
    }
    dir
}

/// Write `contents` to `rel` under `root`, creating its parent directories.
///
/// # Panics
/// Panics if a directory or the file cannot be written, a test-harness bug.
#[allow(clippy::unwrap_used)]
pub fn write_file(root: impl AsRef<Path>, rel: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
    let path = root.as_ref().join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

/// A fresh [`tempdir`] with each `(rel, contents)` file written in it, its
/// parent directories created.
///
/// # Panics
/// Panics if writing a file fails, a test-harness bug.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn tempdir_with_files(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempdir();
    for (rel, contents) in files {
        write_file(dir.path(), rel, contents);
    }
    dir
}

/// A fresh [`tempdir`] holding `.mindtape/config.toml` with `config` and
/// nothing else: no lib, no task root.
///
/// # Panics
/// Panics if writing the file fails, a test-harness bug.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn tempdir_with_config(config: &str) -> tempfile::TempDir {
    let dir = tempdir_with_dirs(&[".mindtape"]);
    std::fs::write(dir.path().join(".mindtape/config.toml"), config).unwrap();
    dir
}

/// A fresh tempdir with an (empty) `home/project` tree already created,
/// `project` carrying an empty `.mindtape/` (no `config.toml`/`defaults.toml`
/// of its own — every caller writes the layer text its scenario needs). Keep
/// the returned `TempDir` alive for as long as the paths are used. Shared by
/// `config/layers_tests.rs` and `config/packs_tests.rs`, which both build a
/// `home`/ancestor/project chain to drive `discover_layers`/`load_config_layered`
/// against.
///
/// # Panics
/// Panics if creating either directory fails — a test-harness bug against a
/// fresh tempdir, not a case to recover from.
///
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn home_and_project() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempdir();
    let home = tmp.path().join("home");
    let project = home.join("project");
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    (tmp, home, project)
}

/// Write the barebones project scaffold under `dir`: a `.mindtape/config.toml`
/// declaring a single `tasks` root, plus the empty root itself. The prologue
/// every test that needs a real project on disk repeats, and deliberately
/// nothing more — a scenario-specific config overwrites the file, and the lib
/// override ([`write_lib_package`]) is a separate decision, since a test about
/// package resolution wants a project WITHOUT one.
///
/// # Panics
/// Panics if writing under `dir` fails — a test-only helper, so an IO failure
/// against a fresh tempdir is a test-harness bug, not a case to recover from.
#[allow(clippy::unwrap_used)]
pub fn write_bare_project(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
}

/// `git init --quiet` in `dir`, asserting success. Shared by every test that
/// drives a real git working copy — `changed()` in the filter language
/// reads one, and both the in-crate unit tests and the CLI's e2e tests need
/// the exact same three-line setup to get one.
///
/// # Panics
/// Panics if `git init` fails — a test-harness bug against a fresh tempdir,
/// not a case to recover from.
#[allow(clippy::unwrap_used)]
pub fn git_init(dir: &Path) {
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(dir)
            .status()
            .unwrap()
            .success(),
        "git init must succeed in a fresh tempdir"
    );
}

/// Run `git <args>` in `dir` and return its stdout, trimmed.
///
/// # Panics
/// Panics if git fails or prints anything but UTF-8.
#[allow(clippy::unwrap_used)]
#[must_use]
pub fn git_stdout(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?} must succeed");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

/// Run `git <args>` in `root` as a throwaway identity with signing off,
/// asserting success — the committing half of [`git_init`] for tests that
/// build a history.
///
/// # Panics
/// Panics if git fails.
#[allow(clippy::unwrap_used)]
pub fn git_as_tester(root: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(root)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?} must succeed");
}
