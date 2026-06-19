#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use crate::config::parse_config;
use crate::numeric::{InputValue, NumValue};

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
fn validate_rejects_framework_exceeding_scale() {
    let config = parse_config(
        "[tasks.priority]\nframework = \"big\"\nmin = 0\nmax = 9\n\
         [tasks.priority.frameworks.big]\nscore = \"impact * 100\"\n\
         [tasks.priority.frameworks.big.inputs.impact]\ntype = \"number\"\nmin = 1\nmax = 10\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(problems.contains("exceeds the scale"), "got: {problems}");
}

#[test]
fn validate_rejects_non_exhaustive_categories() {
    let config = parse_config(
        "[tasks.priority]\nframework = \"partial\"\n\
         [tasks.priority.frameworks.partial.inputs.flag]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.partial.category]]\nwhen = \"flag\"\nscore = 9\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(problems.contains("exhaustive+disjoint"), "got: {problems}");
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
