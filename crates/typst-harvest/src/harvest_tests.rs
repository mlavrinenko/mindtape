#![allow(clippy::unwrap_used)]

use super::super::value::HVal;
use super::super::{HarvestWorld, harvest};

fn project(file_body: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".mindtape")).unwrap();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, file_body).unwrap();
    (dir, file)
}

#[test]
fn harvests_a_marker() {
    let (_dir, file) =
        project("#metadata((marker: \"mindtape.task\", fields: (title: \"T\"), extras: (:)))\n");
    let world = HarvestWorld::new(&file).unwrap();
    let result = harvest(&world).unwrap();

    let markers: Vec<_> = result.with_marker("mindtape.task").collect();
    assert_eq!(markers.len(), 1);
    let fields = markers.first().unwrap().value.get("fields").unwrap();
    assert_eq!(fields.get("title").and_then(HVal::as_str), Some("T"));
}

#[test]
fn ignores_files_without_the_marker() {
    let (_dir, file) = project("= Just a heading\n\nSome prose.\n");
    let world = HarvestWorld::new(&file).unwrap();
    let result = harvest(&world).unwrap();
    assert_eq!(result.with_marker("mindtape.task").count(), 0);
}

#[test]
fn eval_error_is_reported() {
    let (_dir, file) = project("#import \"@local/does-not-exist:9.9.9\": *\n");
    let world = HarvestWorld::new(&file).unwrap();
    assert!(harvest(&world).is_err());
}
