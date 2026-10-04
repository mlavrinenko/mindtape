//! Tests for config-required slot derivation.
#![allow(clippy::unwrap_used)]

use crate::config::parse_config;

use super::{RequiredSlot, required_core_slots};

fn fields(config: &crate::config::Config) -> Vec<String> {
    required_core_slots(config, None)
        .into_iter()
        .map(|s| s.field)
        .collect()
}

/// The single required slot a config yields, asserting there is exactly one.
fn single(config: &crate::config::Config) -> RequiredSlot {
    let mut slots = required_core_slots(config, None);
    assert_eq!(slots.len(), 1);
    slots.pop().unwrap()
}

#[test]
fn default_config_requires_nothing() {
    let config = parse_config("[tasks]\nroots = [\"tasks\"]\n").unwrap();
    assert!(required_core_slots(&config, None).is_empty());
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
    let slot = single(&config);
    assert_eq!(slot.field, "priority");
    // `allow-manual = true` keeps the scalar route: the framework call is for
    // strict frameworks only, and a mutant that splices it here would demand
    // one leaf per framework input instead of the int the config asked for.
    assert!(
        slot.call.starts_with("slot(\"priority\", kind: \"int\""),
        "{}",
        slot.call
    );
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
fn required_extra_yields_choice_slot_named_after_it() {
    let config = parse_config(
        "[tasks.fields.x-machine]\nvalues = [\"alpha\", \"desk\"]\nrequired = true\ndefault = \"alpha\"\n",
    )
    .unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "x-machine");
    assert_eq!(slot.config_path, "[tasks.fields.x-machine]");
    assert_eq!(
        slot.call,
        r#"slot("x-machine", kind: "choice", choices: ("alpha", "desk", ), default: "alpha", example: "alpha")"#
    );
    assert_eq!(slot.default.as_deref(), Some("\"alpha\""));
}

#[test]
fn optional_extra_yields_no_slot() {
    let config = parse_config("[tasks.fields.x-machine]\nvalues = [\"alpha\"]\n").unwrap();
    assert!(required_core_slots(&config, None).is_empty());
}

#[test]
fn required_estimate_yields_text_slot_with_unit_hint() {
    let config = parse_config("[tasks.estimate]\nunits = [\"points\"]\nrequired = true\n").unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "estimate");
    assert!(slot.call.contains("(points: 1)"), "{}", slot.call);
}

#[test]
fn order_is_priority_difficulty_kind_assignee_estimate() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n\
         [tasks.difficulty]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n\
         [tasks.fields.kind]\nvalues = [\"a\"]\nrequired = true\n\
         [tasks.assignee]\nrequired = true\n\
         [tasks.estimate]\nrequired = true\n",
    )
    .unwrap();
    assert_eq!(
        fields(&config),
        vec!["priority", "difficulty", "kind", "assignee", "estimate"]
    );
}

#[test]
fn required_assignee_yields_env_backed_plain_slot() {
    let config = parse_config("[tasks.assignee]\nrequired = true\n").unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "assignee");
    assert_eq!(slot.config_path, "[tasks.assignee]");
    assert_eq!(slot.env, Some("MINDTAPE_PRINCIPAL"));
    assert_eq!(slot.default, None);
    assert!(
        slot.call.contains(r#"env: "MINDTAPE_PRINCIPAL""#),
        "{}",
        slot.call
    );
    assert!(!slot.call.contains("kind: \"choice\""), "{}", slot.call);
}

