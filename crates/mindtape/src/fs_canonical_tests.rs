#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use super::{canonical, relative_to};
use crate::test_support::tempdir;

#[test]
fn canonical_resolves_a_dot_dot() {
    let dir = tempdir();
    std::fs::create_dir(dir.path().join("a")).unwrap();
    let resolved = canonical(&dir.path().join("a").join("..")).unwrap();
    assert_eq!(resolved, canonical(dir.path()).unwrap());
}

#[test]
fn canonical_fails_on_a_missing_path() {
    let dir = tempdir();
    assert!(canonical(&dir.path().join("missing")).is_err());
}

/// The point of the helper: no `\\?\` reaches the user.
#[cfg(windows)]
#[test]
fn canonical_drops_the_verbatim_prefix() {
    let dir = tempdir();
    let resolved = canonical(dir.path()).unwrap();
    assert!(
        !resolved.to_string_lossy().starts_with(r"\\?\"),
        "{resolved:?}"
    );
}

#[test]
fn relative_to_strips_a_root_spelled_the_same_way() {
    let rel = relative_to(Path::new("/a/b/c.toml"), Path::new("/a"));
    assert_eq!(rel, PathBuf::from("b").join("c.toml"));
}

#[test]
fn relative_to_leaves_a_path_outside_the_root_alone() {
    let dir = tempdir();
    let other = tempdir();
    let path = other.path().join("c.toml");
    assert_eq!(relative_to(&path, dir.path()), path);
}

/// A resolved path still sits under a root reached through a symlink.
#[cfg(unix)]
#[test]
fn relative_to_matches_a_resolved_path_under_a_symlinked_root() {
    let dir = tempdir();
    let real = dir.path().join("real");
    std::fs::create_dir(&real).unwrap();
    std::fs::write(real.join("team.toml"), "").unwrap();
    let link = dir.path().join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let resolved = canonical(&real.join("team.toml")).unwrap();
    assert_eq!(relative_to(&resolved, &link), PathBuf::from("team.toml"));
}
