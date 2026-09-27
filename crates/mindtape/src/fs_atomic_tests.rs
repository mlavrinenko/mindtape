#![allow(clippy::unwrap_used)]

use super::write_atomic;

#[test]
fn writes_new_bytes_and_overwrites_existing_ones() {
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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

    let dir = tempfile::tempdir().unwrap();
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
