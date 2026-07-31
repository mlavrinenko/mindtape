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
fn priority_required_yields_real_slot_for_real_kind() {
    let config =
        parse_config("[tasks.priority]\nkind = \"real\"\nmin = 0\nmax = 9\nrequired = true\n")
            .unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "priority");
    assert!(slot.call.contains("kind: \"real\""), "{}", slot.call);
}

#[test]
fn difficulty_required_yields_real_slot_for_real_kind() {
    let config =
        parse_config("[tasks.difficulty]\nkind = \"real\"\nmin = 0\nmax = 9\nrequired = true\n")
            .unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "difficulty");
    assert!(slot.call.contains("kind: \"real\""), "{}", slot.call);
}

#[test]
fn non_terminal_requirement_counts_as_required() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = \"!terminal\"\n",
    )
    .unwrap();
    assert_eq!(fields(&config), vec!["priority"]);
}

#[test]
fn strict_framework_priority_splices_a_leaf_slot_per_input() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 16\nrequired = true\n\
         framework = \"ice\"\nallow-manual = false\n",
    )
    .unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "priority");
    assert!(
        slot.call.starts_with("framework(\"ice\", "),
        "{}",
        slot.call
    );
    for leaf in ["impact", "confidence", "ease"] {
        assert!(
            slot.call
                .contains(&format!("{leaf}: slot(\"{leaf}\", kind: \"float\"")),
            "{}",
            slot.call
        );
    }
    // Number inputs carry their bounds in the prompt.
    assert!(slot.call.contains("ice impact (1..10)"), "{}", slot.call);
    assert!(slot.call.contains("ice confidence (0..1)"), "{}", slot.call);
}

#[test]
fn strict_framework_bool_inputs_become_bool_slots() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n\
         framework = \"eisenhower\"\nallow-manual = false\n",
    )
    .unwrap();
    let call = single(&config).call;
    assert!(call.starts_with("framework(\"eisenhower\", "), "{call}");
    assert!(
        call.contains("urgent: slot(\"urgent\", kind: \"bool\""),
        "{call}"
    );
    assert!(
        call.contains("important: slot(\"important\", kind: \"bool\""),
        "{call}"
    );
}

#[test]
fn strict_framework_enum_inputs_become_choice_slots() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n\
         framework = \"moscow\"\nallow-manual = false\n",
    )
    .unwrap();
    let call = single(&config).call;
    assert!(
        call.contains("level: slot(\"level\", kind: \"choice\""),
        "{call}"
    );
    for value in ["must", "should", "could", "wont"] {
        assert!(call.contains(&format!("\"{value}\"")), "{call}");
    }
}

#[test]
fn strict_custom_framework_uses_the_same_generic_spelling() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 10\nrequired = true\n\
         framework = \"impact-only\"\nallow-manual = false\n\
         [tasks.priority.frameworks.impact-only]\nscore = \"impact\"\n\
         [tasks.priority.frameworks.impact-only.inputs.impact]\n\
         type = \"number\"\nmin = 0\nmax = 10\n",
    )
    .unwrap();
    let call = single(&config).call;
    assert!(call.starts_with("framework(\"impact-only\", "), "{call}");
    assert!(
        call.contains("impact: slot(\"impact\", kind: \"float\""),
        "{call}"
    );
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
