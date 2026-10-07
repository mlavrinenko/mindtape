#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use crate::config::parse_config;
use crate::model::numeric::{InputValue, NumValue};

fn framework_value(name: &str, inputs: &[(&str, InputValue)]) -> NumValue {
    NumValue::Framework {
        name: name.to_owned(),
        inputs: inputs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect(),
    }
}

fn ice_inputs(impact: f64, confidence: f64, ease: f64) -> Vec<(&'static str, InputValue)> {
    vec![
        ("impact", InputValue::Num(impact)),
        ("confidence", InputValue::Num(confidence)),
        ("ease", InputValue::Num(ease)),
    ]
}

#[test]
fn builtins_available_after_parse() {
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let frameworks = &config.tasks.priority.frameworks;
    for name in ["ice", "rice", "wsjf", "eisenhower", "moscow"] {
        assert!(frameworks.contains_key(name), "missing built-in {name}");
    }
}

#[test]
fn resolve_scores_ice_value() {
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let value = framework_value("ice", &ice_inputs(10.0, 0.9, 10.0));
    assert_eq!(config.tasks.priority.resolve(&value), Some(8.0));
}

#[test]
fn ice_validates_against_default_scale() {
    let config = parse_config(
        "[tasks.priority]\nframework = \"ice\"\nmin = 0\nmax = 9\n\
         [[tasks.priority.bands]]\nlabel = \"low\"\nmin = 0\nmax = 9\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn categorical_cell_uses_category_display() {
    let config = parse_config("[tasks.priority]\nframework = \"eisenhower\"\n").unwrap();
    let value = framework_value(
        "eisenhower",
        &[
            ("urgent", InputValue::Bool(true)),
            ("important", InputValue::Bool(true)),
        ],
    );
    let cell = config.tasks.priority.cell(&value).unwrap();
    assert_eq!(cell.label, "do");
    assert_eq!(cell.glyph.as_deref(), Some("🔥"));
    // The "do" quadrant is the highest-scoring category.
    assert_eq!(config.tasks.priority.resolve(&value), Some(9.0));
}

#[test]
fn check_input_values_flags_out_of_bounds() {
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let ice = config.tasks.priority.frameworks.get("ice").unwrap();
    let inputs: BTreeMap<String, InputValue> = ice_inputs(99.0, 0.5, 5.0)
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
    let problems = ice.check_input_values(&inputs);
    assert!(
        problems.iter().any(|p| p.contains("impact")),
        "got: {problems:?}"
    );
}

fn single_value_enum_framework() -> crate::framework::Framework {
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\n\
         [tasks.priority.frameworks.x]\nscore = \"1\"\n\
         [tasks.priority.frameworks.x.inputs.color]\ntype = \"enum\"\nvalues = [\"red\"]\n",
    )
    .unwrap();
    config.tasks.priority.frameworks.get("x").unwrap().clone()
}

#[test]
fn user_definition_deep_merges_over_builtin() {
    // Override only ICE's score; its built-in inputs must survive the merge.
    let config = parse_config(
        "[tasks.priority]\nframework = \"ice\"\n\
         [tasks.priority.frameworks.ice]\nscore = \"round(impact)\"\n",
    )
    .unwrap();
    let ice = config.tasks.priority.frameworks.get("ice").unwrap();
    assert_eq!(ice.score.as_deref(), Some("round(impact)"));
    assert!(ice.inputs.contains_key("confidence"));
    let value = framework_value("ice", &ice_inputs(5.0, 1.0, 1.0));
    assert_eq!(config.tasks.priority.resolve(&value), Some(5.0));
}

// --- check_input_value boundary regression tests ---
// Each test distinguishes real behaviour from a specific mutation in framework.rs.

#[test]
fn check_input_value_accepts_min_boundary() {
    // check_input_value < → == or < → <=. Value at min boundary.
    // Real: *num < min → 1.0 < 1.0 = false → no error.
    // Mutant ==: 1.0 == 1.0 = true → error.
    // Mutant <=: 1.0 <= 1.0 = true → error.
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let ice = config.tasks.priority.frameworks.get("ice").unwrap();
    let inputs: BTreeMap<String, InputValue> = ice_inputs(1.0, 0.0, 1.0)
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    let problems = ice.check_input_values(&inputs);
    assert!(
        !problems.iter().any(|p| p.contains("out of bounds")),
        "got: {problems:?}"
    );
}

#[test]
fn check_input_value_accepts_bool() {
    // check_input_value delete Bool match arm. Fallthrough → "wrong type".
    let config = parse_config("[tasks.priority]\nframework = \"eisenhower\"\n").unwrap();
    let fw = config.tasks.priority.frameworks.get("eisenhower").unwrap();
    let inputs: BTreeMap<String, InputValue> = [
        ("urgent", InputValue::Bool(true)),
        ("important", InputValue::Bool(true)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    let problems = fw.check_input_values(&inputs);
    assert!(problems.is_empty(), "got: {problems:?}");
}

#[test]
fn check_input_value_accepts_enum() {
    // check_input_value delete Enum arm. delete ! or == → !=.
    let fw = single_value_enum_framework();
    let inputs: BTreeMap<String, InputValue> = [("color", InputValue::Text("red".into()))]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    let problems = fw.check_input_values(&inputs);
    assert!(problems.is_empty(), "got: {problems:?}");
}

#[test]
fn check_input_value_rejects_invalid_enum() {
    // check_input_value delete ! or == → !=. Invalid value → real errors, mutant passes.
    let fw = single_value_enum_framework();
    let inputs: BTreeMap<String, InputValue> = [("color", InputValue::Text("blue".into()))]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    let problems = fw.check_input_values(&inputs);
    assert!(
        problems.iter().any(|p| p.contains("not allowed")),
        "got: {problems:?}"
    );
}

#[test]
fn active_framework_returns_configured_framework() {
    // active_framework → None or Some(Default::default()) ().
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let fw = config.tasks.priority.active_framework();
    assert!(fw.is_some());
    assert_eq!(
        fw.unwrap().score.as_deref(),
        Some("round(impact * confidence * ease * 0.09)")
    );
}

#[test]
fn evaluate_returns_correct_score() {
    // evaluate → Ok(0.0) / Ok(1.0) / Ok(-1.0) ().
    let config = parse_config("[tasks.priority]\nframework = \"ice\"\n").unwrap();
    let inputs: BTreeMap<String, InputValue> = ice_inputs(10.0, 0.9, 10.0)
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    let score = config.tasks.priority.evaluate("ice", &inputs).unwrap();
    assert!((score - 8.0).abs() < 1e-9, "got {score}");
}
