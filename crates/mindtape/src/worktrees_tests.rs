//! Unit tests for the pure worktree registry: parsing, classification, token
//! validation, and ordered resolution. Filesystem is injected via `is_project`
//! closures and an explicit `home`, so these touch no real paths.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use super::{WorktreeEntry, WorktreeError, WorktreeRegistry, expand_tilde};

/// A base candidate at `<base>/<token>` is a project only when it is one of
/// these paths — lets a test assert base-skip-and-try-next without a filesystem.
fn projects_at(paths: &[&str]) -> impl Fn(&Path) -> bool {
    let owned: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    move |p: &Path| owned.iter().any(|q| q == p)
}

fn never_project(_: &Path) -> bool {
    false
}

#[test]
fn parses_alias_and_base_in_order() {
    let reg = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        name = "sn"
        path = "~/projects/home/speconaut"

        [[worktrees]]
        base = "~/projects/home"
        "#,
    )
    .unwrap();
    assert_eq!(
        reg.entries(),
        &[
            WorktreeEntry::Alias {
                name: "sn".to_owned(),
                path: PathBuf::from("~/projects/home/speconaut"),
            },
            WorktreeEntry::Base {
                base: PathBuf::from("~/projects/home"),
            },
        ]
    );
}

#[test]
fn empty_document_is_empty_registry() {
    let reg = WorktreeRegistry::parse("").unwrap();
    assert!(reg.is_empty());
    assert!(reg.alias_names().is_empty());
}

#[test]
fn unknown_top_level_key_errors() {
    let err = WorktreeRegistry::parse("bogus = 1").unwrap_err();
    assert!(matches!(err, WorktreeError::Parse(_)), "got {err:?}");
}

#[test]
fn mixed_entry_errors() {
    let err = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        name = "sn"
        path = "/p"
        base = "/b"
        "#,
    )
    .unwrap_err();
    match err {
        WorktreeError::Entry { index, message } => {
            assert_eq!(index, 1);
            assert!(message.contains("mixes `base`"), "{message}");
        }
        other => panic!("expected Entry, got {other:?}"),
    }
}

#[test]
fn alias_without_path_errors() {
    let err = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        name = "sn"
        "#,
    )
    .unwrap_err();
    assert!(
        matches!(err, WorktreeError::Entry { message, .. } if message.contains("needs a `path`")),
        "unexpected error"
    );
}

#[test]
fn base_without_leaf_errors() {
    let err = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        path = "/p"
        "#,
    )
    .unwrap_err();
    assert!(
        matches!(err, WorktreeError::Entry { message, .. } if message.contains("needs a `name`")),
        "unexpected error"
    );
}

#[test]
fn empty_entry_errors() {
    let err = WorktreeRegistry::parse("[[worktrees]]").unwrap_err();
    assert!(
        matches!(err, WorktreeError::Entry { message, .. } if message.contains("empty entry")),
        "unexpected error"
    );
}

#[test]
fn duplicate_alias_errors() {
    let err = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        name = "sn"
        path = "/a"

        [[worktrees]]
        name = "sn"
        path = "/b"
        "#,
    )
    .unwrap_err();
    assert!(
        matches!(err, WorktreeError::DuplicateAlias { name } if name == "sn"),
        "unexpected error"
    );
}

#[test]
fn resolve_alias_returns_expanded_path_regardless_of_project() {
    let reg = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        name = "sn"
        path = "~/wt/speconaut"
        "#,
    )
    .unwrap();
    // never_project: an alias matches on name alone, so it still resolves.
    let got = reg
        .resolve("sn", Some(Path::new("/home/u")), never_project)
        .unwrap();
    assert_eq!(got, Some(PathBuf::from("/home/u/wt/speconaut")));
}

#[test]
fn resolve_alias_wins_over_later_base() {
    let reg = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        name = "sn"
        path = "/explicit/speconaut"

        [[worktrees]]
        base = "/home/u/projects"
        "#,
    )
    .unwrap();
    // The base would also match `sn` (as a project), but the alias is first.
    let got = reg
        .resolve("sn", None, projects_at(&["/home/u/projects/sn"]))
        .unwrap();
    assert_eq!(got, Some(PathBuf::from("/explicit/speconaut")));
}

#[test]
fn resolve_base_joins_token_when_project() {
    let reg = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        base = "~/projects/home"
        "#,
    )
    .unwrap();
    let got = reg
        .resolve(
            "abc",
            Some(Path::new("/home/u")),
            projects_at(&["/home/u/projects/home/abc"]),
        )
        .unwrap();
    assert_eq!(got, Some(PathBuf::from("/home/u/projects/home/abc")));
}

#[test]
fn resolve_base_skips_non_project_and_tries_next() {
    let reg = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        base = "/first"

        [[worktrees]]
        base = "/second"
        "#,
    )
    .unwrap();
    // `/first/abc` is not a project; `/second/abc` is → the second base wins.
    let got = reg
        .resolve("abc", None, projects_at(&["/second/abc"]))
        .unwrap();
    assert_eq!(got, Some(PathBuf::from("/second/abc")));
}

#[test]
fn resolve_unknown_token_returns_none() {
    let reg = WorktreeRegistry::parse(
        r#"
        [[worktrees]]
        name = "sn"
        path = "/p"
        "#,
    )
    .unwrap();
    let got = reg.resolve("nope", None, never_project).unwrap();
    assert_eq!(got, None);
}

#[test]
fn resolve_rejects_unsafe_tokens() {
    let reg = WorktreeRegistry::default();
    for bad in ["", ".", "..", "a/b", "a\\b"] {
        let err = reg.resolve(bad, None, never_project).unwrap_err();
        assert!(
            matches!(err, WorktreeError::InvalidToken { .. }),
            "token {bad:?} should be rejected, got {err:?}"
        );
    }
}

#[test]
fn expand_tilde_covers_bare_and_prefixed_and_passthrough() {
    let home = Path::new("/home/u");
    assert_eq!(expand_tilde(Path::new("~"), Some(home)), home);
    assert_eq!(
        expand_tilde(Path::new("~/a/b"), Some(home)),
        PathBuf::from("/home/u/a/b")
    );
    // No home available: pass through unchanged.
    assert_eq!(expand_tilde(Path::new("~/a"), None), PathBuf::from("~/a"));
    // Absolute path: untouched.
    assert_eq!(
        expand_tilde(Path::new("/abs/p"), Some(home)),
        PathBuf::from("/abs/p")
    );
}
