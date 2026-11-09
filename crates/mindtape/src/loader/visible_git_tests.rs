#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::{ignored_result, is_worktree, parse_repo_ref};
use crate::query::GitFailure;

#[test]
fn is_worktree_reads_the_boolean_truthfully() {
    assert!(is_worktree(b"true\n"));
    assert!(!is_worktree(b"false\n"));
    assert!(!is_worktree(b""));
}

/// Parse the same Git output for `dir`, whose nearest existing ancestor is
/// `base`.
fn parsed(dir: &str, base: &str) -> super::RepoRef {
    parse_repo_ref(b"/repo\nsub/\n", Path::new(dir), Path::new(base)).unwrap()
}

#[test]
fn parse_repo_ref_reads_the_toplevel_and_prefix() {
    let repo = parsed("/repo/sub", "/repo/sub");
    assert_eq!(repo.top, Path::new("/repo"));
    assert_eq!(repo.prefix, Path::new("sub"));
}

#[test]
fn parse_repo_ref_adds_a_root_below_the_existing_ancestor() {
    assert_eq!(
        parsed("/repo/sub/deep", "/repo/sub").prefix,
        Path::new("sub/deep")
    );
}

#[test]
fn parse_repo_ref_refuses_output_without_a_prefix() {
    assert!(matches!(
        parse_repo_ref(b"/repo", Path::new("/repo"), Path::new("/repo")),
        Err(GitFailure::Other(_))
    ));
}

#[test]
fn ignored_result_maps_exit_codes_and_strips_gits_label() {
    assert_eq!(ignored_result(Some(0), b""), Ok(true));
    assert_eq!(ignored_result(Some(1), b""), Ok(false));
    assert!(matches!(
        ignored_result(Some(2), b"fatal: bad exclude file\n"),
        Err(GitFailure::Other(reason)) if reason == "bad exclude file"
    ));
    assert!(matches!(
        ignored_result(None, b""),
        Err(GitFailure::Other(reason)) if reason == "git check-ignore failed"
    ));
}
