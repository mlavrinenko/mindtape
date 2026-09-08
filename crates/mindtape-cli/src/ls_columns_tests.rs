#![allow(clippy::unwrap_used)]

use mindtape::{ColumnConfig, ColumnRule, ConditionEnv, Visibility, parse_config};

use super::apply_column_rules;

/// An environment with the two facts the shipped floor rule reads.
fn env(tty: bool, format: &str) -> ConditionEnv {
    ConditionEnv {
        tty,
        format: format.to_owned(),
        ..ConditionEnv::default()
    }
}

/// The visibility `file` resolves to under one environment, reading the real
/// embedded floor rather than a fixture — the shipped rule is the thing under
/// test, and a fixture restating it would pass while the floor said anything.
fn file_visibility(tty: bool, format: &str) -> Visibility {
    let mut config = parse_config("").unwrap();
    apply_column_rules(&mut config, &env(tty, format)).unwrap();
    config.ls.columns.get("file").unwrap().visibility
}

#[test]
fn the_floor_shows_file_in_a_captured_table() {
    assert_eq!(file_visibility(false, "table"), Visibility::Visible);
}

#[test]
fn the_floor_leaves_file_opt_in_on_a_terminal() {
    assert_eq!(file_visibility(true, "table"), Visibility::Auto);
}

/// CSV resolves the same column set as the table, so the floor rule has to
/// exclude it by name or a script parsing `mt ls --format csv` silently gains
/// a column.
#[test]
fn the_floor_leaves_csv_alone_even_when_captured() {
    assert_eq!(file_visibility(false, "csv"), Visibility::Auto);
}

/// A rule beats the column's own `visibility` key: rules ARE the override
/// mechanism, so the other direction would make them unwritable.
#[test]
fn a_matching_rule_beats_the_unconditional_visibility() {
    let mut config = parse_config(
        r#"
[ls.columns.status]
visibility = "hidden"

[[ls.columns.status.rules]]
condition = '!tty()'
visibility = "visible"
"#,
    )
    .unwrap();
    apply_column_rules(&mut config, &env(false, "table")).unwrap();
    assert_eq!(
        config.ls.columns.get("status").unwrap().visibility,
        Visibility::Visible
    );
}

/// ... and when no rule matches, the unconditional key is what is left.
#[test]
fn a_non_matching_rule_leaves_the_unconditional_visibility() {
    let mut config = parse_config(
        r#"
[ls.columns.status]
visibility = "hidden"

[[ls.columns.status.rules]]
condition = '!tty()'
visibility = "visible"
"#,
    )
    .unwrap();
    apply_column_rules(&mut config, &env(true, "table")).unwrap();
    assert_eq!(
        config.ls.columns.get("status").unwrap().visibility,
        Visibility::Hidden
    );
}

/// First match wins, mirroring `[[ls.rules]]`.
#[test]
fn the_first_matching_rule_wins() {
    let mut config = parse_config("").unwrap();
    config.ls.columns.insert(
        "due".to_owned(),
        ColumnConfig {
            visibility: Visibility::Auto,
            rules: vec![
                ColumnRule {
                    condition: Some("!tty()".to_owned()),
                    visibility: Visibility::Hidden,
                },
                ColumnRule {
                    condition: Some("!tty()".to_owned()),
                    visibility: Visibility::Visible,
                },
            ],
        },
    );
    apply_column_rules(&mut config, &env(false, "table")).unwrap();
    assert_eq!(
        config.ls.columns.get("due").unwrap().visibility,
        Visibility::Hidden
    );
}

/// The documented opt-out: an array replaces rather than merges across config
/// layers, so an empty list drops the inherited floor rule outright.
#[test]
fn an_empty_rules_list_opts_out_of_the_floor_rule() {
    let mut config = parse_config("[ls.columns.file]\nrules = []\n").unwrap();
    apply_column_rules(&mut config, &env(false, "table")).unwrap();
    assert_eq!(
        config.ls.columns.get("file").unwrap().visibility,
        Visibility::Auto
    );
}

/// Config load validates every condition, so reaching this needs a config
/// built in memory — but it must still report rather than silently not match.
#[test]
fn an_unevaluable_condition_names_the_column_and_index() {
    let mut config = parse_config("").unwrap();
    config.ls.columns.insert(
        "title".to_owned(),
        ColumnConfig {
            visibility: Visibility::Auto,
            rules: vec![ColumnRule {
                condition: Some("ttty()".to_owned()),
                visibility: Visibility::Hidden,
            }],
        },
    );
    let err = apply_column_rules(&mut config, &env(false, "table")).unwrap_err();
    assert!(
        format!("{err:#}").contains("ls.columns.title.rules[0].condition"),
        "{err:#}"
    );
}
