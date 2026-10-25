//! The shipped `mindtape:tasks` pack's `flip-with-work` lets a milestone close
//! in a task-only commit (`truth/tasks/exempt-a-milestones-close-from-the-
//! flip-with-work-rule.typ`): it ends through its leaves and carries no work of
//! its own, so its flip is the whole commit. A leaf's bare close stays refused.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use mindtape::test_support::write_file;

use crate::support::{check_commit_msg, git, git_commit_all, project_with_tasks};

/// Two `wip` tasks, `ship` a milestone and `leaf` a feat, under the shipped pack.
fn pack_repo() -> tempfile::TempDir {
    let dir = project_with_tasks(&[("ship.typ", "Ship it"), ("leaf.typ", "Build the leaf")]);
    write_file(
        dir.path(),
        ".mindtape/config.toml",
        "extends = [\"mindtape:tasks\"]\n[tasks]\nroots = [\"tasks\"]\n",
    );
    for (file, kind) in [("ship.typ", "milestone"), ("leaf.typ", "feat")] {
        let path = dir.path().join("tasks").join(file);
        let text = std::fs::read_to_string(&path).unwrap();
        let text = text.replace("status:", &format!("kind: \"{kind}\",\n  status:"));
        std::fs::write(&path, text).unwrap();
    }
    git_commit_all(dir.path());
    dir
}

/// Stage `mt done <stem>`'s edit.
fn stage_close(dir: &Path, stem: &str) {
    let rel = format!("tasks/{stem}.typ");
    let text = std::fs::read_to_string(dir.join(&rel)).unwrap();
    std::fs::write(
        dir.join(&rel),
        text.replace("wip(2026, 6, 1)", "done(2026, 6, 2)"),
    )
    .unwrap();
    git(dir, &["add", &rel]);
}

/// Stage `mt done <stem>` alone and check the commit message that names it.
fn close_alone(stem: &str) -> (bool, String) {
    let dir = pack_repo();
    stage_close(dir.path(), stem);
    check_commit_msg(dir.path(), &format!("task: close {stem}\n\nRefs: {stem}\n"))
}

#[test]
fn a_milestone_closes_in_a_task_only_commit() {
    let (ok, text) = close_alone("ship");
    assert!(ok, "a milestone carries no work of its own: {text}");
}

#[test]
fn a_leaf_closing_alone_is_still_refused() {
    let (ok, text) = close_alone("leaf");
    assert!(!ok && text.contains("flip-with-work: leaf"), "{text}");
}
