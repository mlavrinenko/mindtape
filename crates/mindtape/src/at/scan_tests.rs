//! Unit tests for [`DiskFs`] and the scan helpers over a real temporary tree.

#![allow(clippy::unwrap_used)]

use super::{AtFs, DiskFs, expand_glob, is_fork};

#[test]
fn disk_fs_reads_a_real_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    for dir in [
        "b/.mindtape",
        "a/.mindtape",
        "fork/.mindtape",
        "main/.git/worktrees/fork",
    ] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("file"), "text").unwrap();
    std::fs::write(
        root.join("fork/.git"),
        "gitdir: ../main/.git/worktrees/fork\n",
    )
    .unwrap();
    std::fs::write(root.join("main/.git/worktrees/fork/commondir"), "../..\n").unwrap();
    let fs = DiskFs;
    assert_eq!(fs.subdirs(root), vec!["a", "b", "fork", "main"]);
    assert!(fs.subdirs(&root.join("missing")).is_empty());
    assert!(fs.is_dir(&root.join("a")) && !fs.is_dir(&root.join("file")));
    assert!(fs.is_file(&root.join("file")) && !fs.is_file(&root.join("a")));
    assert_eq!(fs.read_file(&root.join("file")).as_deref(), Some("text"));
    assert_eq!(fs.read_file(&root.join("missing")), None);

    let found = expand_glob(&root.join("*"), &fs).unwrap();
    assert_eq!(
        found,
        vec![root.join("a"), root.join("b"), root.join("fork")]
    );
    assert!(is_fork(&root.join("fork"), &fs));
    assert!(!is_fork(&root.join("a"), &fs));
}

#[cfg(unix)]
#[test]
fn disk_fs_follows_a_symlinked_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("real/.mindtape")).unwrap();
    std::fs::create_dir(root.join("set")).unwrap();
    std::os::unix::fs::symlink(root.join("real"), root.join("set/link")).unwrap();
    assert_eq!(DiskFs.subdirs(&root.join("set")), vec!["link"]);
}
