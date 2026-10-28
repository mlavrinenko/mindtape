use std::path::Path;

use super::{display_path, slash_path, with_separator};

#[test]
fn a_backslash_separator_replaces_every_slash() {
    assert_eq!(
        with_separator("tasks/sub/a.typ", '\\', false),
        r"tasks\sub\a.typ"
    );
}

/// Windows reads both, so a path a config joined with `/` onto a native one
/// comes out with one separator, not `.mindtape/templates\bug.typ`.
#[test]
fn where_backslash_separates_a_mixed_path_comes_out_with_one_separator() {
    assert_eq!(
        with_separator(r".mindtape/templates\bug.typ", '\\', true),
        r".mindtape\templates\bug.typ"
    );
    assert_eq!(
        with_separator(r"C:\proj\tasks/a.typ", '/', true),
        "C:/proj/tasks/a.typ"
    );
}

/// Elsewhere a backslash is a character a file name may hold.
#[test]
fn where_backslash_is_a_name_character_it_stays() {
    assert_eq!(
        with_separator(r"tasks/a\b.typ", '/', false),
        r"tasks/a\b.typ"
    );
}

#[test]
fn a_path_without_separators_is_unchanged() {
    assert_eq!(with_separator("a.typ", '\\', true), "a.typ");
}

/// Unset, a path shows in the platform's own separator.
#[test]
fn display_path_shows_the_native_separator_by_default() {
    let native = std::path::MAIN_SEPARATOR.to_string();
    assert_eq!(
        display_path(Path::new("tasks/sub/a.typ")),
        ["tasks", "sub", "a.typ"].join(&native)
    );
}

#[test]
fn slash_path_joins_native_components_with_slashes() {
    let path = Path::new("tasks").join("sub").join("a.typ");
    assert_eq!(slash_path(&path), "tasks/sub/a.typ");
}

#[test]
fn slash_path_keeps_an_absolute_path_rooted_once() {
    assert_eq!(slash_path(Path::new("/home/u/a.typ")), "/home/u/a.typ");
}

/// A drive and a UNC prefix parse only on Windows.
#[cfg(windows)]
#[test]
fn slash_path_writes_a_windows_prefix_with_slashes() {
    assert_eq!(slash_path(Path::new(r"C:\proj\a.typ")), "C:/proj/a.typ");
    assert_eq!(
        slash_path(Path::new(r"\\server\share\a.typ")),
        "//server/share/a.typ"
    );
}
