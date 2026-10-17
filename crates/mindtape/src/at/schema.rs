//! JSON Schema for `at.toml`, derived from its structs.
//!
//! `schemars` derives the entry and board types, and each key's doc comment
//! becomes its description. [`super::Forks`] has a hand-written
//! `Deserialize`, which schemars cannot read, so [`forks`] spells out the
//! values it takes. `mt install` writes the result beside
//! `config.schema.json`, and the shipped `schema/at.schema.json` is this
//! output, held to it by [`tests`].

use schemars::{Schema, SchemaGenerator, json_schema};

use super::RawFile;
use crate::config::schema::stamped_schema_json;

/// Where the schema is published, its `$id`. An editor fetches and caches it,
/// so an `at.toml` gets completion with no local install.
pub(super) const AT_SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/at.schema.json";

/// Build the JSON Schema for `at.toml` as pretty-printed JSON (draft 2020-12,
/// trailing newline), generated from the file's structs.
#[must_use]
pub fn schema_json() -> String {
    stamped_schema_json(
        schemars::schema_for!(RawFile),
        AT_SCHEMA_URL,
        "Mindtape projects and sets",
        "Schema for `at.toml` in the Mindtape config directory. It names the \
         projects and sets a leading `@name` selects, and the board a bare \
         `mt @name` shows.",
    )
}

/// The values `forks` takes. The field's doc comment says what each means.
pub(super) fn forks(_generator: &mut SchemaGenerator) -> Schema {
    json_schema!({ "enum": [false, true, "only"] })
}

#[cfg(test)]
#[path = "schema_tests.rs"]
mod tests;
