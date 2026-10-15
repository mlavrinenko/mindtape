//! `mt rm <folder>`: a directory under a task root removes every task inside.
#![allow(clippy::unwrap_used)]

use super::*;

/// A project with two tasks in `tasks/batch/` (one a level deeper) and one
/// beside the folder, committed clean.
fn scaffold_folder() -> tempfile::TempDir {
    let dir = scaffold_files(&[("keep.typ", UNRELATED)]);
    let batch = dir.path().join("tasks/batch");
    std::fs::create_dir_all(batch.join("deeper")).unwrap();
    std::fs::create_dir_all(dir.path().join("tasks/empty")).unwrap();
    for (path, body) in [("gone-a.typ", GONE_A), ("deeper/gone-b.typ", GONE_B)] {
        mindtape::write_atomic(&batch.join(path), body.as_bytes()).unwrap();
    }
    git_commit_all(dir.path());
    dir
}

/// `mt rm` in `cwd` (a path under the project) with `args`, run on a fresh project.
fn rm_in(cwd: &str, args: &[&str]) -> (tempfile::TempDir, assert_cmd::assert::Assert) {
    let dir = scaffold_folder();
    let out = mt(&dir.path().join(cwd)).arg("rm").args(args).assert();
    (dir, out)
}

fn batch_left(dir: &tempfile::TempDir) -> bool {
    dir.path().join("tasks/batch/gone-a.typ").exists()
        || dir.path().join("tasks/batch/deeper/gone-b.typ").exists()
}

#[test]
fn rm_dry_run_lists_every_task_under_a_folder_and_writes_nothing() {
    let (dir, out) = rm_in("", &["--dry-run", "tasks/batch"]);

    out.success()
        .stdout(predicates::str::contains("tasks/batch/gone-a.typ"))
        .stdout(predicates::str::contains("tasks/batch/deeper/gone-b.typ"))
        .stdout(predicates::str::contains("keep.typ").not());
    assert!(batch_left(&dir));
}

#[test]
fn rm_deletes_every_task_under_a_folder_named_by_its_path() {
    // From the project root by a path; from `tasks/` by a bare name no task,
    // term or hash claims.
    for (cwd, spec) in [("", "tasks/batch"), ("tasks", "batch")] {
        let (dir, out) = rm_in(cwd, &["-y", spec]);

        out.success()
            .stdout(predicates::str::contains("deleted 2 tasks"));
        assert!(!batch_left(&dir));
        assert!(dir.path().join("tasks/keep.typ").exists());
    }
}

#[test]
fn rm_refuses_a_folder_outside_every_root() {
    let dir = scaffold_folder();
    std::fs::create_dir_all(dir.path().join("elsewhere")).unwrap();

    for spec in ["elsewhere", "elsewhere/"] {
        mt(dir.path())
            .args(["rm", "-y", spec])
            .assert()
            .failure()
            .stderr(predicates::str::contains("outside every task root"));
    }
    assert!(batch_left(&dir));
}

#[test]
fn rm_of_a_folder_holding_no_task_removes_nothing() {
    let (dir, out) = rm_in("", &["-y", "tasks/empty"]);

    out.success()
        .stdout(predicates::str::contains("nothing to remove"));
    assert!(batch_left(&dir));
}
