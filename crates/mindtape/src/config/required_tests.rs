#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{Requirement, RequirementRule};
use crate::config::{Config, parse_config};
use crate::model::numeric::NumValue;
use crate::model::{Status, Task};
use crate::test_support::bare_task;
use crate::validate::validate;

/// A config whose `[tasks.priority] required` is the given TOML value text
/// (`true`, `false`, `"!terminal"`, …), over a two-state `wip`/`done` legend.
fn priority_config(required: &str) -> Config {
    parse_config(&format!(
        r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.priority]
kind = "int"
min = 0
max = 9
required = {required}
"#
    ))
    .unwrap()
}

/// A task in `status` carrying no priority — the shape every `required` rule is
/// asked about.
fn task(id: &str, status: &str) -> Task {
    let mut item = bare_task(id);
    item.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    item
}

fn priority_violations(config: &Config, item: Task) -> usize {
    validate(&[item], config)
        .iter()
        .filter(|violation| violation.field == "priority")
        .count()
}

// --- deserialization ---

#[test]
fn booleans_deserialize_to_the_fixed_requirements() {
    assert_eq!(
        priority_config("true").tasks.priority.field.required,
        Requirement::Always
    );
    assert_eq!(
        priority_config("false").tasks.priority.field.required,
        Requirement::No
    );
}

#[test]
fn quoted_booleans_still_deserialize_to_the_fixed_requirements() {
    assert_eq!(
        priority_config("\"true\"").tasks.priority.field.required,
        Requirement::Always
    );
    assert_eq!(
        priority_config("\"false\"").tasks.priority.field.required,
        Requirement::No
    );
}

#[test]
fn any_other_string_deserializes_as_a_filter_expression() {
    assert_eq!(
        priority_config("\"!terminal\"")
            .tasks
            .priority
            .field
            .required,
        Requirement::Expr("!terminal".to_owned())
    );
}

// --- evaluation: the fixed requirements are unchanged ---

#[test]
fn required_true_flags_every_task() {
    let config = priority_config("true");
    assert_eq!(priority_violations(&config, task("a", "wip")), 1);
    assert_eq!(priority_violations(&config, task("a", "done")), 1);
}

#[test]
fn required_false_flags_nothing() {
    let config = priority_config("false");
    assert_eq!(priority_violations(&config, task("a", "wip")), 0);
    assert_eq!(priority_violations(&config, task("a", "done")), 0);
}

// --- evaluation: `!terminal` is the idiom `"non-terminal"` used to spell ---

#[test]
fn not_terminal_flags_a_non_terminal_task() {
    let config = priority_config("\"!terminal\"");
    assert_eq!(priority_violations(&config, task("a", "wip")), 1);
}

#[test]
fn not_terminal_relaxes_a_terminal_task() {
    let config = priority_config("\"!terminal\"");
    assert_eq!(priority_violations(&config, task("a", "done")), 0);
}

#[test]
fn a_set_field_is_never_flagged_missing() {
    let config = priority_config("\"!terminal\"");
    let mut item = task("a", "wip");
    item.priority = Some(NumValue::Num(5.0));
    assert_eq!(priority_violations(&config, item), 0);
}

// --- evaluation: anything else the filter language can say ---

#[test]
fn an_expression_can_require_a_field_for_one_kind_only() {
    let config = priority_config("\"kind == \\\"bug\\\"\"");
    let mut bug = task("a", "wip");
    bug.kind = Some("bug".to_owned());
    let mut chore = task("b", "wip");
    chore.kind = Some("chore".to_owned());
    assert_eq!(priority_violations(&config, bug), 1);
    assert_eq!(priority_violations(&config, chore), 0);
}

