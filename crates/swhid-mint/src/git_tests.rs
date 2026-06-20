#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{Error, mint_content, mint_rev_path, resolves};
use crate::Kind;
use crate::test_support::{git, repo};

#[test]
fn mints_and_resolves_rev_path() {
    let dir = repo();
    let root = dir.path();
    std::fs::write(root.join("a.typ"), "hello").expect("write");
    git(root, &["add", "a.typ"]);
    git(root, &["commit", "-q", "-m", "add a"]);

    let id = mint_rev_path(root, "a.typ").expect("mint");
    assert_eq!(id.kind, Kind::Revision);
    assert_eq!(id.hash.len(), 40);
    assert_eq!(id.path.as_deref(), Some("a.typ"));
    assert!(resolves(root, &id).expect("resolves"));
}

#[test]
fn refuses_untracked_path() {
    let dir = repo();
    let root = dir.path();
    // Repo needs history; an untracked file then has no touching commit.
    std::fs::write(root.join("seed.typ"), "seed").expect("write");
    git(root, &["add", "seed.typ"]);
    git(root, &["commit", "-q", "-m", "seed"]);
    std::fs::write(root.join("u.typ"), "x").expect("write");
    assert!(matches!(
        mint_rev_path(root, "u.typ"),
        Err(Error::NotTracked(_))
    ));
}

#[test]
fn refuses_dirty_working_tree() {
    let dir = repo();
    let root = dir.path();
    std::fs::write(root.join("a.typ"), "v1").expect("write");
    git(root, &["add", "a.typ"]);
    git(root, &["commit", "-q", "-m", "v1"]);
    std::fs::write(root.join("a.typ"), "v2-uncommitted").expect("write");
    assert!(matches!(mint_rev_path(root, "a.typ"), Err(Error::Dirty(_))));
}

#[test]
fn resolve_is_false_for_absent_path_at_commit() {
    let dir = repo();
    let root = dir.path();
    std::fs::write(root.join("a.typ"), "x").expect("write");
    git(root, &["add", "a.typ"]);
    git(root, &["commit", "-q", "-m", "a"]);
    let mut id = mint_rev_path(root, "a.typ").expect("mint");
    id.path = Some("missing.typ".to_owned());
    assert!(!resolves(root, &id).expect("resolves"));
}

#[test]
fn mints_content_id_without_commit() {
    let dir = repo();
    let root = dir.path();
    std::fs::write(root.join("a.typ"), "hello").expect("write");
    let id = mint_content(root, "a.typ").expect("mint cnt");
    assert_eq!(id.kind, Kind::Content);
    assert_eq!(id.hash.len(), 40);
}
