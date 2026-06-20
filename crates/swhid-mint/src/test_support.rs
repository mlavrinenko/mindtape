//! Shared git fixtures for the workspace's tests.
//!
//! Behind the `test-support` feature so sibling crates (`typst-cleanup`) and the
//! CLI can build temp git repos with the same helpers; within this crate they
//! are available to `#[cfg(test)]` code directly.

// Test fixtures: panic by design (a failed git command aborts the test).
#![allow(clippy::missing_panics_doc)]

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Run `git -C root <args>`, asserting the command succeeds. Identity is set via
/// environment so the call works regardless of the host's git config.
pub fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .status()
        .expect("run git");
    assert!(status.success(), "git {args:?}");
}

/// An empty initialised repo on branch `main`, with commit signing disabled.
#[must_use]
pub fn repo() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["config", "commit.gpgsign", "false"]);
    dir
}

/// A repo on branch `main` with `files` written (creating parent dirs) and
/// committed in one commit. Returns the temp-dir guard and its canonical root.
#[must_use]
pub fn repo_with(files: &[(&str, &str)]) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().canonicalize().expect("canonicalize");
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    for (rel, contents) in files {
        let abs = root.join(rel);
        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(&abs, contents).expect("write file");
    }
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "seed"]);
    (dir, root)
}
