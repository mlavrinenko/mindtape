//! The first errors a new user meets each name the fix, not only the symptom:
//! a task file that cannot find the prelude, a config with an unknown table,
//! and a command run outside any project.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use mindtape::test_support::{tempdir, tempdir_with_config};
use std::path::Path;

use crate::support;

/// `mt <args>` in `dir`, isolated, with a data home that holds no prelude —
/// a machine where `mt install` never ran. Returns stderr and whether it
/// succeeded.
fn run_fresh(dir: &Path, args: &[&str]) -> (bool, String) {
    let output = support::mt(dir)
        .env(mindtape::DATA_HOME_ENV, dir.join("empty-data"))
        .args(args)
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// A freshly `mt init`ed project in a temp dir, on a machine with no prelude.
fn fresh_project() -> (tempfile::TempDir, std::path::PathBuf) {
    let home = tempdir();
    let project = home.path().join("p");
    std::fs::create_dir_all(&project).unwrap();
    let (ok, _) = run_fresh(&project, &["init", "--no-input"]);
    assert!(ok, "mt init failed");
    (home, project)
}

/// Point `project`'s default template at a prelude version this `mt` doesn't
/// embed, as a project pinned to another release does, so only an installed
/// copy could serve it.
fn pin_an_unembedded_prelude(project: &Path) {
    const BUILTIN: &str = include_str!("../../src/cmd/add/templates/task.typ");
    let pinned = BUILTIN.replacen("@local/mindtape:0.2.0", "@local/mindtape:0.1.0", 1);
    assert_ne!(
        pinned, BUILTIN,
        "the built-in template no longer imports 0.2.0"
    );
    write_file(project, ".mindtape/templates/task.typ", &pinned);
}

#[test]
fn add_on_a_fresh_machine_files_the_task_on_the_embedded_prelude() {
    let (_home, project) = fresh_project();

    let (added, stderr) = run_fresh(&project, &["add", "Fix login"]);
    assert!(added, "{stderr}");
    let (checked, stderr) = run_fresh(&project, &["check"]);
    assert!(checked, "{stderr}");
    assert!(!stderr.contains("prelude"), "{stderr}");
}

/// A project template whose prelude doesn't resolve fails the check with one
/// error naming the remedy.
#[test]
fn check_names_mt_install_once_for_a_template_whose_prelude_is_missing() {
    let (_home, project) = fresh_project();
    pin_an_unembedded_prelude(&project);

    let (ok, stderr) = run_fresh(&project, &["check"]);

    assert!(!ok, "{stderr}");
    assert_eq!(
        stderr.matches("can't find the prelude package").count(),
        1,
        "{stderr}"
    );
    assert!(
        stderr.contains("hint: install it once per machine: mt install"),
        "{stderr}"
    );
}

#[test]
fn add_without_the_prelude_names_mt_install_and_the_local_package_pin() {
    let (_home, project) = fresh_project();
    pin_an_unembedded_prelude(&project);

    let (ok, stderr) = run_fresh(&project, &["add", "Fix login"]);

    assert!(!ok, "{stderr}");
    assert!(stderr.contains("did not load"), "{stderr}");
    assert_eq!(
        stderr
            .matches(
                "hint: install it once per machine: mt install\n\
                 hint: or pin a copy under [[typst.local-packages]] in the config\n"
            )
            .count(),
        1,
        "{stderr}"
    );
}

#[test]
fn unknown_config_table_is_reported_once_with_its_line_and_column() {
    let dir = tempdir_with_config("[tasks]\nroots = [\"tasks\"]\n\n[taks]\nx = 1\n");

    let (ok, stderr) = run_fresh(dir.path(), &["ls"]);

    assert!(!ok, "{stderr}");
    assert_eq!(
        stderr.matches("unknown field `taks`").count(),
        1,
        "{stderr}"
    );
    assert!(
        stderr.starts_with(&shown(
            "error: .mindtape/config.toml:4:2: can't parse config"
        )),
        "{stderr}"
    );
}

#[test]
fn a_syntax_error_in_the_config_is_reported_once() {
    let dir = tempdir_with_config("oops =\n");

    let (ok, stderr) = run_fresh(dir.path(), &["ls"]);

    assert!(!ok, "{stderr}");
    assert!(
        stderr.starts_with(&shown(
            "error: .mindtape/config.toml:1:7: can't parse config"
        )),
        "{stderr}"
    );
    assert_eq!(stderr.matches("1:7").count(), 1, "{stderr}");
}

#[test]
fn outside_a_project_the_error_names_mt_init_and_the_worktree_flag() {
    let dir = tempdir();

    let (ok, stderr) = run_fresh(dir.path(), &["ls"]);

    assert!(!ok, "{stderr}");
    assert_eq!(
        stderr,
        "error: no .mindtape/ directory here or above\n\
         hint: run mt init to make one here, or pass -w <path> to use another project\n"
    );
}

const PRELUDE_ERROR: &str = "error: can't find the prelude package: '@local/mindtape'";

const ONE_RECORD: &str = r#"[{"id":"","file":"","title":"Fix login","status":{"name":"proposed","date":"2026-06-01","note":null},"priority":null,"difficulty":null,"kind":null,"estimate":{},"links":[],"tags":[],"derived_tags":[],"due":null,"start":null,"reviewed":null,"body":"","extras":{}}]"#;

/// `mt import` writes the import this `mt` embeds, so a fresh machine files
/// every record.
#[test]
fn import_on_a_fresh_machine_files_every_record() {
    let (_home, project) = fresh_project();
    let record = &ONE_RECORD[1..ONE_RECORD.len() - 1];
    let two = format!("[{},{}]", record, record.replace("Fix login", "Fix logout"));
    write_file(&project, "in.json", &two);

    let (ok, stderr) = run_fresh(&project, &["import", "in.json"]);

    assert!(ok, "{stderr}");
    assert!(!stderr.contains(PRELUDE_ERROR), "{stderr}");
}

#[test]
fn import_skipped_for_another_reason_prints_no_install_note() {
    let (_home, project) = fresh_project();
    write_file(&project, "in.json", ONE_RECORD);
    // Installed where `run_fresh` points the data home, as `mt install` would.
    mindtape::test_support::write_package(
        &project.join("empty-data/typst/packages/local/mindtape/0.2.0"),
        mindtape::test_support::PRELUDE,
    );

    let (_, stderr) = run_fresh(&project, &["import", "in.json"]);

    assert!(!stderr.contains(PRELUDE_ERROR), "{stderr}");
}

/// A project at `dir/p` with its own valid config, ready for a broken layer.
fn project_with_config(dir: &Path) -> std::path::PathBuf {
    let project = dir.join("p");
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    write_file(
        &project,
        ".mindtape/config.toml",
        "[tasks]\nroots = [\"tasks\"]\n",
    );
    project
}

#[test]
fn a_syntax_error_in_the_global_defaults_names_that_file() {
    let dir = tempdir();
    let project = project_with_config(dir.path());
    let config_home = dir.path().join("cfg");
    std::fs::create_dir_all(config_home.join("mindtape")).unwrap();
    let defaults = config_home.join("mindtape/defaults.toml");
    std::fs::write(&defaults, "oops =\n").unwrap();

    let output = support::mt(&project)
        .env(mindtape::CONFIG_HOME_ENV, &config_home)
        .arg("ls")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "{stderr}");
    assert!(
        stderr.contains(&format!(
            "{}:1:7: can't parse config: invalid string, expected",
            mindtape::display_path(&defaults)
        )),
        "{stderr}"
    );
    assert_eq!(stderr.matches(":1:7:").count(), 1, "{stderr}");
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
}

#[test]
fn a_syntax_error_in_a_path_pack_names_the_pack_file() {
    let dir = tempdir();
    let project = project_with_config(dir.path());
    write_file(
        &project,
        ".mindtape/config.toml",
        "extends = [\"./pack.toml\"]\n[tasks]\nroots = [\"tasks\"]\n",
    );
    std::fs::write(project.join("pack.toml"), "oops =\n").unwrap();

    let (ok, stderr) = run_fresh(&project, &["ls"]);

    assert!(!ok, "{stderr}");
    let sep = std::path::MAIN_SEPARATOR;
    assert!(
        stderr.contains(&format!("{sep}pack.toml:1:7: can't parse config: ")),
        "{stderr}"
    );
    assert!(!stderr.contains(&format!(".{sep}pack.toml")), "{stderr}");
}

/// An expected message, each task path in it in the separator `mt` shows.
fn shown(text: &str) -> String {
    support::native_in(text, &[".mindtape/config.toml"])
}
