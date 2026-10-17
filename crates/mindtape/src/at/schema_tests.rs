#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use serde_json::{Value, json};

use super::schema_json;
use crate::config::schema::tests::assert_matches_golden;

/// The schema as a JSON value.
fn schema() -> Value {
    serde_json::from_str(&schema_json()).unwrap()
}

/// The committed `schema/at.schema.json` must equal the schema generated from
/// the `at.toml` structs. On drift, re-bless:
/// `MINDTAPE_BLESS=1 cargo test -p mindtape at_schema_matches_golden`.
#[test]
fn at_schema_matches_golden() {
    assert_matches_golden(&schema_json(), "at.schema.json", "at_schema_matches_golden");
}

/// The root names `at` and `board`, refuses any other key, and carries the
/// `$id`, title and description an editor shows.
#[test]
fn at_schema_root_names_at_and_board() {
    let value = schema();
    assert_eq!(value["$id"], json!(super::AT_SCHEMA_URL));
    assert_eq!(value["type"], json!("object"));
    assert_eq!(value["additionalProperties"], json!(false));
    assert!(value["title"].is_string());
    assert!(value["description"].is_string());
    for key in ["at", "board"] {
        assert!(
            value["properties"][key].is_object(),
            "missing top-level `{key}`"
        );
    }
}

/// An `[at.<name>]` entry names `include`, `exclude`, `forks` and `board`,
/// each with its doc comment as the description.
#[test]
fn at_schema_entry_names_its_keys() {
    let entry = &schema()["$defs"]["AtEntry"];
    assert_eq!(entry["additionalProperties"], json!(false));
    for key in ["include", "exclude", "forks", "board"] {
        let property = &entry["properties"][key];
        assert!(property.is_object(), "missing entry key `{key}`");
        assert!(
            property["description"].is_string(),
            "entry key `{key}` has no description"
        );
    }
}

/// `forks` takes `true`, `false` or `"only"`, and nothing else.
#[test]
fn at_schema_forks_takes_true_false_or_only() {
    let forks = &schema()["$defs"]["AtEntry"]["properties"]["forks"];
    assert_eq!(forks["enum"], json!([false, true, "only"]));
}

/// A board layer names `columns`, `sort` and `hide`, and a column names
/// `name`, `filter`, `top` and `fallback`.
#[test]
fn at_schema_board_names_its_keys() {
    let value = schema();
    let defs = &value["$defs"];
    for key in ["columns", "sort", "hide"] {
        assert!(
            defs["BoardSpec"]["properties"][key].is_object(),
            "missing board key `{key}`"
        );
    }
    for key in ["name", "filter", "top", "fallback"] {
        assert!(
            defs["BoardColumn"]["properties"][key].is_object(),
            "missing column key `{key}`"
        );
    }
}
