//! A commit made with `git commit --amend` replaces `HEAD`, so the
//! `commit-msg` check judges it against `HEAD^`
//! (`truth/tasks/judge-an-amended-commit-against-its-parent-in-the-commit-msg-check.typ`).
//! These tests run real `git commit` with a real hook, because the check has
//! to find out from git's own process that the commit is an amend.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]
#![cfg(unix)]

use mindtape::test_support::write_file;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::support;

use support::{commit_repo, git, isolated_env};

/// This repository's `flip-with-work`, over the one-task fixture.
const FLIP_WITH_WORK: &str = "[commits]\ntrailer = \"Refs\"\n\n\
     [[commits.rules]]\n\
     name = \"flip-with-work\"\n\
     scope = \"task\"\n\
     when = 'into(\"wip\") || into(\"done\")'\n\
     assert = 'named && !tasks_only'\n\
     message = \"start or close a task in a commit that carries its work\"\n\
     severity = \"error\"\n";

fn parser(status: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Write the parser\",\n  status: {status},\n  \
         priority: 3,\n)\n"
    )
}

/// `git commit` with the hook installed, as `(succeeded, output)`.
fn commit(dir: &Path, args: &[&str]) -> (bool, String) {
    let mut command = Command::new("git");
    command
        .current_dir(dir)
        .args(["-c", "commit.gpgsign=false", "commit", "-q"])
        .args(args);
    for (key, value) in isolated_env(dir) {
        command.env(key, value);
    }
    support::text_of(&command.output().unwrap())
}

/// A repository whose `HEAD` is a work commit naming `parser`, parent of it a
/// commit that filed `parser` as proposed, with the `commit-msg` hook in.
fn work_commit_repo() -> tempfile::TempDir {
    let dir = commit_repo(FLIP_WITH_WORK);
    let path = dir.path();
    write_file(path, "tasks/parser.typ", parser("proposed(2026, 6, 1)"));
    git(path, &["add", "-A"]);
    let (ok, text) = commit(path, &["-m", "task: file the parser"]);
    assert!(ok, "{text}");
    std::fs::create_dir_all(path.join("src")).unwrap();
    write_file(path, "src/lib.rs", "work\n");
    git(path, &["add", "-A"]);
    let (ok, text) = commit(path, &["-m", "feat: parse\n\nRefs: parser"]);
    assert!(ok, "{text}");
    install_hook(path);
    dir
}

/// Install the `commit-msg` hook that runs `mt check --commit-msg`.
fn install_hook(path: &Path) {
    let hook = path.join(".git/hooks/commit-msg");
    std::fs::write(
        &hook,
        format!(
            "#!/bin/sh\nexec '{}' check --commit-msg \"$1\"\n",
            env!("CARGO_BIN_EXE_mt")
        ),
    )
    .unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Stage the close of `parser` in `path` and commit it with `args`.
fn close_parser(path: &Path, args: &[&str]) -> (bool, String) {
    write_file(path, "tasks/parser.typ", parser("done(2026, 6, 2)"));
    git(path, &["add", "tasks/parser.typ"]);
    commit(path, args)
}

#[test]
fn a_flip_amended_into_its_work_commit_passes() {
    let dir = work_commit_repo();
    let (ok, text) = close_parser(dir.path(), &["--amend", "--no-edit"]);
    assert!(ok, "the amended commit carries the work: {text}");
}

#[test]
fn a_flip_amended_with_a_new_message_passes() {
    let dir = work_commit_repo();
    let (ok, text) = close_parser(
        dir.path(),
        &["--amend", "-m", "feat: parse it\n\nRefs: parser"],
    );
    assert!(ok, "{text}");
}

#[test]
fn a_flip_in_a_new_commit_on_top_of_the_work_is_still_refused() {
    let dir = work_commit_repo();
    let (ok, text) = close_parser(dir.path(), &["-m", "task: close\n\nRefs: parser"]);
    assert!(!ok, "a flip alone is not carried by HEAD's work: {text}");
    assert!(text.contains("flip-with-work"), "{text}");
}

#[test]
fn an_amend_that_only_edits_the_flip_of_a_flip_only_commit_is_refused() {
    let dir = work_commit_repo();
    let path = dir.path();
    git(path, &["reset", "-q", "--hard", "HEAD^"]);
    let (ok, text) = close_parser(path, &["--amend", "-m", "task: close\n\nRefs: parser"]);
    assert!(
        !ok,
        "the fixture seeds parser as wip, so closing it against that parent carries no work: {text}"
    );
}

#[test]
fn amending_a_root_commit_is_judged_against_the_empty_tree() {
    let dir = commit_repo(FLIP_WITH_WORK);
    let path = dir.path();
    install_hook(path);
    std::fs::create_dir_all(path.join("src")).unwrap();
    write_file(path, "src/lib.rs", "work\n");
    git(path, &["add", "src/lib.rs"]);
    let (ok, text) = close_parser(path, &["--amend", "-m", "feat: parse\n\nRefs: parser"]);
    assert!(ok, "the root commit holds the work and the task: {text}");
}

#[test]
fn an_amend_of_a_flip_only_commit_still_refuses_that_flip() {
    let dir = work_commit_repo();
    let path = dir.path();
    git(path, &["reset", "-q", "--hard", "HEAD^"]);
    write_file(path, "tasks/parser.typ", parser("wip(2026, 6, 2)"));
    git(path, &["add", "tasks/parser.typ"]);
    let (ok, text) = commit(path, &["--no-verify", "-m", "task: start\n\nRefs: parser"]);
    assert!(ok, "{text}");
    let (ok, text) = commit(path, &["--amend", "-m", "task: start it\n\nRefs: parser"]);
    assert!(
        !ok,
        "HEAD's own flip, with no work, is what the amend holds: {text}"
    );
}
