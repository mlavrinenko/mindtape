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

/// A freshly `mt init`ed project in a temp dir, on a machine with no prelude.
fn fresh_project() -> (tempfile::TempDir, std::path::PathBuf) {
    let home = tempfile::tempdir().unwrap();
    let project = home.path().join("p");
    std::fs::create_dir_all(&project).unwrap();
    let (ok, _) = run_fresh(&project, &["init", "--no-input"]);
    assert!(ok, "mt init failed");
    (home, project)
}

#[test]
fn add_without_the_prelude_names_mt_install_and_the_local_package_pin() {
    let (_home, project) = fresh_project();

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
        "error: no .mindtape/ directory here or above\n\
         hint: run mt init to make one here, or pass -w <path> to use another project\n"
    );
}

const NOTE: &str = "note: `@local/mindtape` is not installed — run `mt install` (once per \
                    machine) or pin the prelude under `[[typst.local-packages]]`, then try again";

const ONE_RECORD: &str = r#"[{"id":"","file":"","title":"Fix login","status":{"name":"proposed","date":"2026-06-01","note":null},"priority":null,"difficulty":null,"kind":null,"estimate":{},"links":[],"tags":[],"derived_tags":[],"due":null,"start":null,"reviewed":null,"body":"","extras":{}}]"#;

#[test]
fn import_without_the_prelude_prints_the_install_note_once_before_the_skip_count() {
    let (_home, project) = fresh_project();
    let record = &ONE_RECORD[1..ONE_RECORD.len() - 1];
    let two = format!("[{},{}]", record, record.replace("Fix login", "Fix logout"));
    std::fs::write(project.join("in.json"), two).unwrap();

    let (ok, stderr) = run_fresh(&project, &["import", "in.json"]);

    assert!(!ok, "{stderr}");
    assert_eq!(stderr.matches(NOTE).count(), 1, "{stderr}");
    let note_at = stderr.find(NOTE).unwrap();
    assert!(stderr.rfind("skipped").unwrap() > note_at, "{stderr}");
    assert!(stderr.rfind("did not load").unwrap() < note_at, "{stderr}");
}

#[test]
fn import_skipped_for_another_reason_prints_no_install_note() {
    let (_home, project) = fresh_project();
    std::fs::write(project.join("in.json"), ONE_RECORD).unwrap();
    support::install_lib(&project);

    let (_, stderr) = run_fresh(&project, &["import", "in.json"]);

    assert!(!stderr.contains(NOTE), "{stderr}");
}

/// A project at `dir/p` with its own valid config, ready for a broken layer.
fn project_with_config(dir: &Path) -> std::path::PathBuf {
    let project = dir.join("p");
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    project
}

#[test]
fn a_syntax_error_in_the_global_defaults_names_that_file() {
    let dir = tempfile::tempdir().unwrap();
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
            defaults.display()
        )),
        "{stderr}"
    );
    assert_eq!(stderr.matches(":1:7:").count(), 1, "{stderr}");
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
}

#[test]
fn a_syntax_error_in_a_path_pack_names_the_pack_file() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_config(dir.path());
    std::fs::write(
        project.join(".mindtape/config.toml"),
        "extends = [\"./pack.toml\"]\n[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    let pack = project.join("./pack.toml");
    std::fs::write(&pack, "oops =\n").unwrap();

    let (ok, stderr) = run_fresh(&project, &["ls"]);

    assert!(!ok, "{stderr}");
    assert!(
        stderr.contains(&format!("{}:1:7: can't parse config: ", pack.display())),
        "{stderr}"
    );
}
