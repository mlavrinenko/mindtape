//! Unit tests for the shared writable-field table, renderer and writer.
#![allow(clippy::unwrap_used)]

use crate::template::RawValue;
use crate::test_support::bare_task;

use super::{
    FieldOp, WRITABLE, apply_op, current_literal, current_raw, get, is_list, names, render_op,
};

fn field(name: &str) -> &'static super::WritableField {
    get(name).unwrap()
}

#[test]
fn the_table_is_the_emitters_field_order() {
    assert_eq!(
        names().collect::<Vec<_>>(),
        [
            "title",
            "priority",
            "difficulty",
            "kind",
            "assignee",
            "waits-on",
            "tags",
            "due",
            "start",
        ]
    );
}

#[test]
fn only_the_two_array_fields_are_lists() {
    assert!(is_list("tags"));
    assert!(is_list("waits-on"));
    assert!(!is_list("title"));
    assert!(!is_list("due"));
    assert!(!is_list("nope"));
}

#[test]
fn a_text_field_renders_a_quoted_escaped_literal() {
    let task = bare_task("subject");
    let op = FieldOp::Set(RawValue::Str("say \"hi\"".to_owned()));
    assert_eq!(
        render_op(field("title"), &op, &task).unwrap(),
        Some("\"say \\\"hi\\\"\"".to_owned())
    );
}

#[test]
fn a_number_renders_bare_and_refuses_a_label_or_infinity() {
    let task = bare_task("subject");
    let eight = FieldOp::Set(RawValue::Str("8".to_owned()));
    assert_eq!(
        render_op(field("priority"), &eight, &task).unwrap(),
        Some("8".to_owned())
    );

    let label = FieldOp::Set(RawValue::Str("high".to_owned()));
    assert!(matches!(
        render_op(field("priority"), &label, &task),
        Err(super::WritableError::NotANumber { .. })
    ));

    let infinity = FieldOp::Set(RawValue::Str("inf".to_owned()));
    assert!(render_op(field("priority"), &infinity, &task).is_err());
}

#[test]
fn a_date_renders_an_on_call_and_refuses_anything_else() {
    let task = bare_task("subject");
    let due = FieldOp::Set(RawValue::Str("2026-07-01".to_owned()));
    assert_eq!(
        render_op(field("due"), &due, &task).unwrap(),
        Some("on(2026, 7, 1)".to_owned())
    );

    let bad = FieldOp::Set(RawValue::Str("next tuesday".to_owned()));
    assert!(matches!(
        render_op(field("due"), &bad, &task),
        Err(super::WritableError::NotADate { .. })
    ));
}

#[test]
fn a_set_on_a_list_is_one_item_and_an_append_keeps_the_rest() {
    let mut task = bare_task("subject");
    task.tags = vec!["backend".to_owned()];

    let replace = FieldOp::Set(RawValue::Str("docs".to_owned()));
    assert_eq!(
        render_op(field("tags"), &replace, &task).unwrap(),
        Some("(\"docs\",)".to_owned())
    );

    let append = FieldOp::Append("auth".to_owned());
    assert_eq!(
        render_op(field("tags"), &append, &task).unwrap(),
        Some("(\"backend\", \"auth\")".to_owned())
    );

    let held = FieldOp::Append("backend".to_owned());
    assert_eq!(render_op(field("tags"), &held, &task).unwrap(), None);
}

#[test]
fn a_whole_array_replaces_a_list_field() {
    let task = bare_task("subject");
    let op = FieldOp::Set(RawValue::List(vec!["a".to_owned(), "b".to_owned()]));
    assert_eq!(
        render_op(field("tags"), &op, &task).unwrap(),
        Some("(\"a\", \"b\")".to_owned())
    );
}

#[test]
fn an_append_to_a_scalar_names_the_field() {
    let task = bare_task("subject");
    let err = render_op(field("priority"), &FieldOp::Append("8".to_owned()), &task).unwrap_err();
    assert!(err.to_string().contains("list"));
    assert!(matches!(
        err,
        super::WritableError::AppendToScalar { field: "priority" }
    ));
}

