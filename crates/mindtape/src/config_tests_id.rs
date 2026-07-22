#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{IdFormat, Requirement, parse_config};

#[test]
fn id_format_defaults_to_string() {
    let config = parse_config("").unwrap();
    assert_eq!(config.tasks.id.format, IdFormat::String);
}

#[test]
fn id_format_parses_uuid_v4() {
    let config = parse_config("[tasks.id]\nformat = \"uuid-v4\"\n").unwrap();
    assert_eq!(config.tasks.id.format, IdFormat::UuidV4);
}

#[test]
fn id_format_parses_uuid_v7() {
    let config = parse_config("[tasks.id]\nformat = \"uuid-v7\"\n").unwrap();
    assert_eq!(config.tasks.id.format, IdFormat::UuidV7);
}

#[test]
fn id_format_rejects_unknown() {
    assert!(parse_config("[tasks.id]\nformat = \"base62\"\n").is_err());
}

#[test]
fn id_format_generates_valid_uuids() {
    let v4 = IdFormat::UuidV4.generate().unwrap();
    assert!(uuid::Uuid::parse_str(&v4).is_ok());
    let v7 = IdFormat::UuidV7.generate().unwrap();
    assert!(uuid::Uuid::parse_str(&v7).is_ok());
    assert_eq!(IdFormat::String.generate(), None);
}

#[test]
fn ls_rules_columns_resolves_known_names() {
    let config = parse_config("[[ls.rules]]\ncolumns = [\"title\", \"status\"]\n").unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn requirement_accepts_string_true_false() {
    let config = parse_config("[tasks.fields.priority]\nrequired = \"true\"\n").unwrap();
    assert_eq!(
        config.tasks.fields.get("priority").unwrap().required,
        Requirement::Always
    );

    let config = parse_config("[tasks.fields.priority]\nrequired = \"false\"\n").unwrap();
    assert_eq!(
        config.tasks.fields.get("priority").unwrap().required,
        Requirement::No
    );
}

/// A string that is neither `"true"`/`"false"` nor a usable filter expression
/// parses (it is a filter expression by shape) and fails config validation,
/// naming the field — one config error instead of one violation per task.
#[test]
fn requirement_rejects_an_unusable_expression_at_validation() {
    let config = parse_config("[tasks.fields.priority]\nrequired = \"maybe\"\n").unwrap();
    let problems = config.validate().unwrap_err();
    assert!(
        problems.contains("tasks.fields.priority.required"),
        "the offending field must be named: {problems}"
    );
}
