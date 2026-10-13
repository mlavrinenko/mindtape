//! The shipped `mindtape-tasks` pack's `flip-with-work` counts retiring
//! another task as work (`truth/tasks/count-retiring-other-tasks-as-work-in-
//! the-flip-with-work-rule.typ`): a close that deletes other task files
//! passes, while a bare flip, or a flip beside edits to other tasks, is
//! still refused.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use mindtape::test_support::write_file;

use crate::support::{self, check_commit_msg, git, git_commit_all, project_with_tasks};

const MESSAGE: &str = "task: close the parser\n\nRefs: parser\n";

/// Two `wip` tasks, `parser` and `other`, under the shipped pack.
fn pack_repo() -> tempfile::TempDir {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser"), ("other.typ", "Other")]);
    write_file(
        dir.path(),
        ".mindtape/config.toml",
        "extends = [\"mindtape-tasks\"]\n[tasks]\nroots = [\"tasks\"]\n",
    );
    git_commit_all(dir.path());
    dir
}

/// Stage `mt done parser`'s edit.
fn stage_close(dir: &Path) {
    let path = dir.join("tasks/parser.typ");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("wip(2026, 6, 1)", "done(2026, 6, 2)")).unwrap();
    git(dir, &["add", "tasks/parser.typ"]);
}

#[test]
fn a_close_that_retires_another_task_passes() {
    let dir = pack_repo();
    stage_close(dir.path());
    git(dir.path(), &["rm", "-q", "tasks/other.typ"]);
    let (ok, text) = check_commit_msg(dir.path(), MESSAGE);
    assert!(ok, "retiring another task is the close's work: {text}");
}

#[test]
fn a_close_alone_is_still_refused() {
    let dir = pack_repo();
    stage_close(dir.path());
    let text = support::commit_msg_refusal(dir.path(), MESSAGE);
    assert!(text.contains("flip-with-work: parser"), "{text}");
}

#[test]
fn a_close_beside_an_edit_to_another_task_is_still_refused() {
    let dir = pack_repo();
    stage_close(dir.path());
    let path = dir.path().join("tasks/other.typ");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("\"Other\"", "\"Other, reworded\"")).unwrap();
    git(dir.path(), &["add", "tasks/other.typ"]);
    let text = support::commit_msg_refusal(dir.path(), MESSAGE);
    assert!(text.contains("flip-with-work: parser"), "{text}");
}
