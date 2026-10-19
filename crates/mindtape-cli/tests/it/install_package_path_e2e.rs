//! With `$MINDTAPE_DATA_HOME` unset, `mt install` writes the `@local/mindtape`
//! package where Typst and tinymist read `@local` packages, and loading a
//! project reads it back from there.
//!
//! That directory is `$TYPST_PACKAGE_PATH` when set, else the platform data
//! directory joined with `typst/packages`. The variable moves it on every
//! platform, so these tests pin it. `XDG_DATA_HOME` points at an empty
//! tempdir, so a lookup that ignores the variable misses on Linux instead of
//! touching the machine's own packages.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};

use mindtape::test_support::{tempdir, write_file};

use crate::support::{stderr_of, stdout_of};

/// The variable Typst's `--package-path` reads its default from.
const PACKAGE_PATH_ENV: &str = "TYPST_PACKAGE_PATH";

/// `mt <args>` in `cwd` with Typst's package directory at `packages`, the
/// data home unset, and the config home and XDG data dir under `home`.
fn mt_with_package_path(
    cwd: &Path,
    home: &Path,
    packages: &Path,
    args: &[&str],
) -> assert_cmd::assert::Assert {
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(cwd)
        .env_remove(mindtape::DATA_HOME_ENV)
        .env(mindtape::CONFIG_HOME_ENV, home.join("config"))
        .env("XDG_DATA_HOME", home.join("xdg-data"))
        .env(PACKAGE_PATH_ENV, packages)
        .args(["--color", "never"])
        .args(args)
        .assert()
}

/// The directory `mt install`'s `installed @local/mindtape:<v> -> <dir>` line
/// names.
fn installed_dir(stdout: &str) -> PathBuf {
    let line = stdout
        .lines()
        .find(|line| line.starts_with("installed @local/mindtape:"))
        .unwrap_or_else(|| panic!("no install line:\n{stdout}"));
    PathBuf::from(line.split_once(" -> ").unwrap().1)
}

#[test]
fn install_writes_the_prelude_under_typsts_package_path() {
    let home = tempdir();
    let packages = home.path().join("typst-packages");

    let assert = mt_with_package_path(
        home.path(),
        home.path(),
        &packages,
        &[
            "install",
            "--no-defaults",
            "--no-schema",
            "--no-completions",
        ],
    )
    .success();

    let stdout = stdout_of(&assert);
    let dir = installed_dir(&stdout);
    assert!(
        dir.starts_with(packages.join("local").join("mindtape")),
        "{stdout}"
    );
    assert!(dir.join("prelude.typ").is_file(), "{stdout}");
    assert!(dir.join("typst.toml").is_file(), "{stdout}");
    assert!(!home.path().join("xdg-data").exists(), "{stdout}");
}

#[test]
fn ls_reads_the_prelude_from_typsts_package_path() {
    let staged = tempdir();
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .env(mindtape::DATA_HOME_ENV, staged.path())
        .env(mindtape::CONFIG_HOME_ENV, staged.path().join("config"))
        .args([
            "install",
            "--no-defaults",
            "--no-schema",
            "--no-completions",
        ])
        .assert()
        .success();
    let packages = staged.path().join("typst").join("packages");
    let project = tempdir();
    mindtape::test_support::write_bare_project(project.path());
    write_file(
        project.path(),
        "tasks/only.typ",
        mindtape::typst_task!("(title: \"Only task\")\n"),
    );

    let assert = mt_with_package_path(project.path(), project.path(), &packages, &["ls"]).success();

    let stdout = stdout_of(&assert);
    assert!(stdout.contains("Only task"), "{stdout}");
}

/// Set but empty, the data home names no package directory, and it wins over
/// `$TYPST_PACKAGE_PATH`, so `mt install` refuses rather than write anywhere.
#[test]
fn install_refuses_when_no_package_directory_resolves() {
    let home = tempdir();
    let assert = assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .env(mindtape::DATA_HOME_ENV, "")
        .env(mindtape::CONFIG_HOME_ENV, home.path().join("config"))
        .env(PACKAGE_PATH_ENV, home.path().join("typst-packages"))
        .args([
            "--color",
            "never",
            "install",
            "--no-defaults",
            "--no-schema",
        ])
        .args(["--no-completions"])
        .assert()
        .code(1);

    assert_eq!(
        stderr_of(&assert),
        "error: can't find the Typst package directory\n\
         hint: set MINDTAPE_DATA_HOME to a directory\n"
    );
    assert!(!home.path().join("typst-packages").exists());
}
