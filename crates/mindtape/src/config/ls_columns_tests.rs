#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::{ColumnConfig, validate_ls_columns};
use crate::config::ls::LsConfig;
use crate::parse_config;

/// An `[ls]` config carrying one column with the given `when`.
fn ls_with_when(column: &str, when: &str) -> LsConfig {
    LsConfig {
        columns: BTreeMap::from([(
            column.to_owned(),
            ColumnConfig {
                when: Some(when.to_owned()),
            },
        )]),
        ..LsConfig::default()
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
fn a_valid_when_reports_nothing() {
    let ls = ls_with_when("file", r#"named() || (!tty() && format() == "table")"#);
    assert!(validate_ls_columns(&ls).is_empty());
}

#[test]
fn an_unparseable_when_names_the_column() {
    let ls = ls_with_when("file", "ttty()");
    let problems = validate_ls_columns(&ls);
    let msg = only(&problems);
    assert!(msg.contains("ls.columns.file.when"), "{msg}");
    assert!(msg.contains("ttty"), "{msg}");
}

/// A task field carried over from filter syntax is the mistake the condition
/// grammar's bare-identifier rejection exists for; it has to fail at load,
/// where it names the column, not at render, where a missing column and a
/// never-true `when` look identical.
#[test]
fn a_task_field_in_a_when_fails_at_load() {
    let ls = ls_with_when("assignee", r#"status == "wip""#);
    let problems = validate_ls_columns(&ls);
    let msg = only(&problems);
    assert!(msg.contains("ls.columns.assignee.when"), "{msg}");
}

/// `Condition` reads an empty expression as the always-matching one, so a
/// blank `when` would render the column always while an ABSENT `when` renders
/// it never — two spellings of "nothing written here" with opposite answers.
/// It is refused instead, and the message says what to write.
#[test]
fn a_blank_when_is_refused_rather_than_read_as_always() {
    for blank in ["", "   "] {
        let ls = ls_with_when("due", blank);
        let problems = validate_ls_columns(&ls);
        let msg = only(&problems);
        assert!(msg.contains("ls.columns.due.when"), "{msg}");
        assert!(msg.contains("empty expression"), "{msg}");
    }
}

/// A column with no `when` at all is legal config — it is how a project drops
/// a column the floor ships — so it must not report.
#[test]
fn an_absent_when_reports_nothing() {
    let ls = LsConfig {
        columns: BTreeMap::from([("file".to_owned(), ColumnConfig::default())]),
        ..LsConfig::default()
    };
    assert!(validate_ls_columns(&ls).is_empty());
}

/// An unknown column name still reports, and reports only once — the `when` of
/// a column that does not exist is not also validated, which would bury the
/// real problem under derived noise.
#[test]
fn an_unknown_column_with_a_when_reports_only_the_unknown_name() {
    let ls = ls_with_when("fiel", "ttty()");
    let problems = validate_ls_columns(&ls);
    let msg = only(&problems);
    assert!(msg.contains("unknown column `fiel`"), "{msg}");
}

#[test]
fn a_when_round_trips_through_toml() {
    let config = parse_config("[ls.columns.file]\nwhen = '!tty()'\n").unwrap();
    let column = config.ls.columns.get("file").unwrap();
    assert_eq!(column.when.as_deref(), Some("!tty()"));
}

/// A scalar replaces wholesale across config layers, so a project turns a
/// floored column off with one line and no opt-out convention.
#[test]
fn a_project_when_replaces_the_floor_s() {
    let config = parse_config("[ls.columns.file]\nwhen = 'false'\n").unwrap();
    let file = config.ls.columns.get("file").unwrap();
    assert_eq!(file.when.as_deref(), Some("false"));
}

/// The floor is the complete statement of what `mt ls` shows: every column in
/// the vocabulary carries a `when` there, so no column is left to a built-in
/// default the render layer would have to hold.
///
/// Read through [`crate::test_support::floor_config`], the fixture the CLI's
/// render tests are built on, so the two cannot come to mean different things —
/// and asserted against [`Config::default`] in the same breath, because that is
/// the whole distinction the helper exists to draw: an unfloored config
/// describes no column, and so renders none.
#[test]
fn the_floor_gives_every_column_a_when() {
    let config = crate::test_support::floor_config();
    for column in crate::config::ls::LS_COLUMNS {
        let entry = config
            .ls
            .columns
            .get(column)
            .unwrap_or_else(|| panic!("the floor must describe `{column}`"));
        assert!(
            entry.when.is_some(),
            "the floor's `{column}` must carry a when"
        );
    }
    assert!(config.validate().is_ok());
    assert!(
        crate::Config::default().ls.columns.is_empty(),
        "Config::default() bypasses the floor, so it describes no column"
    );
}
