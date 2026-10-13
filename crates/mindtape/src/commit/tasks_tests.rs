#![allow(clippy::unwrap_used)]

use super::{StatePlan, parse_batch, read_blobs};
use crate::test_support::tempdir;
use crate::test_support::{git_as_tester as git, git_init};

#[test]
fn a_batch_splits_into_objects_and_misses_in_order() {
    let out = b"abc blob 3\nfoo\nHEAD:gone.typ missing\ndef blob 0\n\n";
    assert_eq!(
        parse_batch(out, 3),
        [Some("foo".to_owned()), None, Some(String::new())]
    );
}

#[test]
fn a_short_batch_pads_with_misses() {
    assert_eq!(parse_batch(b"", 2), [None, None]);
    assert_eq!(parse_batch(b"HEAD:a b.typ missing\n", 1), [None]);
}

/// A repository with `kept.typ` and `moved.typ` committed, then `kept.typ`
/// edited and staged, `moved.typ` renamed and staged, and `loose.typ` left
/// untracked.
fn repo() -> tempfile::TempDir {
    let dir = tempdir();
    let root = dir.path();
    git_init(root);
    std::fs::write(root.join("kept.typ"), "old\n").unwrap();
    std::fs::write(root.join("moved.typ"), "same\n").unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "seed"]);
    std::fs::write(root.join("kept.typ"), "new\n").unwrap();
    git(root, &["mv", "moved.typ", "renamed.typ"]);
    git(root, &["add", "kept.typ"]);
    std::fs::write(root.join("loose.typ"), "x\n").unwrap();
    dir
}

#[test]
fn the_index_plan_names_every_side_of_each_change() {
    let dir = repo();
    let plan = StatePlan::index(dir.path()).unwrap();
    let mut head = plan.head.differs.clone();
    head.sort();
    assert_eq!(head, ["kept.typ", "loose.typ", "moved.typ", "renamed.typ"]);
    assert_eq!(plan.staged.differs, ["loose.typ"]);
    assert_eq!(
        (plan.head.rev.as_str(), plan.staged.rev.as_str()),
        ("HEAD", "")
    );
    assert!(plan.worktree);
    assert_eq!(plan.root_prefix, "");
}

#[test]
fn blobs_read_each_revision_and_miss_what_it_lacks() {
    let dir = repo();
    let specs = [
        "HEAD:kept.typ",
        ":kept.typ",
        "HEAD:renamed.typ",
        ":moved.typ",
    ]
    .map(ToOwned::to_owned);
    assert_eq!(
        read_blobs(dir.path(), &specs).unwrap(),
        [
            Some("old\n".to_owned()),
            Some("new\n".to_owned()),
            None,
            None
        ]
    );
}
