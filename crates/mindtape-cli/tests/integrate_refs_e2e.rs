//! `just integrate` refuses a branch that names a task in `Refs:` and leaves
//! it `wip` at its tip (`.just/scripts/integrate.sh`). The script asks before
//! it reads the gate record, so these runs never reach `mmz`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};
use std::process::Output;

mod support;

use support::{git, git_commit_all, isolated_env, project_with_tasks};

/// A committed-clean `main` holding the `wip` task `parser`, and a worktree on
/// the new branch `feature` in its own directory, which the second value owns.
fn repo_with_worktree() -> (tempfile::TempDir, tempfile::TempDir, PathBuf) {
    let main = project_with_tasks(&[("parser.typ", "Write the parser")]);
    git_commit_all(main.path());
    let holder = tempfile::tempdir().unwrap();
    let wt = holder.path().join("feature");
    git(
        main.path(),
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature",
            wt.to_str().unwrap(),
        ],
    );
    (main, holder, wt)
}

/// Commit in `wt` with `message`, leaving the tree clean.
fn commit(wt: &Path, message: &str) {
    git(
        wt,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-a",
            "--allow-empty",
            "-m",
            message,
        ],
    );
}

/// Run the integrate script for `feature` from `main`, with `mt` on `PATH`.
fn integrate(main: &Path) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.just/scripts/integrate.sh");
    let mt_dir = Path::new(env!("CARGO_BIN_EXE_mt")).parent().unwrap();
    let path = format!(
        "{}:{}",
        mt_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut command = std::process::Command::new("bash");
    command
        .arg(script)
        .arg("feature")
        .current_dir(main)
        .env("PATH", path);
    for (key, value) in isolated_env(main) {
        command.env(key, value);
    }
    command.output().unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The refusal text of a run that got past the refs check: none of the
/// script's earlier refusals, and no word of this one.
fn past_the_refs_check(text: &str) {
    for earlier in [
        "no worktree",
        "uncommitted changes",
        "not a fast-forward",
        "IS the worktree",
    ] {
        assert!(!text.contains(earlier), "refused early: {text}");
    }
    assert!(!text.contains("still wip"), "{text}");
}

#[test]
fn a_branch_leaving_a_named_task_wip_is_refused() {
    let (main, _holder, wt) = repo_with_worktree();
    commit(&wt, "feat: parse\n\nRefs: parser");
    let out = integrate(main.path());
    let text = stderr(&out);
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains("still wip at its tip") && text.contains("  parser"),
        "the refusal names the task: {text}"
    );
}

#[test]
fn a_branch_that_closes_the_task_it_names_passes_the_refs_check() {
    let (main, _holder, wt) = repo_with_worktree();
    let task = wt.join("tasks/parser.typ");
    let source = std::fs::read_to_string(&task).unwrap();
    std::fs::write(&task, source.replace("wip(", "done(")).unwrap();
    commit(&wt, "feat: parse\n\nRefs: parser");
    let text = stderr(&integrate(main.path()));
    past_the_refs_check(&text);
}

#[test]
fn a_branch_naming_no_task_passes_the_refs_check() {
    let (main, _holder, wt) = repo_with_worktree();
    commit(&wt, "task: note the parser");
    let text = stderr(&integrate(main.path()));
    past_the_refs_check(&text);
}
