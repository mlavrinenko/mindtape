#![allow(clippy::unwrap_used)]

use super::read_blob;

/// [`read_blob`] of `id` from `output`: the bytes written, or the error.
fn read(output: &[u8], id: &str) -> Result<Vec<u8>, String> {
    let mut written = Vec::new();
    read_blob(&mut &output[..], id, &mut written)
        .map(|()| written)
        .map_err(|err| err.to_string())
}

#[test]
fn read_blob_copies_exactly_its_size() {
    assert_eq!(
        read(b"ab blob 3\nx\ny\nab blob 0\n\n", "ab").unwrap(),
        b"x\ny"
    );
}

#[test]
fn read_blob_refuses_what_is_not_the_blob_it_asked_for() {
    for (output, why) in [
        (&b"ab tree 3\nxyz\n"[..], "a tree"),
        (b"cd blob 3\nxyz\n", "another id"),
        (b"ab missing\n", "a missing object"),
        (b"ab blob 3\nxyz", "no trailing newline"),
        (b"ab blob 3\nxyzw", "a wrong trailing byte"),
        (b"ab blob 9\nxyz\n", "a short body"),
        (b"ab blob x\nxyz\n", "a size that is not a number"),
        (b"", "no output"),
    ] {
        let err = read(output, "ab").unwrap_err();
        assert!(err.starts_with("git cat-file failed: "), "{why}: {err}");
    }
}