#[test]
fn an_expression_governs_difficulty_and_estimate_too() {
    let config = parse_config(
        r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.difficulty]
required = "!terminal"

[tasks.estimate]
required = "!terminal"
"#,
    )
    .unwrap();
    let fields: Vec<String> = validate(&[task("a", "wip")], &config)
        .iter()
        .map(|violation| violation.field.clone())
        .collect();
    assert!(fields.contains(&"difficulty".to_owned()), "{fields:?}");
    assert!(fields.contains(&"estimate".to_owned()), "{fields:?}");

    let mut done = task("b", "done");
    done.estimate = BTreeMap::new();
    assert!(validate(&[done], &config).is_empty());
}

#[test]
fn an_expression_governs_an_enum_field() {
    let config = parse_config(
        r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.fields.kind]
values = ["bug", "chore"]
required = "!terminal"
"#,
    )
    .unwrap();
    assert!(
        validate(&[task("a", "wip")], &config)
            .iter()
            .any(|violation| violation.field == "kind")
    );
    assert!(
        !validate(&[task("b", "done")], &config)
            .iter()
            .any(|violation| violation.field == "kind")
    );
}

// --- load-time validation ---

#[test]
fn a_malformed_expression_fails_config_validation_naming_the_field() {
    let problems = priority_config("\"priority >=\"").validate().unwrap_err();
    assert!(
        problems.contains("tasks.priority.required"),
        "the field must be named: {problems}"
    );
}

#[test]
fn an_unknown_field_in_an_expression_fails_at_load_not_per_task() {
    let problems = priority_config("\"bogus == 1\"").validate().unwrap_err();
    assert!(
        problems.contains("tasks.priority.required") && problems.contains("bogus"),
        "the dry-run must catch an unknown field: {problems}"
    );
}

#[test]
fn the_non_terminal_string_fails_at_load_naming_the_field_not_a_replacement() {
    // `"non-terminal"` carries no special-cased hint any more: it is dry-run
    // as an ordinary filter expression like any other, and whatever that
    // expression fails on is reported under the field's path — same as
    // `an_unknown_field_in_an_expression_fails_at_load_not_per_task` above.
    let problems = priority_config("\"non-terminal\"").validate().unwrap_err();
    assert!(
        problems.contains("tasks.priority.required"),
        "the field must still be named: {problems}"
    );
    assert!(
        !problems.contains("!terminal"),
        "there is no replacement to point at any more: {problems}"
    );
}

#[test]
fn the_non_terminal_string_is_reported_under_every_field_path() {
    let config = parse_config(
        r#"
[tasks.difficulty]
required = "non-terminal"

[tasks.estimate]
required = "non-terminal"

[tasks.fields.kind]
required = "non-terminal"
"#,
    )
    .unwrap();
    let problems = config.validate().unwrap_err();
    for path in [
        "tasks.difficulty.required",
        "tasks.estimate.required",
        "tasks.fields.kind.required",
    ] {
        assert!(problems.contains(path), "{path} must be named: {problems}");
    }
}

#[test]
fn a_usable_expression_passes_config_validation() {
    priority_config("\"!terminal\"").validate().unwrap();
    priority_config("true").validate().unwrap();
}

// --- the compiled rule ---

#[test]
fn may_apply_is_false_only_for_the_never_requirement() {
    assert!(!Requirement::No.may_apply());
    assert!(Requirement::Always.may_apply());
    assert!(Requirement::Expr("!terminal".to_owned()).may_apply());
}

/// Config load rejects an unparsable expression, so this only covers a config
/// built by hand: the rule constrains nothing rather than flagging every task.
#[test]
fn an_unparsable_expression_compiles_to_never_required() {
    let rule = RequirementRule::compile(&Requirement::Expr("(".to_owned()));
    assert!(matches!(rule, RequirementRule::Fixed(false)));
    assert!(!rule.applies(&task("a", "wip"), &priority_config("false")));
}

/// Likewise for an expression that parses but cannot be evaluated.
#[test]
fn an_unevaluatable_expression_reads_as_not_required() {
    let rule = RequirementRule::compile(&Requirement::Expr("bogus == 1".to_owned()));
    assert!(matches!(rule, RequirementRule::Expr(_)));
    assert!(!rule.applies(&task("a", "wip"), &priority_config("false")));
}
