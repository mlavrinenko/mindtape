//! End-to-end tests for the `install` command that shell out to the `mt`
//! binary, exercising real app-directory resolution
//! (`mindtape::config_home()` / `mindtape::data_home()`).
//!
//! Staged through `$MINDTAPE_CONFIG_HOME`/`$MINDTAPE_DATA_HOME` rather than the
//! XDG pair: `dirs` reads the environment on Unix only, so an XDG-injected
//! directory is not where a Windows `mt install` writes and every assertion
//! below would be about a path nothing touched.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;
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

/// Each schema `mt install` writes, beside `config.schema.json`, with the
/// text it must hold.
fn schemas(home: &Path) -> [(PathBuf, String); 2] {
    let config = schema_path(home);
    let at = config.with_file_name("at.schema.json");
    [
        (config, mindtape::config_schema_json()),
        (at, mindtape::at::schema_json()),
    ]
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
/// `defaults.toml`, and both JSON Schemas.
#[test]
fn install_writes_lib_defaults_and_schema() {
    let home = tempdir();
    run_install(home.path(), &["install"]).success();

    assert!(lib_typst_toml(home.path()).exists());
    assert!(lib_prelude(home.path()).exists());
    let content = std::fs::read_to_string(defaults_path(home.path())).unwrap();
    assert_eq!(content, mindtape::embedded_defaults());
    for (path, expected) in schemas(home.path()) {
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            expected,
            "{path:?}"
        );
    }
}

/// The files each `--no-*` flag skips, by flag: every other part still lands.
fn parts(home: &Path) -> [(&'static str, Vec<PathBuf>); 3] {
    [
        ("--no-lib", vec![lib_typst_toml(home), lib_prelude(home)]),
        ("--no-defaults", vec![defaults_path(home)]),
        (
            "--no-schema",
            schemas(home).into_iter().map(|(path, _)| path).collect(),
        ),
    ]
}

/// Each of `--no-lib`, `--no-defaults` and `--no-schema` skips its own files
/// and only those.
#[test]
fn each_skip_flag_skips_only_its_part() {
    for (flag, _) in parts(Path::new("")) {
        let home = tempdir();
        run_install(home.path(), &["install", flag]).success();
        for (part, paths) in parts(home.path()) {
            for path in paths {
                assert_eq!(path.exists(), part != flag, "{flag}: {path:?}");
            }
        }
    }
}

/// An existing `config.schema.json` survives a bare `mt install`, and `--force`
/// rewrites it.
#[test]
fn schema_force_interplay() {
    let home = tempdir();
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
    let home = tempdir();
    let path = seed_existing_defaults(home.path());

    run_install(home.path(), &["install"]).success();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "# existing");
}

/// `--force` overwrites an existing XDG `defaults.toml`.
#[test]
fn existing_defaults_overwritten_with_force() {
    let home = tempdir();
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
    let home = tempdir();
    run_install(home.path(), &["install"]).success();

    let content = std::fs::read_to_string(defaults_path(home.path())).unwrap();
    let config = mindtape::parse_config(&content).unwrap();
    config.validate().unwrap();
}

/// `install-lib` is gone, not merely renamed: it is neither a recognised
/// subcommand nor a legend shortcut verb, so it falls through to the same
/// "unknown command" error any other typo does, naming nothing in
/// particular to replace it with.
#[test]
fn install_lib_is_an_unknown_command() {
    let home = tempdir();
    let assert = run_install(home.path(), &["install-lib"]).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("unknown command: 'install-lib'"),
        "{stderr}"
    );
}

/// Every skip flag `InstallArgs` declares is named in `install`'s own long
/// help, which is the text `mt _cli --json` carries into the docs site's CLI
/// reference. `--no-schema` shipped without ever reaching that sentence, so
/// the site rendered "performs both by default" directly above the
/// hand-written note listing all three — the derived reference contradicting
/// itself on one page. Deriving the reference from clap moved the drift into
/// the doc comment, where nothing was watching; this is the thing watching.
#[test]
fn install_long_help_names_every_skip_flag() {
    let home = tempdir();
    let assert = run_install(home.path(), &["install", "--help"]).success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // The flag list under Options: is clap's own render of the struct, so
    // split there and read only the prose above it — the half a human writes.
    let long_about = output.split("Usage:").next().unwrap();
    for flag in ["--no-lib", "--no-defaults", "--no-schema"] {
        assert!(
            long_about.contains(flag),
            "`mt install`'s long help never mentions {flag}:\n{long_about}"
        );
    }
}

