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

// --- Boundary regression tests ---
// Each test distinguishes real behaviour from a specific mutation in framework.rs.

#[test]
fn validate_rejects_number_input_with_min_ge_max() {
    // validate_inputs: number inputs need min < max; killing replace body with ()
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\n\
         [tasks.priority.frameworks.x]\nscore = \"v\"\n\
         [tasks.priority.frameworks.x.inputs.v]\ntype = \"number\"\nmin = 5\nmax = 3\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(problems.contains("needs min < max"), "got: {problems}");
}

#[test]
fn validate_rejects_enum_input_without_values() {
    // validate_inputs: enum inputs need non-empty values; killing replace body with ()
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\n\
         [tasks.priority.frameworks.x]\nscore = \"1\"\n\
         [tasks.priority.frameworks.x.inputs.x]\ntype = \"enum\"\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(problems.contains("needs values"), "got: {problems}");
}

#[test]
fn validate_rejects_scalar_below_scale_min() {
    // validate_scalar interval.lo < scope.min distinguishes < → ==.
    // interval.lo = 1, scope.min = 5. Real: 1 < 5 - 1e-9 = true → error.
    // Mutant ==: 1 == 5 - 1e-9 = false → miss.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\nmin = 5\nmax = 100\n\
         [tasks.priority.frameworks.x]\nscore = \"v\"\n\
         [tasks.priority.frameworks.x.inputs.v]\ntype = \"number\"\nmin = 1\nmax = 100\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(problems.contains("exceeds the scale"), "got: {problems}");
}

