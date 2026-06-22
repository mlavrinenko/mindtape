//! Tests for config-required slot derivation.
#![allow(clippy::unwrap_used)]

use crate::config::parse_config;

use super::{RequiredSlot, required_core_slots};

fn fields(config: &crate::config::Config) -> Vec<&'static str> {
    required_core_slots(config)
        .into_iter()
        .map(|s| s.field)
        .collect()
}

/// The single required slot a config yields, asserting there is exactly one.
fn single(config: &crate::config::Config) -> RequiredSlot {
    let mut slots = required_core_slots(config);
    assert_eq!(slots.len(), 1);
    slots.pop().unwrap()
}

#[test]
fn default_config_requires_nothing() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    assert!(required_core_slots(&config).is_empty());
}

#[test]
fn priority_required_yields_int_slot() {
    let config =
        parse_config("[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n")
            .unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "priority");
    assert!(slot.call.contains("kind: \"int\""), "{}", slot.call);
}

#[test]
fn non_terminal_requirement_counts_as_required() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = \"non-terminal\"\n",
    )
    .unwrap();
    assert_eq!(fields(&config), vec!["priority"]);
}

#[test]
fn strict_framework_priority_is_omitted() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 16\nrequired = true\n\
         framework = \"ice\"\nallow-manual = false\n",
    )
    .unwrap();
    assert!(required_core_slots(&config).is_empty());
}

#[test]
fn framework_with_manual_priority_still_offers_int_slot() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 16\nrequired = true\n\
         framework = \"ice\"\nallow-manual = true\n",
    )
    .unwrap();
    assert_eq!(fields(&config), vec!["priority"]);
}

#[test]
fn required_kind_yields_choice_slot_with_values() {
    let config = parse_config(
        "[tasks.fields.kind]\nvalues = [\"feature\", \"bug\"]\nrequired = true\ndefault = \"feature\"\n",
    )
    .unwrap();
    let call = single(&config).call;
    assert!(call.contains("kind: \"choice\""), "{call}");
    assert!(call.contains("\"feature\""), "{call}");
    assert!(call.contains("default: \"feature\""), "{call}");
}

#[test]
fn required_estimate_yields_text_slot_with_unit_hint() {
    let config = parse_config("[tasks.estimate]\nunits = [\"points\"]\nrequired = true\n").unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "estimate");
    assert!(slot.call.contains("(points: 1)"), "{}", slot.call);
}

#[test]
fn order_is_priority_difficulty_kind_estimate() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n\
         [tasks.difficulty]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n\
         [tasks.fields.kind]\nvalues = [\"a\"]\nrequired = true\n\
         [tasks.estimate]\nrequired = true\n",
    )
    .unwrap();
    assert_eq!(
        fields(&config),
        vec!["priority", "difficulty", "kind", "estimate"]
    );
}
