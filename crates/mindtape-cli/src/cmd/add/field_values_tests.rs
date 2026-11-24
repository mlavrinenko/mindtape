//! Unit tests for the non-slot field split. The write itself belongs to
//! `tests/it/add_fields_e2e.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use mindtape::{FieldOp, RawValue};

use super::{touched, upsert};
use crate::cmd::add::support_tests::specs_of;
use crate::cmd::add::values::Collected;

fn collected(ops: &[(&str, FieldOp)]) -> Collected {
    Collected {
        ops: ops
            .iter()
            .map(|(name, op)| ((*name).to_owned(), op.clone()))
            .collect(),
    }
}

#[test]
fn a_field_with_no_slot_is_touched_in_operation_order() {
    let specs = specs_of(r#"#f(title: slot("title"))"#);
    let values = collected(&[
        ("tags", FieldOp::Append("b".to_owned())),
        ("due", FieldOp::Set(RawValue::Str("2026-01-01".to_owned()))),
        ("tags", FieldOp::Set(RawValue::List(vec!["a".to_owned()]))),
    ]);
    let fields = touched(&values, &specs);
    let names: Vec<&str> = fields.iter().map(|(field, _)| field.name).collect();
    assert_eq!(names, ["tags", "due"]);
    assert_eq!(fields[0].1.len(), 2);
    assert!(matches!(fields[0].1[0], FieldOp::Append(_)));
    assert_eq!(fields[1].1.len(), 1);
}

#[test]
fn a_field_the_template_exposes_as_a_slot_is_not_touched() {
    let specs = specs_of(r#"#f(tags: slot("tags", kind: "list"))"#);
    let values = collected(&[("tags", FieldOp::Append("a".to_owned()))]);
    assert!(touched(&values, &specs).is_empty());
}

#[test]
fn upsert_skips_a_field_the_task_does_not_hold() {
    let source = mindtape::typst_task!(r#"(title: "Do thing")"#);
    let task = mindtape::Task::default();
    assert_eq!(
        upsert(
            source,
            &[mindtape::model::writable::get("kind").unwrap()],
            &task
        )
        .unwrap(),
        source
    );
}
