use super::{encode, tail_of};

/// Windows paths, parsed the way Windows reads them on every OS, and the URI
/// each one links to.
const WINDOWS: &[(&str, &str)] = &[
    (r"C:\proj\tasks\a.typ", "file:///C:/proj/tasks/a.typ"),
    // A root joined with a slashed path reads as one path.
    (
        r"C:\proj\truth/tasks/a.typ",
        "file:///C:/proj/truth/tasks/a.typ",
    ),
    // The verbatim prefix goes.
    (r"\\?\C:\proj\a.typ", "file:///C:/proj/a.typ"),
    // A UNC path names its server as the host.
    (r"\\server\share\a.typ", "file://server/share/a.typ"),
    (r"\\?\UNC\server\share\a.typ", "file://server/share/a.typ"),
    (r"C:\my proj\ä.typ", "file:///C:/my%20proj/%C3%A4.typ"),
];

#[test]
fn a_windows_path_links_to_an_rfc_8089_uri() {
    for (path, uri) in WINDOWS {
        assert_eq!(format!("file://{}", tail_of(path, true)), *uri, "{path}");
    }
}

#[test]
fn a_unix_path_keeps_its_root_and_encodes_a_backslash() {
    assert_eq!(
        format!("file://{}", tail_of(r"/home/u/my proj/a\b.typ", false)),
        "file:///home/u/my%20proj/a%5Cb.typ"
    );
}

#[test]
fn encode_keeps_unreserved_characters_and_slashes() {
    assert_eq!(encode("a-b_c.d~e/f"), "a-b_c.d~e/f");
    assert_eq!(encode("a#b?c"), "a%23b%3Fc");
}