#[test]
fn validate_accepts_scalar_at_scale_min_boundary() {
    // validate_scalar interval.lo == scope.min - EPS distinguishes < → <=.
    // interval.lo = 9.999999999, scope.min = 10, EPS = 1e-9.
    // Real: 9.999999999 < 10 - 1e-9 = false → passes.
    // Mutant <=: 9.999999999 <= 10 - 1e-9 = true → error.
    let config = parse_config(
        "[tasks.priority]\nframework = \"t\"\nmin = 10\nmax = 100\n\
         [[tasks.priority.bands]]\nlabel = \"all\"\nmin = 10\nmax = 100\n\
         [tasks.priority.frameworks.t]\nscore = \"v\"\n\
         [tasks.priority.frameworks.t.inputs.v]\ntype = \"number\"\nmin = 9.999999999\nmax = 100\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn input_range_strictly_inside_the_scale_validates() {
    // The reachable score interval [10, 50] sits strictly inside the declared
    // scale [5, 100], so validation accepts it. The bound is compared with an
    // epsilon slack (`scope.min - EPS`); an arithmetic slip there turns the
    // slack into a huge number and rejects this perfectly valid config.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\nmin = 5\nmax = 100\n\
         [[tasks.priority.bands]]\nlabel = \"all\"\nmin = 5\nmax = 100\n\
         [tasks.priority.frameworks.x]\nscore = \"v\"\n\
         [tasks.priority.frameworks.x.inputs.v]\ntype = \"number\"\nmin = 10\nmax = 50\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_accepts_scalar_at_scale_max_boundary() {
    // validate_scalar interval.hi == scope.max + EPS distinguishes > → >=.
    // interval.hi = 10.000000001, scope.max = 10, EPS = 1e-9.
    // Real: 10.000000001 > 10 + 1e-9 = false → passes.
    // Mutant >=: 10.000000001 >= 10 + 1e-9 = true → error.
    let config = parse_config(
        "[tasks.priority]\nframework = \"t\"\nmin = 0\nmax = 10\n\
         [[tasks.priority.bands]]\nlabel = \"all\"\nmin = 0\nmax = 10\n\
         [tasks.priority.frameworks.t]\nscore = \"v\"\n\
         [tasks.priority.frameworks.t.inputs.v]\ntype = \"number\"\nmin = 0\nmax = 10.000000001\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_rejects_categorical_score_below_min() {
    // validate_categorical || → &&. score below min.
    // Real: 3 < 5 || 3 > 10 = true || false = true → error.
    // Mutant &&: 3 < 5 && 3 > 10 = true && false = false → miss.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\nmin = 5\nmax = 10\n\
         [tasks.priority.frameworks.x.inputs.flag]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"flag\"\nscore = 3\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"!flag\"\nscore = 8\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(problems.contains("outside the scale"), "got: {problems}");
}

#[test]
fn validate_accepts_categorical_score_at_min() {
    // validate_categorical < → == and < → <= at min boundary.
    // score = 5, min = 5. Real: 5 < 5 || 5 > 10 = false → OK.
    // Mutant ==: 5 == 5 || ... = true → error.
    // Mutant <=: 5 <= 5 || ... = true → error.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\nmin = 5\nmax = 10\n\
         [[tasks.priority.bands]]\nlabel = \"all\"\nmin = 5\nmax = 10\n\
         [tasks.priority.frameworks.x.inputs.flag]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"flag\"\nscore = 5\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"!flag\"\nscore = 8\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_accepts_categorical_strictly_above_min() {
    // validate_categorical < → >. score > min but in bounds.
    // score = 8, min = 5. Real: 8 < 5 || 8 > 10 = false → OK.
    // Mutant >: 8 > 5 || 8 > 10 = true || false = true → error.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\nmin = 5\nmax = 10\n\
         [[tasks.priority.bands]]\nlabel = \"all\"\nmin = 5\nmax = 10\n\
         [tasks.priority.frameworks.x.inputs.flag]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"flag\"\nscore = 8\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"!flag\"\nscore = 8\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_accepts_categorical_score_at_max() {
    // validate_categorical > → == and > → >= at max boundary.
    // score = 10, max = 10. Real: 0 < 10 || 10 > 10 = false → OK.
    // Mutant ==: 0 < 10 || 10 == 10 = false || true = true → error.
    // Mutant >=: 0 < 10 || 10 >= 10 = false || true = true → error.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\nmin = 0\nmax = 10\n\
         [[tasks.priority.bands]]\nlabel = \"all\"\nmin = 0\nmax = 10\n\
         [tasks.priority.frameworks.x.inputs.flag]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"flag\"\nscore = 10\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"!flag\"\nscore = 5\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_accepts_categorical_strictly_below_max() {
    // validate_categorical > → <. score < max but in bounds.
    // score = 3, max = 10. Real: 3 < 0 || 3 > 10 = false || false = false → OK.
    // Mutant <: 3 < 0 || 3 < 10 = false || true = true → error.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\nmin = 0\nmax = 10\n\
         [[tasks.priority.bands]]\nlabel = \"all\"\nmin = 0\nmax = 10\n\
         [tasks.priority.frameworks.x.inputs.flag]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"flag\"\nscore = 3\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"!flag\"\nscore = 5\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_skips_coverage_with_number_input() {
    // check_coverage || → &&. Real: has Number input → skip coverage.
    // Mutant &&: inputs not empty && has Number = false && true = false → doesn't skip → errors.
    let config = parse_config(
        "[tasks.priority]\nframework = \"partial\"\n\
         [tasks.priority.frameworks.partial.inputs.flag]\ntype = \"bool\"\n\
         [tasks.priority.frameworks.partial.inputs.x]\ntype = \"number\"\nmin = 1\nmax = 10\n\
         [[tasks.priority.frameworks.partial.category]]\nwhen = \"flag\"\nscore = 5\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_accepts_exhaustive_categories() {
    // check_coverage != → ==. Exhaustive+disjoint: matched == 1 for all combos.
    // Real: matched != 1 is false → no error.
    // Mutant ==: matched == 1 is true → error.
    let config = parse_config(
        "[tasks.priority]\nframework = \"full\"\n\
         [tasks.priority.frameworks.full.inputs.flag]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.full.category]]\nwhen = \"flag\"\nscore = 5\n\
         [[tasks.priority.frameworks.full.category]]\nwhen = \"!flag\"\nscore = 5\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_accepts_exhaustive_categories_multi_bool() {
    // category_domains → vec![] () and combinations → vec![BTreeMap::new()] ().
    // With 2 bool inputs + 4 exhaustive categories, real checks all 4 combos → matched==1 → OK.
    // Mutant empty domain/combo → only 1 empty combo → predicate eval fails → matched=0 → error.
    let config = parse_config(
        "[tasks.priority]\nframework = \"full\"\n\
         [tasks.priority.frameworks.full.inputs.a]\ntype = \"bool\"\n\
         [tasks.priority.frameworks.full.inputs.b]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.full.category]]\nwhen = \"a && b\"\nscore = 5\n\
         [[tasks.priority.frameworks.full.category]]\nwhen = \"a && !b\"\nscore = 5\n\
         [[tasks.priority.frameworks.full.category]]\nwhen = \"!a && b\"\nscore = 5\n\
         [[tasks.priority.frameworks.full.category]]\nwhen = \"!a && !b\"\nscore = 5\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

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

// Private-fn tests: describe, display_input, combinations are accessible via super::.

#[test]
fn describe_formats_combination() {
    // describe → String::new() or "xyzzy" ().
    let mut combo = BTreeMap::new();
    combo.insert("x".to_owned(), InputValue::Num(42.0));
    combo.insert("y".to_owned(), InputValue::Bool(true));
    let result = super::describe(&combo);
    assert!(result.contains("x=42"), "got: {result}");
    assert!(result.contains("y=true"), "got: {result}");
}

#[test]
fn display_input_formats_variants() {
    // display_input → String::new() or "xyzzy" ().
    assert_eq!(super::display_input(&InputValue::Num(42.0)), "42");
    assert_eq!(super::display_input(&InputValue::Bool(true)), "true");
    assert_eq!(super::display_input(&InputValue::Bool(false)), "false");
    assert_eq!(
        super::display_input(&InputValue::Text("hello".into())),
        "hello"
    );
}

#[test]
fn combinations_produces_cartesian_product() {
    // combinations → vec![BTreeMap::new()] ().
    let domains = vec![(
        "a".to_owned(),
        vec![InputValue::Bool(false), InputValue::Bool(true)],
    )];
    let result = super::combinations(&domains);
    assert_eq!(result.len(), 2);
    assert!(
        result
            .iter()
            .any(|m| m.get("a") == Some(&InputValue::Bool(false)))
    );
    assert!(
        result
            .iter()
            .any(|m| m.get("a") == Some(&InputValue::Bool(true)))
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
