//! JSON Schema for `.mindtape/config.toml`, derived from the config structs.
//!
//! `schemars` derives the bulk straight off [`crate::config::Config`] and its
//! nested types — the doc comments on each field become the schema's
//! descriptions — so the schema tracks the types with no hand-maintained
//! mirror. This module supplies the manual `JsonSchema` impls for the config
//! types whose `Deserialize` is hand-written (schemars cannot read a custom
//! deserializer), assembles the root schema, and stamps a title / `$id`.
//!
//! `mt install` writes the schema next to the app-wide config; `mt init`
//! scaffolds a project config carrying a `#:schema` comment directive that
//! points at it, so a TOML-schema-aware editor (tombi, Taplo) assists filling
//! values. The shipped `schema/config.schema.json` is exactly this output,
//! gate-checked for drift by [`tests`].

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};

use crate::config::{
    ContentAssert, DensityUnit, HeaderAction, HeaderPathSegment, IdFormat, Requirement,
};
use crate::model::naming::DedupMode;

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
        map.insert("title".to_owned(), "Mindtape project configuration".into());
        map.insert(
            "description".to_owned(),
            "Schema for `.mindtape/config.toml`, the per-project Mindtape \
             configuration. Every section is optional; an omitted section takes \
             the shipped defaults."
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

impl JsonSchema for DedupMode {
    fn schema_name() -> Cow<'static, str> {
        "DedupMode".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "mindtape::model::naming::DedupMode".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "`mt add`'s collision policy when the destination stem \
                already exists: `fail` bails before writing, `skip` exits 0 without \
                writing and reports the existing path, `add-suffix` mints `-2`, \
                `-3`, … until a free stem turns up. `mt add --dedup` overrides this \
                for one run; see `[tasks.naming] dedup` for the unset default.",
            "type": "string",
            "enum": ["fail", "skip", "add-suffix"]
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

impl JsonSchema for DensityUnit {
    fn schema_name() -> Cow<'static, str> {
        "DensityUnit".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "mindtape::body::content::DensityUnit".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Unit a `density` assert counts matches in: `section` \
                (the default), `paragraph` or `sentence`.",
            "type": "string",
            "enum": ["section", "paragraph", "sentence"]
        })
    }
}

impl JsonSchema for ContentAssert {
    fn schema_name() -> Cow<'static, str> {
        "ContentAssert".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "mindtape::body::content::ContentAssert".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let density = generator.subschema_for::<crate::body::content::DensitySpec>();
        json_schema!({
            "description": "The assertion made over a matched section's content: the \
                string `\"non-empty\"`, or a one-key table `{ min-words = N }`, \
                `{ max-words = N }`, `{ must-match = \"regex\" }`, \
                `{ must-not-match = \"regex\" }`, or `{ density = { pattern = \"regex\", \
                max = N, per = \"section\" } }`.",
            "oneOf": [
                { "type": "string", "enum": ["non-empty"] },
                {
                    "type": "object",
                    "properties": {
                        "min-words": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "The section must contain at least this many words."
                        }
                    },
                    "required": ["min-words"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "max-words": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "The section must contain at most this many words."
                        }
                    },
                    "required": ["max-words"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "must-match": {
                            "type": "string",
                            "description": "Rust regex the section's source text must match."
                        }
                    },
                    "required": ["must-match"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "must-not-match": {
                            "type": "string",
                            "description": "Rust regex the section's source text must not match."
                        }
                    },
                    "required": ["must-not-match"],
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "properties": {
                        "density": density
                    },
                    "required": ["density"],
                    "additionalProperties": false
                }
            ]
        })
    }
}

impl JsonSchema for HeaderAction {
    fn schema_name() -> Cow<'static, str> {
        "HeaderAction".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "mindtape::body::headers::HeaderAction".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "What a heading rule checks: `require`, `forbid` or \
                `unique` over `path`, `order` of `before` and `after`, or an \
                `allowlist` of headings.",
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
        "mindtape::body::headers::HeaderPathSegment".into()
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
#[path = "schema_tests.rs"]
mod tests;
