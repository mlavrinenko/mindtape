#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{Condition, ConditionEnv, ConditionError};

fn env(branch: Option<&str>, today: &str) -> ConditionEnv {
    ConditionEnv {
        branch: branch.map(str::to_owned),
        today: today.to_owned(),
        env: BTreeMap::new(),
    }
}

fn env_with_mode(mode: &str) -> ConditionEnv {
    let mut e = env(None, "2026-07-12");
    e.env.insert("MODE".to_owned(), mode.to_owned());
    e
}

/// Assert that parsing `expr` fails with [`ConditionError::Invalid`].
fn assert_invalid(expr: &str) {
    let err = Condition::parse(expr).unwrap_err();
    assert!(matches!(err, ConditionError::Invalid(_)), "{err}");
}

#[test]
fn empty_condition_always_matches() {
    let condition = Condition::parse("").unwrap();
    assert!(condition.matches(&ConditionEnv::default()).unwrap());
}

#[test]
fn whitespace_only_condition_always_matches() {
    let condition = Condition::parse("   ").unwrap();
    assert!(condition.matches(&ConditionEnv::default()).unwrap());
}

#[test]
fn on_branch_matches_exact_name() {
    let condition = Condition::parse(r#"on_branch("main")"#).unwrap();
    assert!(condition.matches(&env(Some("main"), "2026-07-12")).unwrap());
    assert!(!condition.matches(&env(Some("dev"), "2026-07-12")).unwrap());
}

#[test]
fn on_branch_glob_matches_prefix() {
    let condition = Condition::parse(r#"on_branch("release/*")"#).unwrap();
    assert!(
        condition
            .matches(&env(Some("release/1.0"), "2026-07-12"))
            .unwrap()
    );
    assert!(!condition.matches(&env(Some("main"), "2026-07-12")).unwrap());
}

#[test]
fn on_branch_false_with_no_git() {
    let condition = Condition::parse(r#"on_branch("*")"#).unwrap();
    assert!(!condition.matches(&env(None, "2026-07-12")).unwrap());
}

#[test]
fn env_unset_reads_as_empty_string() {
    let condition = Condition::parse(r#"env("MODE") == """#).unwrap();
    assert!(condition.matches(&env(None, "2026-07-12")).unwrap());
}

#[test]
fn value_may_appear_on_either_side_of_a_comparison() {
    // `env(...)` reading a set variable, checked with the value leading and
    // trailing the comparison — filter.rs only ever allows a field on the
    // left, condition.rs is symmetric.
    let reviewing = env_with_mode("review");
    let exprs = [r#"env("MODE") == "review""#, r#""review" == env("MODE")"#];
    for expr in exprs {
        assert!(Condition::parse(expr).unwrap().matches(&reviewing).unwrap());
    }
}

#[test]
fn weekday_compares_numerically() {
    // 2026-07-12 is a Sunday -> weekday 7.
    let condition = Condition::parse("weekday() == 7").unwrap();
    assert!(condition.matches(&env(None, "2026-07-12")).unwrap());
    let condition = Condition::parse("weekday() >= 6").unwrap();
    assert!(condition.matches(&env(None, "2026-07-12")).unwrap());
    let monday = Condition::parse("weekday() == 1").unwrap();
    assert!(monday.matches(&env(None, "2026-07-13")).unwrap());
}

#[test]
fn now_compares_iso_dates() {
    let condition = Condition::parse(r#"now() >= "2026-07-01""#).unwrap();
    assert!(condition.matches(&env(None, "2026-07-12")).unwrap());
    assert!(!condition.matches(&env(None, "2026-06-01")).unwrap());
}

#[test]
fn and_or_not_combine_predicates() {
    let condition = Condition::parse(r#"on_branch("main") && weekday() <= 5"#).unwrap();
    assert!(!condition.matches(&env(Some("main"), "2026-07-12")).unwrap());
    let condition = Condition::parse(r#"on_branch("main") || weekday() >= 6"#).unwrap();
    assert!(condition.matches(&env(Some("dev"), "2026-07-12")).unwrap());
    let condition = Condition::parse(r#"!on_branch("main")"#).unwrap();
    assert!(condition.matches(&env(Some("dev"), "2026-07-12")).unwrap());
}

#[test]
fn task_field_identifier_is_load_time_error() {
    let err = Condition::parse("priority > 5").unwrap_err();
    assert!(matches!(err, ConditionError::TaskField(name) if name == "priority"));
}

#[test]
fn unknown_predicate_is_load_time_error() {
    let err = Condition::parse(r"ready()").unwrap_err();
    assert!(matches!(err, ConditionError::UnknownPredicate(_)));
}

#[test]
fn bare_value_function_is_not_boolean() {
    assert_invalid(r#"env("MODE")"#);
}

#[test]
fn on_branch_in_comparison_is_invalid() {
    assert_invalid(r#"on_branch("main") == "main""#);
}

#[test]
fn malformed_glob_is_load_time_error() {
    assert_invalid(r#"on_branch("[")"#);
}

#[test]
fn syntax_error_is_parse_error() {
    let err = Condition::parse("on_branch(").unwrap_err();
    assert!(matches!(err, ConditionError::Parse(_)));
}

#[test]
fn weekday_needs_no_arguments() {
    assert_invalid(r#"weekday("mon") == 1"#);
}
