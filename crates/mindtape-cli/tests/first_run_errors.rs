//! The first errors a new user meets each name the fix, not only the symptom:
//! a task file that cannot find the prelude, a config with an unknown table,
//! and a command run outside any project.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

mod support;

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

#[test]
fn add_without_the_prelude_names_mt_install_and_the_local_package_pin() {
    let home = tempfile::tempdir().unwrap();
    let project = home.path().join("p");
    std::fs::create_dir_all(&project).unwrap();
    let (ok, _) = run_fresh(&project, &["init", "--no-input"]);
    assert!(ok, "mt init failed");

    let (ok, stderr) = run_fresh(&project, &["add", "Fix login"]);

    assert!(!ok, "{stderr}");
    assert!(stderr.contains("did not load"), "{stderr}");
    assert_eq!(
        stderr
            .matches(
                "note: `@local/mindtape` is not installed — run `mt install` (once per \
                 machine) or pin the prelude under `[[typst.local-packages]]`, then try again"
            )
            .count(),
        1,
        "{stderr}"
    );
}

#[test]
fn unknown_config_table_is_reported_once_with_its_line_and_column() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n\n[taks]\nx = 1\n",
    )
    .unwrap();

    let (ok, stderr) = run_fresh(dir.path(), &["ls"]);

    assert!(!ok, "{stderr}");
    assert_eq!(
        stderr.matches("unknown field `taks`").count(),
        1,
        "{stderr}"
    );
    assert!(stderr.contains("line 4, column 2"), "{stderr}");
}

#[test]
fn a_syntax_error_in_the_config_is_reported_once() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape/config.toml"), "oops =\n").unwrap();

    let (ok, stderr) = run_fresh(dir.path(), &["ls"]);

    assert!(!ok, "{stderr}");
    assert_eq!(stderr.matches("line 1, column 7").count(), 1, "{stderr}");
}

#[test]
fn outside_a_project_the_error_names_mt_init_and_the_worktree_flag() {
    let dir = tempfile::tempdir().unwrap();

    let (ok, stderr) = run_fresh(dir.path(), &["ls"]);

    assert!(!ok, "{stderr}");
    assert_eq!(
        stderr,
        "Error: not inside a MindTape project (no .mindtape/ directory found) — run \
         `mt init` to make one here, or pass `-w <path>` to use another\n"
    );
}
