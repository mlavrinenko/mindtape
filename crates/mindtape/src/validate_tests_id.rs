//! The `[tasks.id] format` check: a plain-string id under the default config,
//! and the `uuid-v4` / `uuid-v7` formats accepting their own version and
//! rejecting the other (and any non-UUID string). Split out of
//! `validate_tests_fields.rs` (file-size gate).

#![allow(clippy::unwrap_used)]

use super::tests::task;
use super::validate;
use crate::config::{Config, parse_config};

#[test]
fn default_config_accepts_plain_string_id() {
    let mut t = task("my-task", "wip");
    t.priority = None; // remove priority so the only check is id
    // default config has no priority constraints
    let violations = validate(&[t], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "id"));
}

#[test]
fn uuid_v4_config_accepts_valid_v4() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v4\"\n").unwrap();
    let mut t = task("550e8400-e29b-41d4-a716-446655440000", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(!violations.iter().any(|v| v.field == "id"));
}

#[test]
fn uuid_v4_config_rejects_non_v4() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v4\"\n").unwrap();
    // This is a v7 UUID
    let mut t = task("018f4e2a-0000-7000-8000-000000000000", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "v7 id should be rejected by uuid-v4 format"
    );
}

#[test]
fn uuid_v4_config_rejects_non_uuid_string() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v4\"\n").unwrap();
    let mut t = task("my-plain-task", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "plain string rejected by uuid-v4 format"
    );
}

#[test]
fn uuid_v7_config_accepts_valid_v7() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v7\"\n").unwrap();
    // UUID v7: version nibble = 7
    let mut t = task("018f4e2a-0000-7000-8000-000000000000", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(!violations.iter().any(|v| v.field == "id"));
}

#[test]
fn uuid_v7_rejects_v4() {
    let cfg = parse_config("[tasks.id]\nformat = \"uuid-v7\"\n").unwrap();
    let mut t = task("550e8400-e29b-41d4-a716-446655440000", "wip");
    t.priority = None;
    let violations = validate(&[t], &cfg);
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "v4 id should be rejected by uuid-v7 format"
    );
}
