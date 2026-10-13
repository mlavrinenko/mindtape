#![allow(clippy::unwrap_used)]

use crate::test_support::tempdir;
use crate::test_support::write_file;
use std::fs;

use super::current_branch;

#[test]
fn branch_head_resolves_name() {
    let dir = tempdir();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(
        dir.path().join(".git").join("HEAD"),
        "ref: refs/heads/main\n",
    )
    .unwrap();
    assert_eq!(current_branch(dir.path()), Some("main".to_owned()));
}

#[test]
fn branch_head_resolves_nested_name() {
    let dir = tempdir();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(
        dir.path().join(".git").join("HEAD"),
        "ref: refs/heads/release/1.0\n",
    )
    .unwrap();
    assert_eq!(current_branch(dir.path()), Some("release/1.0".to_owned()));
}

#[test]
fn detached_head_is_none() {
    let dir = tempdir();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(
        dir.path().join(".git").join("HEAD"),
        "8f3a1c2b9e0d4f5a6b7c8d9e0f1a2b3c4d5e6f70\n",
    )
    .unwrap();
    assert_eq!(current_branch(dir.path()), None);
}

#[test]
fn worktree_gitdir_file_resolves() {
    let dir = tempdir();
    let real_git = dir.path().join("real-git-dir");
    fs::create_dir_all(&real_git).unwrap();
    write_file(&real_git, "HEAD", "ref: refs/heads/feature/x\n");
    write_file(
        dir.path(),
        ".git",
        format!("gitdir: {}\n", real_git.display()),
    );
    assert_eq!(current_branch(dir.path()), Some("feature/x".to_owned()));
}

#[test]
fn worktree_gitdir_file_relative_path_resolves() {
    let dir = tempdir();
    fs::create_dir_all(dir.path().join("real-git-dir")).unwrap();
    fs::write(
        dir.path().join("real-git-dir").join("HEAD"),
        "ref: refs/heads/relative\n",
    )
    .unwrap();
    write_file(dir.path(), ".git", "gitdir: real-git-dir\n");
    assert_eq!(current_branch(dir.path()), Some("relative".to_owned()));
}

#[test]
fn no_git_directory_is_none() {
    let dir = tempdir();
    assert_eq!(current_branch(dir.path()), None);
}

#[test]
fn missing_head_file_is_none() {
    let dir = tempdir();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    assert_eq!(current_branch(dir.path()), None);
}

#[test]
fn malformed_head_is_none() {
    let dir = tempdir();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".git").join("HEAD"), "not a valid head\n").unwrap();
    assert_eq!(current_branch(dir.path()), None);
}

#[test]
fn ref_to_non_branch_namespace_is_none() {
    let dir = tempdir();
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(
        dir.path().join(".git").join("HEAD"),
        "ref: refs/tags/v1.0\n",
    )
    .unwrap();
    assert_eq!(current_branch(dir.path()), None);
}