#[test]
fn apply_sets_and_appends_the_task_fields() {
    let mut task = bare_task("subject");
    let config = crate::Config::default();

    apply_op(
        &mut task,
        field("title"),
        &FieldOp::Set(RawValue::Str("Renamed".to_owned())),
        &config,
    )
    .unwrap();
    apply_op(
        &mut task,
        field("priority"),
        &FieldOp::Set(RawValue::Str("8".to_owned())),
        &config,
    )
    .unwrap();
    apply_op(
        &mut task,
        field("due"),
        &FieldOp::Set(RawValue::Str("2026-12-01".to_owned())),
        &config,
    )
    .unwrap();
    apply_op(
        &mut task,
        field("tags"),
        &FieldOp::Set(RawValue::List(vec!["backend".to_owned()])),
        &config,
    )
    .unwrap();
    apply_op(
        &mut task,
        field("tags"),
        &FieldOp::Append("auth".to_owned()),
        &config,
    )
    .unwrap();
    apply_op(
        &mut task,
        field("tags"),
        &FieldOp::Append("auth".to_owned()),
        &config,
    )
    .unwrap();

    assert_eq!(task.title, "Renamed");
    assert_eq!(
        task.priority,
        Some(crate::model::numeric::NumValue::Num(8.0))
    );
    assert_eq!(task.due.as_deref(), Some("2026-12-01"));
    assert_eq!(task.tags, ["backend", "auth"]);
}

#[test]
fn apply_refuses_a_bad_date_and_a_label_on_a_numeric_field() {
    let mut task = bare_task("subject");
    let config = crate::Config::default();
    assert!(matches!(
        apply_op(
            &mut task,
            field("due"),
            &FieldOp::Set(RawValue::Str("soon".to_owned())),
            &config,
        ),
        Err(super::WritableError::NotADate { .. })
    ));
    assert!(matches!(
        apply_op(
            &mut task,
            field("priority"),
            &FieldOp::Set(RawValue::Str("high".to_owned())),
            &config,
        ),
        Err(super::WritableError::NotANumber { .. })
    ));
}

#[test]
fn current_raw_and_current_literal_read_what_the_task_holds() {
    let mut task = bare_task("subject");
    task.kind = Some("feat".to_owned());
    task.due = Some("2026-03-04".to_owned());
    task.priority = Some(crate::model::numeric::NumValue::Label("high".to_owned()));
    task.tags = vec!["a".to_owned(), "b".to_owned()];

    assert_eq!(
        current_raw(field("kind"), &task),
        Some(RawValue::Str("feat".to_owned()))
    );
    assert_eq!(
        current_raw(field("due"), &task),
        Some(RawValue::Str("2026-03-04".to_owned()))
    );
    assert_eq!(
        current_raw(field("priority"), &task),
        Some(RawValue::Str("high".to_owned()))
    );
    assert_eq!(
        current_raw(field("tags"), &task),
        Some(RawValue::List(vec!["a".to_owned(), "b".to_owned()]))
    );
    assert_eq!(current_raw(field("assignee"), &task), None);

    assert_eq!(
        current_literal(field("kind"), &task).as_deref(),
        Some("\"feat\"")
    );
    assert_eq!(
        current_literal(field("due"), &task).as_deref(),
        Some("on(2026, 3, 4)")
    );
    assert_eq!(
        current_literal(field("priority"), &task).as_deref(),
        Some("\"high\"")
    );
    assert_eq!(
        current_literal(field("tags"), &task).as_deref(),
        Some("(\"a\", \"b\")")
    );
    assert_eq!(current_literal(field("assignee"), &task), None);

    // A number reads as its text form for a slot's kind coercion, `3` rather
    // than a float an `int` slot refuses.
    task.priority = Some(crate::model::numeric::NumValue::Num(3.0));
    assert_eq!(
        current_raw(field("priority"), &task),
        Some(RawValue::Str("3".to_owned()))
    );
}

#[test]
fn dir_is_refused_with_the_rename_or_dir_hint() {
    assert!(get("dir").is_none());
    let hint = super::elsewhere("dir").unwrap();
    assert!(hint.contains("mt mv"));
    assert!(hint.contains("--dir"));
}

#[test]
fn the_table_has_no_duplicate_names() {
    let mut seen: Vec<&str> = names().collect();
    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), total);
    assert_eq!(seen.len(), WRITABLE.len());
}
