#![allow(clippy::unwrap_used)]

use super::write_atomic;
use crate::test_support::tempdir;
use crate::test_support::write_file;

#[test]
fn writes_new_bytes_and_overwrites_existing_ones() {
    let dir = tempdir();
    let path = dir.path().join("task.typ");

    write_atomic(&path, b"first").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"first");

    write_atomic(&path, b"second, longer than first").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"second, longer than first");
}

/// A killed `mt flip` must leave the old file whole — no `.tmp` sibling ever
/// survives a successful write for the next `mt ls` to trip over.
#[test]
fn no_temp_file_survives_a_successful_write() {
    let dir = tempdir();
    let path = dir.path().join("task.typ");

    write_atomic(&path, b"content").unwrap();

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("task.typ")]);
}

/// The other half of the same guarantee: a write that never reaches the
/// rename (temp file creation itself fails) must leave the target's existing
/// bytes untouched, never a truncated or empty file.
#[test]
#[cfg(unix)]
fn a_failed_write_leaves_the_target_unchanged() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir();
    let path = dir.path().join("task.typ");
    std::fs::write(&path, b"original").unwrap();

    // No write permission on the directory: the sibling temp file can never
    // be created, so `write_atomic` must fail before touching `path`.
    let original_mode = std::fs::metadata(dir.path()).unwrap().permissions().mode();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = write_atomic(&path, b"replacement");
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(original_mode)).unwrap();

    assert!(result.is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
}

/// The temp file is a sibling of the target, never a child of the process cwd:
/// the rename must stay within one directory, so a cwd that cannot accept a new
/// file must not stop a write to an absolute target. nextest runs one test per
/// process, so moving the cwd here is contained; it is restored either way.
#[test]
#[cfg(unix)]
fn the_temp_file_is_a_sibling_of_the_target_not_of_the_cwd() {
    use std::os::unix::fs::PermissionsExt;

    let target_dir = tempdir();
    let path = target_dir.path().join("task.typ");

    let cwd = tempdir();
    std::fs::set_permissions(cwd.path(), std::fs::Permissions::from_mode(0o555)).unwrap();

    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(cwd.path()).unwrap();
    let result = write_atomic(&path, b"content");
    std::env::set_current_dir(&original).unwrap();

    assert!(
        result.is_ok(),
        "a write must not depend on a writable cwd: {result:?}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"content");
}

/// The cleanup arm: a failure *after* the temp file exists must remove it. An
/// existing non-empty directory at the target makes the final rename fail
/// (a file cannot replace a directory) once the temp file is written and
/// fsynced, so this reaches `write_atomic`'s `remove_file(&tmp)`.
#[test]
#[cfg(unix)]
fn a_failed_rename_removes_the_temp_file_and_leaves_the_target_untouched() {
    let dir = tempdir();
    let path = dir.path().join("task.typ");
    std::fs::create_dir(&path).unwrap();
    write_file(&path, "inner.txt", b"keep me");

    let result = write_atomic(&path, b"replacement");

    assert!(result.is_err());
    assert!(path.is_dir());
    assert_eq!(std::fs::read(path.join("inner.txt")).unwrap(), b"keep me");
    let mut entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    entries.sort();
    assert_eq!(entries, vec![std::ffi::OsString::from("task.typ")]);
    assert_eq!(std::fs::read_dir(&path).unwrap().count(), 1);
}
