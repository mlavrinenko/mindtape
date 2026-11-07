#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Unit tests for the schema registry and the recorded copies under
//! `schema/output/`.

use std::path::Path;

use serde_json::Value;

use super::{LS, SCHEMAS, TASK, check_all, find, write_all};

#[test]
fn every_schema_is_stamped_with_its_canonical_id() {
    for schema in SCHEMAS {
        let value: Value = serde_json::from_str(&(schema.json)()).unwrap();
        assert_eq!(value["$id"], schema.id.canonical_url());
        assert!(value["title"].is_string(), "{value}");
        assert!(value["description"].is_string(), "{value}");
        assert!(value["$schema"].is_string(), "the draft key stays: {value}");
    }
}

#[test]
fn find_resolves_a_registered_name_and_refuses_an_unknown_one() {
    assert_eq!(find("ls").unwrap().id, LS);
    let text = crate::diag::plain(&find("lss").unwrap_err());
    assert!(text.contains("unknown schema"), "{text}");
    assert!(text.contains("ls"), "{text}");
}

#[test]
fn registered_ids_are_unique_and_start_at_version_one() {
    let mut names: Vec<&str> = SCHEMAS.iter().map(|schema| schema.id.name).collect();
    let before = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), before, "duplicate schema name");
    assert_eq!((LS.name, LS.version), ("ls", 1));
    assert_eq!((TASK.name, TASK.version), ("task", 1));
}

#[test]
fn write_then_check_passes_and_writes_deterministic_bytes() {
    let dir = tempfile::tempdir().unwrap();
    write_all(dir.path()).unwrap();
    let first = std::fs::read(dir.path().join(LS.relative_path())).unwrap();
    write_all(dir.path()).unwrap();
    let second = std::fs::read(dir.path().join(LS.relative_path())).unwrap();
    assert_eq!(first, second);
    check_all(dir.path()).unwrap();
}

#[test]
fn check_names_every_missing_and_different_file() {
    let dir = tempfile::tempdir().unwrap();
    write_all(dir.path()).unwrap();
    let ls_path = dir.path().join(LS.relative_path());
    std::fs::write(&ls_path, "{}").unwrap();
    std::fs::remove_file(dir.path().join(TASK.relative_path())).unwrap();

    let text = out_of_date(dir.path());
    let ls = mindtape::display_path(Path::new(&LS.relative_path()));
    let task = mindtape::display_path(Path::new(&TASK.relative_path()));
    assert!(text.contains(&format!("{ls} differs")), "{text}");
    assert!(text.contains(&format!("{task} is missing")), "{text}");
    assert!(text.contains("mt schema --write"), "{text}");
}

/// The check failure for `dir`, rendered as stderr would show it.
fn out_of_date(dir: &Path) -> String {
    crate::diag::plain(&check_all(dir).unwrap_err())
}
