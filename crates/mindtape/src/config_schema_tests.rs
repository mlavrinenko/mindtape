#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use schemars::JsonSchema;

use super::{SCHEMA_URL, config_schema_json};
use crate::body_headers::{HeaderAction, HeaderPathSegment};
use crate::config::{IdFormat, Requirement};

/// The committed schema artifact (`schema/config.schema.json`) must equal the
/// schema generated from the config structs. On drift, re-bless:
/// `MINDTAPE_BLESS=1 cargo test -p mindtape schema_matches_golden`.
#[test]
fn schema_matches_golden() {
    let generated = config_schema_json();
    let golden_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../schema/config.schema.json"
    );
    if std::env::var_os("MINDTAPE_BLESS").is_some() {
        std::fs::write(golden_path, &generated).unwrap();
        return;
    }
    let golden = std::fs::read_to_string(golden_path).unwrap_or_default();
    assert_eq!(
        generated, golden,
        "config JSON schema drifted from the config structs; re-bless with \
         `MINDTAPE_BLESS=1 cargo test -p mindtape schema_matches_golden`"
    );
}

/// Each hand-written `JsonSchema::schema_id` returns its unique dotted path.
/// cargo-mutants would replace the string with `""` or `"xyzzy"`; the assertion
/// on the exact string kills all 4 variants (empty-Borrowed, empty-Owned,
/// xyzzy-Borrowed, xyzzy-Owned) because every mutant returns a different value.
#[test]
fn hand_written_schema_ids_are_correct() {
    assert_eq!(
        <IdFormat as JsonSchema>::schema_id(),
        "mindtape::config::IdFormat"
    );
    assert_eq!(
        <Requirement as JsonSchema>::schema_id(),
        "mindtape::config::Requirement"
    );
    assert_eq!(
        <HeaderAction as JsonSchema>::schema_id(),
        "mindtape::body_headers::HeaderAction"
    );
    assert_eq!(
        <HeaderPathSegment as JsonSchema>::schema_id(),
        "mindtape::body_headers::HeaderPathSegment"
    );
}

/// Sanity: the generated schema is valid JSON with the expected root shape and
/// the top-level config sections present as properties.
#[test]
fn schema_has_expected_root() {
    let value: serde_json::Value = serde_json::from_str(&config_schema_json()).unwrap();
    assert_eq!(value["$id"], serde_json::json!(SCHEMA_URL));
    assert_eq!(value["type"], serde_json::json!("object"));
    for section in [
        "project",
        "tasks",
        "ls",
        "templates",
        "add",
        "go",
        "typst",
        "format",
        "views",
    ] {
        assert!(
            value["properties"][section].is_object(),
            "missing top-level section `{section}` in schema"
        );
    }
}
