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
    // A bare identifier is a task-field mistake whether it's a comparison
    // operand (`resolve_value`'s arm) or lands straight in boolean position
    // with no comparison at all (`eval_bool`'s own arm) — both routes should
    // read alike to a caller.
    let err = Condition::parse("priority > 5").unwrap_err();
    assert!(matches!(err, ConditionError::TaskField(name) if name == "priority"));
    let err = Condition::parse("priority").unwrap_err();
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

#[test]
fn bare_literal_in_boolean_position_is_unsupported() {
    let err = Condition::parse("5").unwrap_err();
    assert!(matches!(err, ConditionError::Unsupported(_)), "{err}");
}

#[test]
fn mixed_type_comparison_falls_back_to_string_ordering() {
    // `weekday()` yields a number; compared against a non-numeric string
    // literal neither side is coerced — both display as text and compare
    // lexically, so a numeric weekday never accidentally equals a word.
    let condition = Condition::parse(r#"weekday() == "monday""#).unwrap();
    assert!(!condition.matches(&env(None, "2026-07-13")).unwrap());
}

#[test]
fn non_value_node_as_comparison_operand_is_invalid() {
    // A parenthesized boolean sub-expression used where a value is expected —
    // `resolve_value` rejects any operator it does not already special-case.
    assert_invalid("(1 && 1) == 1");
}

#[test]
fn weekday_with_malformed_today_is_invalid_at_match_time() {
    // `Condition::parse`'s dry-run always probes with a valid placeholder
    // date, so a malformed `today` can only surface later, at `matches`.
    let condition = Condition::parse("weekday() == 1").unwrap();
    let err = condition.matches(&env(None, "not-a-date")).unwrap_err();
    assert!(
        matches!(&err, ConditionError::Invalid(msg) if msg.contains("malformed date")),
        "{err}"
    );
}

#[test]
fn unknown_function_in_value_position_is_unknown_predicate() {
    // Distinct from `unknown_predicate_is_load_time_error`: that one is an
    // unknown call in boolean position (`predicate`'s arm); this one is an
    // unknown call used as a comparison operand (`resolve_function`'s arm).
    let err = Condition::parse(r"foo() == 1").unwrap_err();
    assert!(matches!(err, ConditionError::UnknownPredicate(name) if name == "foo"));
}

#[test]
fn wrong_arg_count_to_on_branch_is_invalid() {
    assert_invalid(r#"on_branch("a", "b")"#);
}
