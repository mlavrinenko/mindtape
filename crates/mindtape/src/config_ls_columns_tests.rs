#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{ColumnConfig, ColumnRule, Visibility, validate_ls_columns};
use crate::config_ls::LsConfig;
use crate::parse_config;

/// An `[ls]` config carrying one column with the given rules.
fn ls_with_rules(column: &str, rules: Vec<ColumnRule>) -> LsConfig {
    LsConfig {
        columns: BTreeMap::from([(
            column.to_owned(),
            ColumnConfig {
                rules,
                ..ColumnConfig::default()
            },
        )]),
        ..LsConfig::default()
    }
}

fn rule(condition: &str) -> ColumnRule {
    ColumnRule {
        condition: Some(condition.to_owned()),
        visibility: Visibility::Visible,
    }
}

/// The single problem `problems` must carry — every case below asserts the
/// count as part of the claim, so a second, unrelated message is a failure
/// rather than something the assertion reads past.
fn only(problems: &[String]) -> &str {
    assert_eq!(problems.len(), 1, "{problems:?}");
    problems.first().unwrap()
}

#[test]
fn a_valid_column_rule_condition_reports_nothing() {
    let ls = ls_with_rules("file", vec![rule(r#"!tty() && format() == "table""#)]);
    assert!(validate_ls_columns(&ls).is_empty());
}

#[test]
fn an_unparseable_column_rule_condition_names_the_column_and_index() {
    let ls = ls_with_rules("file", vec![rule("tty()"), rule("ttty()")]);
    let problems = validate_ls_columns(&ls);
    let msg = only(&problems);
    assert!(msg.contains("ls.columns.file.rules[1].condition"), "{msg}");
    assert!(msg.contains("ttty"), "{msg}");
}

/// A task field carried over from filter syntax is the mistake the condition
/// grammar's bare-identifier rejection exists for; it has to fail at load,
/// where it names the column, not at render, where a missing column and a
/// never-matching rule look identical.
#[test]
fn a_task_field_in_a_column_rule_condition_fails_at_load() {
    let ls = ls_with_rules("assignee", vec![rule(r#"status == "wip""#)]);
    let problems = validate_ls_columns(&ls);
    let msg = only(&problems);
    assert!(
        msg.contains("ls.columns.assignee.rules[0].condition"),
        "{msg}"
    );
}

#[test]
fn an_absent_condition_is_valid_and_always_matches_later() {
    let ls = ls_with_rules(
        "file",
        vec![ColumnRule {
            condition: None,
            visibility: Visibility::Hidden,
        }],
    );
    assert!(validate_ls_columns(&ls).is_empty());
}

/// An unknown column name still reports, and reports only once — the rules of
/// a column that does not exist are not also validated, which would bury the
/// real problem under derived noise.
#[test]
fn an_unknown_column_with_rules_reports_only_the_unknown_name() {
    let ls = ls_with_rules("fiel", vec![rule("ttty()")]);
    let problems = validate_ls_columns(&ls);
    let msg = only(&problems);
    assert!(msg.contains("unknown column `fiel`"), "{msg}");
}

#[test]
fn a_column_rule_round_trips_through_toml() {
    let config = parse_config(
        r#"
[[ls.columns.file.rules]]
condition = '!tty()'
visibility = "visible"
"#,
    )
    .unwrap();
    let column = config.ls.columns.get("file").unwrap();
    assert_eq!(column.rules.len(), 1);
    let rule = column.rules.first().unwrap();
    assert_eq!(rule.condition.as_deref(), Some("!tty()"));
    assert_eq!(rule.visibility, Visibility::Visible);
    // The unconditional key keeps its default; a rule does not overwrite it.
    assert_eq!(column.visibility, Visibility::Auto);
}

/// The documented opt-out: an empty list, which a config layer can write over
/// an inherited one.
#[test]
fn an_empty_rules_list_parses_and_carries_no_rule() {
    let config = parse_config("[ls.columns.file]\nrules = []\n").unwrap();
    assert!(config.ls.columns.get("file").unwrap().rules.is_empty());
}
