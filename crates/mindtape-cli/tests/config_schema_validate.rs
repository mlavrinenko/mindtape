//! Validate the shipped configs against the generated JSON Schema.
//!
//! The schema is derived from the config structs, and the shipped configs parse
//! into those same structs — so agreement is expected by construction. This
//! guards the seam that construction does not cover: the four hand-written
//! `JsonSchema` impls (`config_schema.rs`) and any accidental over/under-strict
//! keyword, caught end-to-end through a real draft-2020-12 validator.
#![allow(clippy::unwrap_used)]

/// Convert TOML config text to a `serde_json` value for schema validation.
fn toml_to_json(text: &str) -> serde_json::Value {
    let table: toml::Value = toml::from_str(text).unwrap();
    serde_json::to_value(table).unwrap()
}

/// Every shipped config — the embedded floor and both `mt init`
/// scaffolds — must validate against the generated schema.
#[test]
fn shipped_configs_validate_against_schema() {
    let schema: serde_json::Value = serde_json::from_str(&mindtape::config_schema_json()).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();

    let cases = [
        ("embedded floor", mindtape::embedded_defaults()),
        ("slim scaffold", include_str!("../src/slim-config.toml")),
        ("full scaffold", include_str!("../src/default-config.toml")),
        (
            "dogfood config",
            include_str!("../../../.mindtape/config.toml"),
        ),
    ];
    for (name, text) in cases {
        let instance = toml_to_json(text);
        let errors: Vec<String> = validator
            .iter_errors(&instance)
            .map(|error| error.to_string())
            .collect();
        assert!(
            errors.is_empty(),
            "{name} fails the schema:\n{}",
            errors.join("\n")
        );
    }
}

/// An unknown top-level key must be rejected (the schema mirrors the config's
/// `deny_unknown_fields`), so a typo surfaces in the editor.
#[test]
fn unknown_key_is_rejected() {
    let schema: serde_json::Value = serde_json::from_str(&mindtape::config_schema_json()).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let instance = toml_to_json("[taskz]\nroots = [\"tasks\"]\n");
    assert!(!validator.is_valid(&instance));
}
