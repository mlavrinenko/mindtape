#![allow(clippy::unwrap_used)]

use super::hash;
use crate::test_support::{tempdir, write_file};

#[test]
fn hash_frames_each_file_in_bytewise_path_order() {
    let dir = tempdir();
    write_file(dir.path(), "a/b", "1");
    write_file(dir.path(), "a.b", "22");
    write_file(dir.path(), "B", "");
    let mut framed = Vec::new();
    for (path, bytes) in [("B", ""), ("a.b", "22"), ("a/b", "1")] {
        framed.extend((path.len() as u64).to_le_bytes());
        framed.extend(path.as_bytes());
        framed.extend((bytes.len() as u64).to_le_bytes());
        framed.extend(bytes.as_bytes());
    }
    use sha2::Digest;
    let expected = crate::loader::cache::hex(&sha2::Sha256::digest(&framed));
    assert_eq!(hash(dir.path()).unwrap(), expected);
}

#[test]
fn hash_of_an_empty_directory_is_the_empty_digest() {
    let dir = tempdir();
    assert_eq!(
        hash(dir.path()).unwrap(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn hash_counts_an_empty_directory_as_nothing() {
    let dir = tempdir();
    write_file(dir.path(), "a", "1");
    let before = hash(dir.path()).unwrap();
    std::fs::create_dir(dir.path().join("empty")).unwrap();
    assert_eq!(hash(dir.path()).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn hash_refuses_a_symlink() {
    let dir = tempdir();
    write_file(dir.path(), "a", "1");
    std::os::unix::fs::symlink("a", dir.path().join("link")).unwrap();
    let err = hash(dir.path()).unwrap_err();
    assert_eq!(err.to_string(), "pack source holds a symlink: 'link'");
}
