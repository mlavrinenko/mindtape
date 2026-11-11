//! `old("field")`, the pre-hook draft an `[[on.add]]` `when` reads.
#![allow(clippy::unwrap_used)]

use crate::filter::Filter;
use crate::model::Task;
use crate::parse_config;

fn titled(title: &str) -> Task {
    Task {
        id: "add-auth".to_owned(),
        title: title.to_owned(),
        ..Task::default()
    }
}

#[test]
fn old_reads_the_draft_before_the_hooks() {
    let config = parse_config("").unwrap();
    let mut current = titled("Fix the login");
    current.kind = Some("bug".to_owned());
    let mut before = current.clone();
    before.kind = Some("feat".to_owned());
    let filter = Filter::parse(r#"old("kind") == "feat""#).unwrap();
    assert!(
        filter
            .matches_add(&current, &before, &config, &crate::Readiness::empty())
            .unwrap()
    );
    assert!(
        !filter
            .matches_add(&current, &current, &config, &crate::Readiness::empty())
            .unwrap()
    );
}

#[test]
fn old_is_refused_outside_an_add_when() {
    let config = parse_config("").unwrap();
    let task = titled("Fix the login");
    let err = Filter::parse(r#"old("kind") == "feat""#)
        .unwrap()
        .matches(&task, &config)
        .unwrap_err();
    assert!(err.to_string().contains("on.add"), "{err}");
}

#[test]
fn old_reads_text_in_a_string_position() {
    let config = parse_config("").unwrap();
    let current = titled("Fix the login");
    let before = titled("Add the login");
    let filter = Filter::parse(r#"matches(old("title"), "^Add")"#).unwrap();
    assert!(
        filter
            .matches_add(&current, &before, &config, &crate::Readiness::empty())
            .unwrap()
    );
}

#[test]
fn old_numeric_fields_compare_as_numbers() {
    use crate::model::numeric::NumValue;

    let config = parse_config("").unwrap();
    let mut current = titled("Fix the login");
    current.priority = Some(NumValue::Num(10.0));
    let before = current.clone();
    let filter = Filter::parse(r#"old("priority") < 9"#).unwrap();
    assert!(
        !filter
            .matches_add(&current, &before, &config, &crate::Readiness::empty())
            .unwrap()
    );
}

#[test]
fn old_refuses_the_drafts_identity_and_place() {
    let config = parse_config("").unwrap();
    let task = titled("Fix the login");
    for field in ["id", "path", "dir"] {
        let filter = Filter::parse(&format!(r#"old("{field}") != "x""#)).unwrap();
        let err = filter
            .matches_add(&task, &task, &config, &crate::Readiness::empty())
            .unwrap_err();
        assert!(err.to_string().contains(field), "{field}: {err}");
    }
}

#[test]
fn old_status_is_checked_against_the_legend() {
    let legend =
        parse_config("[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\n")
            .unwrap()
            .tasks
            .status;

    for expr in [r#"status == "ghost""#, r#"old("status") == "ghost""#] {
        let err = Filter::parse(expr)
            .unwrap()
            .check_statuses(&legend)
            .unwrap_err();
        assert!(
            matches!(err, crate::filter::FilterError::UnknownStatus { .. }),
            "{expr}: {err:?}"
        );
    }
    assert!(
        Filter::parse(r#"old("status") == "wip""#)
            .unwrap()
            .check_statuses(&legend)
            .is_ok()
    );
}

#[test]
fn old_on_an_unset_field_is_missing() {
    let config = parse_config("").unwrap();
    let task = titled("Fix the login");
    let filter = Filter::parse(r#"old("assignee") != "ada""#).unwrap();
    assert!(
        filter
            .matches_add(&task, &task, &config, &crate::Readiness::empty())
            .unwrap()
    );
}
