#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use crate::config::parse_config;
use crate::model::numeric::InputValue;

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
fn validate_rejects_framework_with_both_score_and_categories() {
    // validate: the (Some(_), false) match arm — score and categories are
    // mutually exclusive shapes.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\n\
         [tasks.priority.frameworks.x]\nscore = \"1\"\n\
         [tasks.priority.frameworks.x.inputs.flag]\ntype = \"bool\"\n\
         [[tasks.priority.frameworks.x.category]]\nwhen = \"flag\"\nscore = 5\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(
        problems.contains("declares both a score and categories"),
        "got: {problems}"
    );
}

#[test]
fn validate_rejects_framework_with_neither_score_nor_categories() {
    // validate: the (None, true) match arm — an input-only framework cannot
    // score at all.
    let config = parse_config(
        "[tasks.priority]\nframework = \"x\"\n\
         [tasks.priority.frameworks.x.inputs.v]\ntype = \"number\"\nmin = 1\nmax = 10\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(
        problems.contains("declares neither a score nor categories"),
        "got: {problems}"
    );
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

// --- Boundary regression tests ---
// Each test distinguishes real behaviour from a specific mutation in
// framework/validate.rs.

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

// --- Input names must be Typst identifiers ---
// Input names splice into generated `framework("f", input: …)` calls and serve
// as `--set` keys; both paths need a name a named argument can carry.

#[test]
fn validate_rejects_input_name_with_space() {
    let config = parse_config(
        "[tasks.priority]\nframework = \"ice\"\n\
         [tasks.priority.frameworks.ice.inputs.\"has space\"]\ntype = \"number\"\nmin = 1\nmax = 10\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(
        problems.contains("input `has space` is not a valid typst identifier"),
        "got: {problems}"
    );
}

#[test]
fn validate_rejects_keyword_and_underscore_input_names() {
    // `none` lexes as a keyword and `_` as the underscore token; neither can
    // be a named argument. Pins is_typst_ident's two exclusions.
    let config = parse_config(
        "[tasks.priority]\nframework = \"k\"\nmin = 0\nmax = 9\n\
         [tasks.priority.frameworks.k]\nscore = \"1\"\n\
         [tasks.priority.frameworks.k.inputs.none]\ntype = \"number\"\nmin = 1\nmax = 10\n\
         [tasks.priority.frameworks.k.inputs._]\ntype = \"number\"\nmin = 1\nmax = 10\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(problems.contains("input `none`"), "got: {problems}");
    assert!(problems.contains("input `_`"), "got: {problems}");
}

#[test]
fn validate_accepts_hyphen_and_unicode_input_names() {
    // `my-input` (bare TOML key) and `café` (quoted key) both satisfy the
    // identifier rules; only spaces or a leading digit would fail them.
    let config = parse_config(
        "[tasks.priority]\nframework = \"g\"\nmin = 0\nmax = 9\n\
         [tasks.priority.frameworks.g]\nscore = \"1\"\n\
         [tasks.priority.frameworks.g.inputs.my-input]\ntype = \"number\"\nmin = 1\nmax = 10\n\
         [tasks.priority.frameworks.g.inputs.\"café\"]\ntype = \"number\"\nmin = 1\nmax = 10\n",
    )
    .unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn validate_checks_inactive_framework_input_names() {
    // Every declared framework is checked, not just the active one: a bad
    // input name in an inactive framework must still fail validation.
    let config = parse_config(
        "[tasks.priority]\nframework = \"ice\"\n\
         [tasks.priority.frameworks.f]\nscore = \"1\"\n\
         [tasks.priority.frameworks.f.inputs.\"bad name\"]\ntype = \"number\"\nmin = 1\nmax = 10\n",
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    assert!(
        problems.contains("framework `f` input `bad name`"),
        "got: {problems}"
    );
}

#[test]
fn is_typst_ident_accepts_plain_names() {
    assert!(super::is_typst_ident("impact"));
    assert!(super::is_typst_ident("cost_of_delay"));
    assert!(super::is_typst_ident("my-input"));
    assert!(super::is_typst_ident("_hidden"));
    assert!(super::is_typst_ident("café"));
}

#[test]
fn is_typst_ident_rejects_keywords_underscore_and_non_idents() {
    assert!(!super::is_typst_ident("none"));
    assert!(!super::is_typst_ident("let"));
    assert!(!super::is_typst_ident("_"));
    assert!(!super::is_typst_ident("has space"));
    assert!(!super::is_typst_ident("9lives"));
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
