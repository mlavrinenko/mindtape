#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! Unit tests for [`Envelope`]: `$schema` leads, the payload stays flat, and
//! the derived schema carries both.

use serde::Serialize;

use super::Envelope;
use crate::cmd::schema::SchemaId;

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct Payload {
    name: String,
    count: u32,
}

fn envelope() -> Envelope<Payload> {
    Envelope::new(
        SchemaId {
            name: "demo",
            version: 1,
        },
        Payload {
            name: "alpha".to_owned(),
            count: 2,
        },
    )
}

#[test]
fn schema_is_the_first_serialized_key() {
    let text = serde_json::to_string(&envelope()).unwrap();
    assert!(text.starts_with(r#"{"$schema":"#), "{text}");
}

#[test]
fn payload_keys_are_flattened_beside_schema() {
    let value = serde_json::to_value(envelope()).unwrap();
    assert_eq!(value["name"], "alpha");
    assert_eq!(value["count"], 2);
    assert!(
        value.get("payload").is_none(),
        "the payload must not nest under its own key: {value}"
    );
}

#[test]
fn derived_schema_names_schema_and_payload_properties() {
    let schema = serde_json::to_value(schemars::schema_for!(Envelope<Payload>)).unwrap();
    let properties = &schema["properties"];
    assert!(properties.get("$schema").is_some(), "{schema}");
    assert!(properties.get("name").is_some(), "{schema}");
    assert!(properties.get("count").is_some(), "{schema}");
}
