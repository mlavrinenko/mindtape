#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use schemars::JsonSchema;

use super::{SCHEMA_URL, config_schema_json};
use crate::body::content::ContentAssert;
use crate::body::headers::{HeaderAction, HeaderPathSegment};
use crate::config::{DensityUnit, IdFormat, Requirement};
use crate::naming::DedupMode;

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
    assert!(
        generated == golden,
        "config JSON schema drifted from the config structs; re-bless with \
         `MINDTAPE_BLESS=1 cargo test -p mindtape schema_matches_golden`\n{}",
        first_difference(&golden, &generated)
    );
}

/// Where two schema documents first disagree, as three lines: the line number
/// and each side's text, each capped.
///
/// `assert_eq!` on the two documents instead printed both in full — ~20 000
/// characters of JSON to say "re-bless", on every run that touches a config
/// struct, in a message whose actionable half is the sentence above it. The
/// only thing the dumps added over a line number was *what* moved, and one
/// differing line says that; the whole shape of the move is
/// `git diff schema/config.schema.json` after blessing, which is the next step
/// either way.
fn first_difference(golden: &str, generated: &str) -> String {
    /// Long enough for a `"description"` line to show which field it belongs
    /// to, short enough that a prose-heavy one cannot re-flood the message.
    const CAP: usize = 200;
    let cut = |line: &str| line.chars().take(CAP).collect::<String>();
    for (index, (want, got)) in golden.lines().zip(generated.lines()).enumerate() {
        if want != got {
            let line = index + 1;
            return format!(
                "first difference at line {line}\n  committed: {}\n  generated: {}",
                cut(want),
                cut(got)
            );
        }
    }
    // No differing line in the common prefix, so one document is longer.
    format!(
        "identical for {} line(s); committed has {}, generated has {}",
        golden.lines().count().min(generated.lines().count()),
        golden.lines().count(),
        generated.lines().count()
    )
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
        "mindtape::body::headers::HeaderAction"
    );
    assert_eq!(
        <HeaderPathSegment as JsonSchema>::schema_id(),
        "mindtape::body::headers::HeaderPathSegment"
    );
    assert_eq!(
        <ContentAssert as JsonSchema>::schema_id(),
        "mindtape::body::content::ContentAssert"
    );
    assert_eq!(
        <DedupMode as JsonSchema>::schema_id(),
        "mindtape::naming::DedupMode"
    );
    assert_eq!(
        <DensityUnit as JsonSchema>::schema_id(),
        "mindtape::body::content::DensityUnit"
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
        "dicts",
        "actors",
        "roles",
    ] {
        assert!(
            value["properties"][section].is_object(),
            "missing top-level section `{section}` in schema"
        );
    }
}
