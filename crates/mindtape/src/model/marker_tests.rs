//! The marker projection's shape-error channel: a field, or an element inside
//! one, that does not match the shape it projects into must be REPORTED, never
//! read as unset. Every test here goes through [`Task::from_marker`] rather
//! than the private helpers, because what is being pinned is what an author
//! gets back — the field path they can count to in their own source.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;

use typst_harvest::HVal;

use super::shape_error;
use crate::model::Task;
use crate::model::numeric::NumValue;

fn dict(pairs: Vec<(&str, HVal)>) -> HVal {
    HVal::Dict(
        pairs
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn text(value: &str) -> HVal {
    HVal::Str(value.to_owned())
}

/// Build a task whose only field is `field_name: value`.
fn task_with(field_name: &str, value: HVal) -> Task {
    let marker = dict(vec![
        ("marker", text("mindtape.task")),
        ("fields", dict(vec![(field_name, value)])),
    ]);
    Task::from_marker("x".to_owned(), "x.typ".into(), &marker)
}

/// Build a task from a marker whose only field is `field_name: value`, and
/// assert it reports exactly one [`crate::model::marker::ShapeError`] naming `path`
/// and `expected_message` — shared setup for the wrong-shape tests below, each
/// of which also checks what its field degraded to.
fn assert_reports_shape_error(
    field_name: &str,
    value: HVal,
    path: &str,
    expected_message: &str,
) -> Task {
    let task = task_with(field_name, value);
    assert_eq!(
        task.shape_errors.len(),
        1,
        "expected one report for `{field_name}`, got {:?}",
        task.shape_errors
    );
    assert_eq!(task.shape_errors[0].field, path);
    assert_eq!(task.shape_errors[0].message, expected_message);
    task
}

/// `tags: ("backend")` — a Typst tuple missing its trailing comma, so a plain
/// string rather than a one-element array — must not silently look like no
/// tags authored at all: `from_marker` still coerces `tags` to empty (the
/// field is typed `Vec<String>`), but now names the mismatch in
/// `shape_errors` instead of leaving it unpoliced.
#[test]
fn reports_tags_wrong_shape_instead_of_silent_empty() {
    let task = assert_reports_shape_error(
        "tags",
        text("backend"),
        "tags",
        "expected an array, found a string",
    );
    assert!(task.tags.is_empty());
}

/// `estimate: "2 days"` — a bare string rather than the `(unit: quantity)`
/// dict — must likewise be reported, not silently coerced to an empty map.
#[test]
fn reports_estimate_wrong_shape_instead_of_silent_empty() {
    let task = assert_reports_shape_error(
        "estimate",
        text("2 days"),
        "estimate",
        "expected a dict, found a string",
    );
    assert!(task.estimate.is_empty());
}

/// `links: "a.typ"` — a bare string rather than an array of edges — must
/// likewise be reported, not silently coerced to no edges.
#[test]
fn reports_links_wrong_shape_instead_of_silent_empty() {
    let task = assert_reports_shape_error(
        "links",
        text("a.typ"),
        "links",
        "expected an array, found a string",
    );
    assert!(task.links.is_empty());
}

/// A well-shaped `tags:` array with one wrong-typed ENTRY keeps every entry it
/// can read and names the position of the one it cannot — the case a
/// field-level check passes clean, since the field itself is the right shape.
#[test]
fn reports_a_wrong_typed_tag_by_its_index() {
    let value = HVal::Array(vec![text("a"), HVal::Int(7), text("c")]);
    let task = assert_reports_shape_error(
        "tags",
        value,
        "tags[1]",
        "expected a string, found a number",
    );
    assert_eq!(task.tags, vec!["a".to_owned(), "c".to_owned()]);
}

/// A `none` inside an array is a value the author wrote, not an unset slot:
/// unlike a dict entry, an array has no key to leave out.
#[test]
fn reports_a_none_tag_rather_than_reading_it_as_unset() {
    let value = HVal::Array(vec![text("a"), HVal::Null]);
    assert_reports_shape_error("tags", value, "tags[1]", "expected a string, found none");
}

/// `estimate: (days: "two")` — a well-shaped unit dict whose quantity is not a
/// number. The other units survive; the offending one is named by its unit
/// label, which is the only handle it has.
#[test]
fn reports_a_wrong_typed_estimate_quantity_by_its_unit() {
    let value = dict(vec![("days", text("two")), ("hours", HVal::Int(2))]);
    let task = assert_reports_shape_error(
        "estimate",
        value,
        "estimate.days",
        "expected a number or count(...), found a string",
    );
    assert_eq!(task.estimate.get("hours"), Some(&2.0));
    assert!(!task.estimate.contains_key("days"));
}

/// A `priority:` that is neither a number, a label, nor a framework dict —
/// `(8, 9)` is the shape a hurried author writes for a two-input framework —
/// must not read as an unprioritised task.
#[test]
fn reports_a_priority_that_is_no_numeric_value_at_all() {
    let value = HVal::Array(vec![HVal::Int(8), HVal::Int(9)]);
    let task = assert_reports_shape_error(
        "priority",
        value,
        "priority",
        "expected a number, a label, or a framework value like `ice(...)`, found an array",
    );
    assert!(task.priority.is_none());
}

/// A dict is only a numeric value when it carries a `framework` name; one that
/// does not is reported against the field, since nothing in it is readable.
#[test]
fn reports_a_dict_priority_carrying_no_framework_name() {
    let value = dict(vec![("impact", HVal::Int(8))]);
    assert_reports_shape_error(
        "difficulty",
        value,
        "difficulty",
        "expected a number, a label, or a framework value like `ice(...)`, found a dict",
    );
}

/// A framework's `inputs` is a dict of scalars. A non-scalar input is named by
/// its own key — `priority.inputs.ease` — and the framework survives with the
/// inputs that did read, which is what leaves `check` free to report the
/// framework's own rules on top.
#[test]
fn reports_a_non_scalar_framework_input_by_its_key() {
    let value = dict(vec![
        ("framework", text("ice")),
        (
            "inputs",
            dict(vec![
                ("impact", HVal::Int(8)),
                ("ease", HVal::Array(vec![text("a")])),
            ]),
        ),
    ]);
    let task = assert_reports_shape_error(
        "priority",
        value,
        "priority.inputs.ease",
        "expected a number, a boolean, or a string, found an array",
    );
    let Some(NumValue::Framework { name, inputs }) = task.priority else {
        panic!("expected a framework priority");
    };
    assert_eq!(name, "ice");
    assert_eq!(inputs.keys().collect::<Vec<_>>(), vec!["impact"]);
}

/// `ice(impact: 8)` arrives with `confidence: none` and `ease: none` — the
/// prelude passes its own parameter defaults through. A `none` DICT entry is
/// an unset key, so it must stay silent here; whether the framework requires
/// it is `framework_validate`'s question, not a shape one.
#[test]
fn an_omitted_framework_input_is_not_a_shape_error() {
    let value = dict(vec![
        ("framework", text("ice")),
        (
            "inputs",
            dict(vec![("impact", HVal::Int(8)), ("ease", HVal::Null)]),
        ),
    ]);
    let task = task_with("priority", value);

    assert_eq!(task.shape_errors, Vec::new());
    let Some(NumValue::Framework { inputs, .. }) = task.priority else {
        panic!("expected a framework priority");
    };
    assert_eq!(inputs.keys().collect::<Vec<_>>(), vec!["impact"]);
}

/// A `status:` that is not the prelude's `(name, date, note)` dict at all.
/// `status: "wip"` is the spelling that skips the constructor, and it used to
/// read as no status — leaving `check` with nothing to say but "missing
/// required status", about a field the author had written.
#[test]
fn reports_a_status_that_is_not_a_status_dict() {
    let task = assert_reports_shape_error(
        "status",
        text("wip"),
        "status",
        "expected a status like `wip(2026, 6, 14)`, found a string",
    );
    assert!(task.status.is_none());
}

/// A dict with no `name` is no status either — same report, since the name is
/// the one part nothing downstream can do without.
#[test]
fn reports_a_status_dict_carrying_no_name() {
    let value = dict(vec![("date", text("2026-01-01"))]);
    assert_reports_shape_error(
        "status",
        value,
        "status",
        "expected a status like `wip(2026, 6, 14)`, found a status with no `name`",
    );
}

/// A readable status whose `date`/`note` is wrong-typed keeps the status and
/// names the part — the split that lets `check` report the real mistake
/// instead of "missing required status".
#[test]
fn reports_a_wrong_typed_status_part_and_keeps_the_status() {
    let value = dict(vec![("name", text("wip")), ("date", HVal::Int(42))]);
    let task = assert_reports_shape_error(
        "status",
        value,
        "status.date",
        "expected a string, found a number",
    );
    let status = task.status.unwrap();
    assert_eq!(status.name, "wip");
    assert_eq!(status.date, None);
}

/// Every scalar string slot reports the same way — the class `field_str` used
/// to drop wholesale. `id` is in the list because a wrong-typed one silently
/// falls back to the file stem, which looks exactly like no `id:` authored.
#[test]
fn reports_every_wrong_typed_scalar_string_slot() {
    for field_name in [
        "title", "kind", "assignee", "due", "start", "reviewed", "id",
    ] {
        assert_reports_shape_error(
            field_name,
            HVal::Int(42),
            field_name,
            "expected a string, found a number",
        );
    }
}

/// A date slot authored with the `on(2026, 6, 18)` helper arrives as a Typst
/// `datetime`, which the harvester renders as a `YYYY-MM-DD` string — so the
/// string check must accept it rather than reporting every dated task.
#[test]
fn a_datetime_date_slot_is_not_a_shape_error() {
    let task = task_with("due", HVal::Date("2026-06-18".to_owned()));

    assert_eq!(task.shape_errors, Vec::new());
    assert_eq!(task.due.as_deref(), Some("2026-06-18"));
}

/// `shape_error` names the Typst kind of the offending value in its message,
/// via `hval_kind` — one arm per `HVal` variant. `Null` reaches it only from
/// an array element (the `field` helper filters a `none` FIELD out), so it is
/// exercised directly here.
#[test]
fn shape_error_names_every_hval_kind() {
    let cases = vec![
        (HVal::Null, "none"),
        (HVal::Bool(true), "a boolean"),
        (HVal::Int(1), "a number"),
        (HVal::Float(1.5), "a number"),
        (HVal::Str("x".to_owned()), "a string"),
        (HVal::Date("2026-01-01".to_owned()), "a date"),
        (HVal::Array(vec![]), "an array"),
        (HVal::Dict(BTreeMap::new()), "a dict"),
    ];
    for (value, kind) in cases {
        let err = shape_error("tags", "an array", &value);
        assert_eq!(err.field, "tags");
        assert_eq!(err.message, format!("expected an array, found {kind}"));
    }
}