/// The kind slot's config default is carried out so a CLI can tell a
/// default-filled value from a hand-supplied one; the env-backed assignee slot
/// names its environment; and the rest expose neither.
#[test]
fn required_slots_carry_their_resolution() {
    let config = parse_config(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n\
         [tasks.difficulty]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n\
         [tasks.fields.kind]\nvalues = [\"feature\"]\nrequired = true\ndefault = \"feature\"\n\
         [tasks.assignee]\nrequired = true\n\
         [tasks.estimate]\nrequired = true\n",
    )
    .unwrap();
    let slots = required_core_slots(&config, None);
    let slot = |field: &str| {
        slots
            .iter()
            .find(|slot| slot.field == field)
            .unwrap_or_else(|| panic!("{field} is required"))
    };
    assert_eq!(slot("kind").default.as_deref(), Some("\"feature\""));
    assert_eq!(slot("kind").config_path, "[tasks.fields.kind]");
    assert_eq!(slot("assignee").env, Some("MINDTAPE_PRINCIPAL"));
    for field in ["priority", "difficulty", "estimate"] {
        assert_eq!(slot(field).env, None, "{field}");
        assert_eq!(slot(field).default, None, "{field}");
    }
    assert_eq!(slot("priority").config_path, "[tasks.priority]");
    assert_eq!(slot("difficulty").config_path, "[tasks.difficulty]");
    assert_eq!(slot("estimate").config_path, "[tasks.estimate]");
}

#[test]
fn required_assignee_with_allowed_yields_choice_slot() {
    let config =
        parse_config("[tasks.assignee]\nrequired = true\nallowed = [\"ada\", \"lin\"]\n").unwrap();
    let slot = single(&config);
    assert_eq!(slot.field, "assignee");
    assert!(slot.call.contains("kind: \"choice\""), "{}", slot.call);
    for actor in ["ada", "lin"] {
        assert!(slot.call.contains(&format!("\"{actor}\"")), "{}", slot.call);
    }
    assert!(
        slot.call.contains(r#"env: "MINDTAPE_PRINCIPAL""#),
        "{}",
        slot.call
    );
}

/// A task filed at a terminal status owes nothing `required = "!terminal"`
/// asks, while a requirement reading another field stays, because the status
/// alone cannot settle it.
#[test]
fn a_terminal_status_drops_what_only_open_tasks_need() {
    let config = parse_config(
        "[tasks.difficulty]\nrequired = \"!terminal\"\n\
         [tasks.estimate]\nunits = [\"days\"]\nrequired = 'status == \"proposed\" && kind == \"bug\"'\n",
    )
    .unwrap();
    let at = |status| {
        required_core_slots(&config, Some(status))
            .into_iter()
            .map(|slot| slot.field)
            .collect::<Vec<_>>()
    };
    assert_eq!(at("done"), ["estimate"]);
    assert_eq!(at("proposed"), ["difficulty", "estimate"]);
}

/// The slot each `field` yields under `toml`, by field name.
fn slot_for(toml: &str, field: &str) -> RequiredSlot {
    let config = parse_config(toml).unwrap();
    required_core_slots(&config, None)
        .into_iter()
        .find(|slot| slot.field == field)
        .unwrap()
}

fn example_of(slot: &RequiredSlot) -> String {
    let call = format!("#f({})", slot.call);
    let specs = crate::template::discover_slots(&call);
    assert_eq!(specs.len(), 1, "{}", slot.call);
    specs.into_iter().next().unwrap().example.unwrap()
}

#[test]
fn a_choice_field_s_example_is_its_default_else_its_first_value() {
    let first = slot_for(
        "[tasks.fields.kind]\nvalues = [\"feat\", \"fix\"]\nrequired = true\n",
        "kind",
    );
    assert_eq!(example_of(&first), "\"feat\"");
    let defaulted = slot_for(
        "[tasks.fields.kind]\nvalues = [\"feat\", \"fix\"]\ndefault = \"fix\"\nrequired = true\n",
        "kind",
    );
    assert_eq!(example_of(&defaulted), "\"fix\"");
}

#[test]
fn a_free_field_s_example_names_the_field() {
    let slot = slot_for("[tasks.fields.x-team]\nrequired = true\n", "x-team");
    assert_eq!(example_of(&slot), "\"team\"");
}

#[test]
fn an_assignee_s_example_is_the_first_allowed_and_keeps_its_choices() {
    let slot = slot_for(
        "[[actors]]\nname = \"max\"\nkind = \"human\"\n[[actors]]\nname = \"agent\"\nkind = \"llm\"\n\
         [tasks.assignee]\nallowed = [\"max\", \"agent\"]\nrequired = \"!terminal\"\n",
        "assignee",
    );
    assert_eq!(example_of(&slot), "\"max\"");
    assert!(
        slot.call.contains("choices: (\"max\", \"agent\", )"),
        "{}",
        slot.call
    );
    assert!(
        slot.call.contains("env: \"MINDTAPE_PRINCIPAL\""),
        "{}",
        slot.call
    );
}

#[test]
fn an_open_assignee_s_example_is_the_first_actor_else_a_placeholder() {
    let actor = slot_for(
        "[[actors]]\nname = \"ada\"\nkind = \"human\"\n[tasks.assignee]\nrequired = true\n",
        "assignee",
    );
    assert_eq!(example_of(&actor), "\"ada\"");
    let none = slot_for("[tasks.assignee]\nrequired = true\n", "assignee");
    assert_eq!(example_of(&none), "\"someone\"");
}

#[test]
fn a_numeric_field_s_example_is_its_scale_midpoint() {
    let priority = slot_for(
        "[tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n",
        "priority",
    );
    assert_eq!(example_of(&priority), "4");
    let difficulty = slot_for(
        "[tasks.difficulty]\nkind = \"real\"\nmin = 1\nmax = 2\nrequired = true\n",
        "difficulty",
    );
    assert_eq!(example_of(&difficulty), "1.5");
}

#[test]
fn each_framework_leaf_s_example_sits_inside_its_range() {
    let slot = slot_for(
        "[tasks.priority]\nframework = \"ice\"\nallow-manual = false\nrequired = true\n",
        "priority",
    );
    let call = format!("#f({})", slot.call);
    let examples: Vec<(String, String)> = crate::template::discover_slots(&call)
        .into_iter()
        .map(|spec| (spec.name, spec.example.unwrap()))
        .collect();
    assert!(!examples.is_empty(), "{}", slot.call);
    let config = parse_config("").unwrap();
    let ice = config.tasks.priority.frameworks.get("ice").unwrap();
    for (name, example) in examples {
        let input = ice.inputs.get(&name).unwrap();
        let value: f64 = example.parse().unwrap();
        assert!(
            input.min.is_none_or(|lo| value >= lo) && input.max.is_none_or(|hi| value <= hi),
            "{name} = {example}"
        );
    }
}

#[test]
fn an_estimate_s_example_is_one_of_its_first_unit() {
    let slot = slot_for(
        "[tasks.estimate]\nunits = [\"src-files\"]\nrequired = true\n",
        "estimate",
    );
    assert_eq!(example_of(&slot), "\"(src-files: 1)\"");
}
