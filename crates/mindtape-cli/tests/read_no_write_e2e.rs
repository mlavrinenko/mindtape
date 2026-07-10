//! Regression test for the read/write separation (CQRS) principle: read
//! commands must never mutate the working tree.
//!
//! See `truth/tasks/command-semantics-cqrs.typ` and the "Command Semantics
//! (read/write separation)" section of `CONTRIBUTING.md`. Each test builds a
//! link-free fixture task tree, hashes it, runs a single read command, hashes
//! it again, and asserts the two digests match — byte-identical, not just
//! "still parses".
#![allow(clippy::unwrap_used)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

mod support;

/// Scaffold a temp project (`.mindtape/config.toml` + the in-tree lib
/// override) and file two link-free tasks into it via `mt add`.
///
/// No task holds a `depends-on`/`relates-to`/`supersedes`/`duplicates`/`links`
/// field: a parallel task is reworking the link DSL on the same base commit,
/// so these fixtures stay decoupled from it.
fn build_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    support::install_lib(dir.path());

    for (name, title) in [("alpha", "Alpha task"), ("beta", "Beta task")] {
        assert_cmd::Command::cargo_bin("mt")
            .unwrap()
            .current_dir(dir.path())
            .args(["add", "--name", name, "--set", &format!("title={title}")])
            .assert()
            .success();
    }

    dir
}

/// Hash every file under `dir` by relative path and byte content into a
/// single order-independent digest, so any mutation — content change, added
/// or removed file — changes the result.
fn hash_tree(dir: &Path) -> u64 {
    let mut files = Vec::new();
    collect_files(dir, dir, &mut files);
    files.sort();

    let mut hasher = DefaultHasher::new();
    for (rel, bytes) in &files {
        rel.hash(&mut hasher);
        bytes.hash(&mut hasher);
    }
    hasher.finish()
}

/// Recursively collect `(path relative to root, file bytes)` pairs under `dir`.
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out);
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            out.push((rel, std::fs::read(&path).unwrap()));
        }
    }
}

/// Run `mt <args>` in `dir` and assert the tree hash is byte-identical before
/// and after — the read command must succeed, but leave nothing changed.
fn assert_read_untouched(dir: &Path, args: &[&str]) {
    let before = hash_tree(dir);

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir)
        .args(args)
        .assert()
        .success();

    let after = hash_tree(dir);
    assert_eq!(
        before,
        after,
        "`mt {}` mutated the fixture tree",
        args.join(" ")
    );
}

#[test]
fn ls_default_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["ls"]);
}

#[test]
fn ls_json_format_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["ls", "--format", "json"]);
}

#[test]
fn check_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["check"]);
}

#[test]
fn config_show_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["config", "show"]);
}

#[test]
fn config_show_json_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["config", "show", "--json"]);
}

#[test]
fn config_show_origin_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["config", "show", "--origin"]);
}

#[test]
fn eval_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["eval", "tasks/alpha.typ"]);
}

#[test]
fn views_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["views"]);
}

#[test]
fn worktrees_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["worktrees"]);
}
