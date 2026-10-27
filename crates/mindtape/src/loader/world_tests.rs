#![allow(clippy::unwrap_used)]

use typst::World;

use super::disk_world;

/// A file in a folder gets a `/`-separated main id. On Windows the folder's
/// separator is `\`, which Typst refuses in a virtual path.
#[test]
fn a_file_in_a_folder_gets_a_slash_separated_main_id() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tasks").join("t.typ");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "").unwrap();

    let world = disk_world(&file, dir.path()).unwrap();

    assert_eq!(world.main().vpath().get_with_slash(), "/tasks/t.typ");
}

#[test]
fn a_file_outside_the_root_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let file = other.path().join("t.typ");
    std::fs::write(&file, "").unwrap();

    let err = disk_world(&file, root.path()).err().unwrap();

    assert!(
        err.to_string().contains("is not under project root"),
        "{err}"
    );
}
