//! The pure helpers of add-hook execution: derive field conversion, `dir`
//! expansion and the error wordings, without evaluating a Typst file.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::path::Path;

use mindtape::config::add_hooks::DeriveAction;
use mindtape::model::writable::{self, Shape};
use mindtape::{Config, Task};
use typst_harvest::HVal;

use super::{
    Derivation, Outcome, Run, expand, field, field_derive, field_error, field_value, fires,
    macro_value,
};

fn run<'a>(config: &'a Config, explicit: &'a BTreeSet<String>) -> Run<'a> {
    Run {
        root: Path::new("."),
        template: "task",
        explicit_set: explicit,
        explicit_dir: false,
        config,
        label: "fixes",
    }
}

#[test]
fn a_derive_writes_nothing_without_a_field_or_past_an_explicit_set() {
    for (field, explicit) in [
        (None, BTreeSet::new()),
        (Some("kind".to_owned()), BTreeSet::from(["kind".to_owned()])),
    ] {
        let config = Config::default();
        let run = run(&config, &explicit);
        let mut task = Task::default();
        let mut outcome = Outcome::default();
        let derive = DeriveAction {
            typst: "add.typ".to_owned(),
            function: "classify".to_owned(),
            field,
        };
        field_derive(&derive, &run, &mut task, &mut outcome).unwrap();
        assert!(outcome.touched.is_empty());
        assert_eq!(task.kind, None);
    }
}

#[test]
fn a_derived_value_must_fit_its_field() {
    let list = writable::get("tags").unwrap();
    assert_eq!(
        field_value(list, &HVal::Array(vec![HVal::Str("a".to_owned())])),
        Some(mindtape::RawValue::List(vec!["a".to_owned()]))
    );
    assert!(matches!(list.shape, Shape::List(_)));
    assert_eq!(field_value(list, &HVal::Int(1)), None);

    let kind = writable::get("kind").unwrap();
    for (value, expected) in [
        (
            HVal::Str("bug".to_owned()),
            Some(mindtape::RawValue::Str("bug".to_owned())),
        ),
        (
            HVal::Date("2026-01-01".to_owned()),
            Some(mindtape::RawValue::Str("2026-01-01".to_owned())),
        ),
        (HVal::Int(1), Some(mindtape::RawValue::Int(1))),
        (HVal::Float(1.5), Some(mindtape::RawValue::Float(1.5))),
        (HVal::Bool(true), Some(mindtape::RawValue::Bool(true))),
        (HVal::Null, None),
    ] {
        assert_eq!(field_value(kind, &value), expected);
    }
}

#[test]
fn dir_macros_read_the_template_and_scalars() {
    let task = Task {
        kind: Some("bug".to_owned()),
        ..Task::default()
    };
    assert_eq!(
        macro_value("template", "task", &task).as_deref(),
        Some("task")
    );
    assert_eq!(macro_value("kind", "task", &task).as_deref(), Some("bug"));
    assert_eq!(macro_value("assignee", "task", &task), None);
    assert_eq!(macro_value("nope", "task", &task), None);
}

#[test]
fn an_unclosed_dir_macro_is_refused() {
    let task = Task::default();
    let err = expand("tasks/{kind", "task", &task, "fixes").unwrap_err();
    assert!(crate::diag::plain(&err).contains("fixes"), "{err}");
}

#[test]
fn an_unknown_field_is_refused_by_name() {
    let err = field("nope", "fixes").unwrap_err();
    assert!(crate::diag::plain(&err).contains("nope"), "{err}");
}

fn assert_hook_error(diag: &crate::diag::Diag, needle: &str) {
    let text = crate::diag::plain(diag);
    assert!(text.starts_with("error: on.add fixes: "), "{diag}");
    assert!(text.contains(needle), "{diag}");
}

#[test]
fn the_error_wordings_name_the_field_and_hook() {
    let call = Derivation::of("add.typ", "classify");
    assert_hook_error(&call.type_error("tags", "fixes"), "tags");

    let write = field_error(
        writable::WritableError::NotANumber {
            field: "priority",
            value: "high".to_owned(),
        },
        "priority",
        "fixes",
    );
    assert_hook_error(&write, "priority");
    assert_hook_error(&call.fail("boom", "fixes"), "boom");
}

#[test]
fn a_malformed_when_is_refused_by_the_hook_label() {
    let config = Config::default();
    let explicit = BTreeSet::new();
    let run = run(&config, &explicit);
    let task = Task::default();
    let hook = mindtape::config::add_hooks::AddHook {
        name: Some("fixes".to_owned()),
        when: "template".to_owned(),
        actions: Vec::new(),
    };
    let err = fires(&hook, &task, &task, &run).unwrap_err();
    assert!(crate::diag::plain(&err).contains("fixes"), "{err}");
}

#[test]
fn a_dir_macro_reads_text_and_refuses_a_list() {
    let task = Task {
        kind: Some("bug".to_owned()),
        tags: vec!["backend".to_owned()],
        ..Task::default()
    };
    assert_eq!(
        macro_value("template", "fix", &task).as_deref(),
        Some("fix")
    );
    assert_eq!(macro_value("kind", "fix", &task).as_deref(), Some("bug"));
    assert_eq!(macro_value("tags", "fix", &task), None);
}

#[test]
fn a_derived_write_error_names_the_hook() {
    let call = Derivation::of("add.typ", "classify");
    let diag = call.field_error(
        writable::WritableError::ManyValues { field: "tags" },
        "tags",
        "fixes",
    );
    assert_hook_error(&diag, "tags");
}