/// A project with no in-tree `lib/`, so its one task can only load through the
/// data home's installed package. Returned so the caller keeps the tempdir
/// alive.
fn project_without_a_lib() -> tempfile::TempDir {
    let dir = tempdir();
    mindtape::test_support::write_bare_project(dir.path());
    write_file(
        dir.path(),
        "tasks/only.typ",
        mindtape::typst_task!("(title: \"Only task\")\n"),
    );
    dir
}

/// `mt ls` in `project`, with the data home pointed at `data` (verbatim, so an
/// empty value stays empty) and the global config layer out of reach.
fn ls_with_data_home(project: &Path, data: &std::ffi::OsStr) -> assert_cmd::assert::Assert {
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(project)
        .env(mindtape::CONFIG_HOME_ENV, project.join("no-global-config"))
        .env(mindtape::DATA_HOME_ENV, data)
        .args(["--color", "never", "ls"])
        .assert()
}

/// The install and the read must name the same directory. `$MINDTAPE_DATA_HOME`
/// moved where `mt install` WRITES the `@local/mindtape` package long before it
/// moved where the loader READS it — resolution went through `typst-world`,
/// which looked the directory up through `dirs` and had never heard of the
/// variable. Pointed at a tempdir, `mt install` filled that tempdir while every
/// task file kept importing the machine's own package.
///
/// The pair is what makes this a test rather than a coincidence: a data home
/// that HAS the package loads the task, and a second one that does not fails
/// naming its own path. Before the fix both answers came from the machine, so
/// they were identical whichever way the variable pointed.
#[test]
fn ls_resolves_the_prelude_from_the_configured_data_home() {
    let home = tempdir();
    run_install(home.path(), &["install", "--no-defaults", "--no-schema"]).success();
    let project = project_without_a_lib();

    let assert = ls_with_data_home(project.path(), home.path().join("data").as_os_str()).success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Only task"), "{stdout}");
}

#[test]
fn ls_fails_against_a_data_home_the_package_was_not_installed_into() {
    let installed = tempdir();
    run_install(
        installed.path(),
        &["install", "--no-defaults", "--no-schema"],
    )
    .success();
    let elsewhere = tempdir();
    let project = project_without_a_lib();

    let assert = ls_with_data_home(project.path(), elsewhere.path().as_os_str());
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains(&elsewhere.path().display().to_string()),
        "the failure must name the data home that was searched:\n{stderr}"
    );
}

/// Set but empty is `app_dirs`'s "no such directory", and a read honours it the
/// same way a write does: no global package resolves at all, even on a machine
/// that has one installed. What `just check-examples` leans on to prove each
/// example fixture resolves its prelude from the checkout beside it.
#[test]
fn an_empty_data_home_resolves_no_global_package() {
    let project = project_without_a_lib();

    let assert = ls_with_data_home(project.path(), std::ffi::OsStr::new(""));
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("@local/mindtape"),
        "the failure must name the unresolved package:\n{stderr}"
    );
}

/// A prelude that does not resolve fails every task file the same way. The
/// listing says so once, naming the command and the config key, not once per
/// file.
#[test]
fn a_missing_prelude_is_one_error_however_many_tasks_fail() {
    let project = project_without_a_lib();
    write_file(
        project.path(),
        "tasks/second.typ",
        mindtape::typst_task!("(title: \"Second task\")\n"),
    );

    let assert = ls_with_data_home(project.path(), std::ffi::OsStr::new(""));
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr.matches("can't find the prelude package").count(),
        1,
        "{stderr}"
    );
    assert!(
        stderr.contains("hint: install it once per machine: mt install"),
        "{stderr}"
    );
    assert!(stderr.contains("[[typst.local-packages]]"), "{stderr}");
}
