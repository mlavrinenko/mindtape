//! The checkout's root `.gitignore` ignores a task cache wherever the project
//! sits, before `mt` has written the cache's own `.gitignore`.
//!
//! A pattern with a slash in the middle is anchored to the repo root, so
//! `.mindtape/cache/` alone covers no project under `examples/`. mmz hashes
//! what git does not ignore, so a gate that reaches `examples/` hashed a cache
//! file that another gate's `mt` then renamed away.
//!
//! Unix only: the checkout and the `git` door come from `support::shell`.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::tempdir;

/// What `git check-ignore -q path` exits with in a scratch repo that holds only
/// the checkout's `.gitignore`: 0 for an ignored path, 1 for one that is not.
fn check_ignore(path: &str) -> Option<i32> {
    let dir = tempdir();
    let root = support::checkout();
    std::fs::copy(root.join(".gitignore"), dir.path().join(".gitignore")).unwrap();
    support::git(dir.path(), &["init", "-q"]);
    support::shell("git", dir.path())
        .args(["check-ignore", "-q", path])
        .status()
        .unwrap()
        .code()
}

#[test]
fn a_cache_is_ignored_at_any_depth() {
    for path in [
        ".mindtape/cache/key/entry.json",
        "examples/minimal/.mindtape/cache/key/entry.json",
        "examples/across-projects/projects/blog/.mindtape/cache/key/.entry.json.tmp",
    ] {
        assert_eq!(check_ignore(path), Some(0), "{path} is not ignored");
    }
}

#[test]
fn the_rest_of_a_project_stays_tracked() {
    for path in [
        ".mindtape/config.toml",
        "examples/minimal/.mindtape/config.toml",
    ] {
        assert_eq!(check_ignore(path), Some(1), "{path} is ignored");
    }
}
