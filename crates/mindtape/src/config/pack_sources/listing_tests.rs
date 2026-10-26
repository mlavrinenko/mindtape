#![allow(clippy::unwrap_used)]

use std::path::Path;

use super::{parse_listing, safe_join};

/// [`parse_listing`]'s error for one `ls-tree -z` record, as text.
fn listing_error(records: &[&str]) -> String {
    let listing: Vec<u8> = records
        .iter()
        .flat_map(|r| [r.as_bytes(), b"\0"].concat())
        .collect();
    parse_listing(&listing).unwrap_err().to_string()
}

#[test]
fn a_listing_refuses_what_the_cache_cant_hold_the_same_everywhere() {
    let id = "1".repeat(40);
    let submodule = format!("160000 commit {id}\tvendor");
    assert_eq!(
        listing_error(&[&submodule]),
        "pack source holds a submodule: 'vendor'"
    );
    let upper = format!("100644 blob {id}\tA.toml");
    let lower = format!("100644 blob {id}\ta.toml");
    assert_eq!(
        listing_error(&[&upper, &lower]),
        "pack source holds a path that differs only in case: 'a.toml'"
    );
    let pathless = format!("100644 blob {id}");
    assert!(listing_error(&[&pathless]).starts_with("git ls-tree failed: "));
}

/// One blob record at `path`.
fn blob(path: &str) -> String {
    format!("100644 blob {}\t{path}", "1".repeat(40))
}

/// A listing of one blob at each of `paths`.
fn listing_of(paths: &[&str]) -> Vec<u8> {
    let id = "1".repeat(40);
    paths
        .iter()
        .flat_map(|path| format!("100644 blob {id}\t{path}\0").into_bytes())
        .collect()
}

/// The error a listing of one blob at each of `paths` gives.
fn paths_error(paths: &[&str]) -> String {
    let records: Vec<String> = paths.iter().map(|path| blob(path)).collect();
    listing_error(&records.iter().map(String::as_str).collect::<Vec<_>>())
}

#[test]
fn a_dot_dot_dot_or_empty_part_is_refused() {
    for path in ["../../evil", "a/../b", "./a", "a/./b", "a//b", "/a", "a/"] {
        assert_eq!(
            paths_error(&[path]),
            format!("pack source holds a path with an empty, '.' or '..' part: '{path}'"),
        );
    }
}

#[test]
fn a_name_windows_cant_hold_is_refused_everywhere() {
    let names = [
        "C:evil",
        "b\\c",
        "a\u{1}",
        "a\u{7f}",
        "a<b",
        "a>b",
        "a\"b",
        "a|b",
        "a?b",
        "a*b",
        "a.",
        "a ",
        "CON",
        "con.toml",
        "Prn",
        "aux.x.y",
        "NUL",
        "com0",
        "COM9.toml",
        "lpt1",
        "dir/LPT5",
    ];
    for path in names {
        assert_eq!(
            paths_error(&[path]),
            format!(
                "pack source holds a name Windows can't hold: {}",
                crate::config::quote(path)
            ),
            "{path:?}"
        );
    }
}

#[test]
fn a_name_like_a_device_but_not_one_is_kept() {
    let listing = listing_of(&["CONSOLE.toml", "com10", "lpt", "a.b/c", ".hidden"]);
    assert_eq!(parse_listing(&listing).unwrap().len(), 5);
}

#[test]
fn a_path_that_is_not_ascii_is_refused() {
    assert_eq!(
        paths_error(&["caf\u{e9}.toml"]),
        "pack source holds a path that is not ASCII: 'caf\u{e9}.toml'"
    );
}

#[test]
fn a_directory_spelled_two_ways_is_refused() {
    assert_eq!(
        paths_error(&["A/x", "a/y"]),
        "pack source holds a path that differs only in case: 'a/y'"
    );
    assert_eq!(
        paths_error(&["a", "A/b"]),
        "pack source holds a path that differs only in case: 'A/b'"
    );
    assert_eq!(
        parse_listing(&listing_of(&["A/x", "A/y"])).unwrap().len(),
        2
    );
}

#[test]
fn safe_join_keeps_a_path_inside_its_tree() {
    let tree = Path::new("cache").join("tree");
    assert_eq!(
        safe_join(&tree, "a/b.toml").unwrap(),
        tree.join("a").join("b.toml")
    );
    for path in [
        "../evil",
        "a/../../evil",
        "/etc/passwd",
        "a//b",
        "C:evil",
        ".",
    ] {
        assert!(safe_join(&tree, path).is_err(), "{path}");
    }
}
