//! JSON Schema for `.mindtape/config.toml`, derived from the config structs.
//!
//! `schemars` derives the bulk straight off [`crate::config::Config`] and its
//! nested types — the doc comments on each field become the schema's
//! descriptions — so the schema tracks the types with no hand-maintained
//! mirror. This module supplies the manual `JsonSchema` impls for the four
//! config types whose `Deserialize` is hand-written (schemars cannot read a
//! custom deserializer), assembles the root schema, and stamps a title / `$id`.
//!
//! `mt install` writes the schema next to the app-wide config; `mt init`
//! scaffolds a project config carrying a `#:schema` comment directive that
//! points at it, so a TOML-schema-aware editor (tombi, Taplo) assists filling
//! values. The shipped `schema/config.schema.json` is exactly this output,
//! gate-checked for drift by [`tests`].

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};

use crate::config::{HeaderAction, HeaderPathSegment, IdFormat, Requirement};

/// Canonical published location of this schema. Used as the schema `$id`, and
/// as the `#:schema` directive `mt init` writes by default — a checked-in
/// config has to mean the same thing on someone else's machine, and an editor
/// fetches and caches the URL, so no local install is required.
pub const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/config.schema.json";

/// Build the JSON Schema for `.mindtape/config.toml` as pretty-printed JSON
/// (draft 2020-12, trailing newline). Generated from [`crate::config::Config`],
/// so it can never drift from the accepted config shape.
///
/// # Panics
/// Panics only if the schemars-generated schema fails to serialise to JSON —
/// unreachable for the well-formed `Config` schema.
#[must_use]
pub fn config_schema_json() -> String {
    let mut root = serde_json::to_value(schemars::schema_for!(crate::config::Config))
        .expect("Config JSON schema serialises to a value");
    if let serde_json::Value::Object(map) = &mut root {
        map.insert("$id".to_owned(), SCHEMA_URL.into());
        map.insert("title".to_owned(), "MindTape project configuration".into());
        map.insert(
            "description".to_owned(),
            "Schema for `.mindtape/config.toml`, MindTape's per-project \
             configuration. MindTape is a white-list: every section is optional \
             and inherits the shipped floor when omitted."
                .into(),
        );
    }
    let mut out = serde_json::to_string_pretty(&root).expect("schema serialises to JSON");
    out.push('\n');
    out
}

impl JsonSchema for IdFormat {
    fn schema_name() -> Cow<'static, str> {
        "IdFormat".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "mindtape::config::IdFormat".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Format requirement for an explicit task `id:` field.",
            "type": "string",
            "enum": ["string", "uuid-v4", "uuid-v7"]
        })
    }
}

impl JsonSchema for Requirement {
    fn schema_name() -> Cow<'static, str> {
        "Requirement".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "mindtape::config::Requirement".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Whether a field is required: `false` (never), `true` \
                (always), or a filter expression matching exactly the tasks that must \
                carry it (e.g. `\"!terminal\"` — required until the task reaches a \
                terminal status).",
            "oneOf": [
                { "type": "boolean" },
                { "type": "string" }
            ]
        })
    }
}

impl JsonSchema for HeaderAction {
    fn schema_name() -> Cow<'static, str> {
        "HeaderAction".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "mindtape::body_headers::HeaderAction".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "The shape of a body-header rule.",
            "type": "string",
            "enum": ["require", "forbid", "unique", "order", "allowlist"]
        })
    }
}

impl JsonSchema for HeaderPathSegment {
    fn schema_name() -> Cow<'static, str> {
        "HeaderPathSegment".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "mindtape::body_headers::HeaderPathSegment".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "One step in a heading path: an exact string (direct-child \
                match) or `{ match = \"regex\", recursive = bool }`.",
            "oneOf": [
                { "type": "string" },
                {
                    "type": "object",
                    "properties": {
                        "match": {
                            "type": "string",
                            "description": "Rust regex matched against heading text."
                        },
                        "recursive": {
                            "type": "boolean",
                            "description": "Match any descendant (true) or only direct children (false)."
                        }
                    },
                    "required": ["match"],
                    "additionalProperties": false
                }
            ]
        })
    }
}

#[cfg(test)]
#[path = "config_schema_tests.rs"]
mod tests;
