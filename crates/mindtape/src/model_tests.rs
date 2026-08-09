#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;
use std::path::Path;

use typst_harvest::HVal;

use super::{Status, Task, is_history_pointer, normalize_link, resolve_link_path, shape_error};
use crate::numeric::{InputValue, NumValue};
use crate::test_support::bare_task;

fn dict(pairs: Vec<(&str, HVal)>) -> HVal {
    HVal::Dict(
        pairs
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

#[test]
fn from_marker_reads_core_fields() {
    let status = dict(vec![
        ("name", HVal::Str("wip".to_owned())),
        ("date", HVal::Date("2026-06-14".to_owned())),
        ("note", HVal::Null),
    ]);
    let fields = dict(vec![
        ("title", HVal::Str("Build".to_owned())),
        ("status", status),
        ("priority", HVal::Int(8)),
        ("difficulty", HVal::Str("senior".to_owned())),
        (
            "estimate",
            dict(vec![("days", HVal::Int(5)), ("hours", HVal::Int(5))]),
        ),
        (
            "links",
            HVal::Array(vec![dict(vec![
                ("task", HVal::Str("other.task-260101.typ".to_owned())),
                ("kind", HVal::Str("related".to_owned())),
                ("note", HVal::Str("carried over".to_owned())),
            ])]),
        ),
        ("tags", HVal::Array(vec![HVal::Str("backend".to_owned())])),
    ]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
        (
            "extras",
            dict(vec![("x-speconaut", HVal::Str("opaque".to_owned()))]),
        ),
    ]);

    let task = Task::from_marker("build".to_owned(), "tasks/build.typ".into(), &marker);
    assert_eq!(task.id, "build");
    assert_eq!(task.title, "Build");
    assert_eq!(task.status.as_ref().unwrap().name, "wip");
    assert_eq!(
        task.status.as_ref().unwrap().date.as_deref(),
        Some("2026-06-14")
    );
    assert_eq!(task.priority, Some(NumValue::Num(8.0)));
    assert_eq!(task.difficulty, Some(NumValue::Label("senior".to_owned())));
    assert_eq!(task.estimate.get("days"), Some(&5.0));
    assert_eq!(task.estimate.get("hours"), Some(&5.0));
    assert_eq!(
        task.link_targets("related").collect::<Vec<_>>(),
        vec!["other.task-260101.typ"]
    );
    assert_eq!(task.links[0].note.as_deref(), Some("carried over"));
    assert_eq!(task.tags, vec!["backend".to_owned()]);
    assert!(task.extras.contains_key("x-speconaut"));
}

#[test]
fn from_marker_reads_assignee_and_blank_is_absent() {
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        (
            "fields",
            dict(vec![("assignee", HVal::Str("ada".to_owned()))]),
        ),
    ]);
    let task = Task::from_marker("x".to_owned(), "x.typ".into(), &marker);
    assert_eq!(task.assignee.as_deref(), Some("ada"));

    let blank = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", dict(vec![("assignee", HVal::Str(String::new()))])),
    ]);
    let task = Task::from_marker("y".to_owned(), "y.typ".into(), &blank);
    assert!(task.assignee.is_none());
}

#[test]
fn from_marker_reads_recur_field() {
    let rule = dict(vec![
        ("kind", HVal::Str("period".to_owned())),
        ("unit", HVal::Str("weeks".to_owned())),
        ("value", HVal::Int(1)),
        ("on", HVal::Null),
    ]);
    let recur = dict(vec![
        ("rule", rule),
        ("anchor", HVal::Null),
        ("until", HVal::Null),
        ("count", HVal::Null),
        ("materialize", HVal::Null),
    ]);
    let fields = dict(vec![("recur", recur)]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
    ]);
    let task = Task::from_marker("chore".to_owned(), "chore.typ".into(), &marker);
    assert!(task.recur.is_some());
}

#[test]
fn from_marker_tolerates_missing_fields() {
    let marker = dict(vec![("marker", HVal::Str("mindtape.task".to_owned()))]);
    let task = Task::from_marker("x".to_owned(), "x.typ".into(), &marker);
    assert_eq!(task.title, "");
    assert!(task.status.is_none());
    assert!(task.tags.is_empty());
    assert!(task.shape_errors.is_empty());
}

/// Build a task from a marker whose only field is `field_name: value`, and
/// assert it reports exactly one [`crate::model::ShapeError`] naming
/// `field_name` and `expected_message` — shared setup for the wrong-shape
/// tests below, each of which also checks its field still coerced to empty.
fn assert_reports_shape_error(field_name: &str, value: HVal, expected_message: &str) -> Task {
    let fields = dict(vec![(field_name, value)]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
    ]);
    let task = Task::from_marker("x".to_owned(), "x.typ".into(), &marker);
    assert_eq!(task.shape_errors.len(), 1);
    assert_eq!(task.shape_errors[0].field, field_name);
    assert_eq!(task.shape_errors[0].message, expected_message);
    task
}

/// `tags: ("backend")` — a Typst tuple missing its trailing comma, so a plain
/// string rather than a one-element array — must not silently look like no
/// tags authored at all: `from_marker` still coerces `tags` to empty (the
/// field is typed `Vec<String>`), but now names the mismatch in
/// `shape_errors` instead of leaving it unpoliced.
#[test]
fn from_marker_reports_tags_wrong_shape_instead_of_silent_empty() {
    let task = assert_reports_shape_error(
        "tags",
        HVal::Str("backend".to_owned()),
        "expected an array, found a string",
    );
    assert!(task.tags.is_empty());
}

/// `estimate: "2 days"` — a bare string rather than the `(unit: quantity)`
/// dict — must likewise be reported, not silently coerced to an empty map.
#[test]
fn from_marker_reports_estimate_wrong_shape_instead_of_silent_empty() {
    let task = assert_reports_shape_error(
        "estimate",
        HVal::Str("2 days".to_owned()),
        "expected a dict, found a string",
    );
    assert!(task.estimate.is_empty());
}

/// `links: "a.typ"` — a bare string rather than an array of edges — must
/// likewise be reported, not silently coerced to no edges.
#[test]
fn from_marker_reports_links_wrong_shape_instead_of_silent_empty() {
    let task = assert_reports_shape_error(
        "links",
        HVal::Str("a.typ".to_owned()),
        "expected an array, found a string",
    );
    assert!(task.links.is_empty());
}

/// `shape_error` names the Typst kind of the offending value in its message,
/// via `hval_kind` — one arm per `HVal` variant. `Null` never reaches it
/// through `from_marker` (the `field` helper filters nulls out), so it is
/// exercised directly on the crate-level surface.
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

#[test]
fn normalize_link_strips_path_and_extension() {
    assert_eq!(
        normalize_link("../tasks/foo.task-260101.typ"),
        "foo.task-260101"
    );
    assert_eq!(normalize_link("bar"), "bar");
}

/// A `links:` edge dict, mirroring the prelude helper shape (`task`, `kind`,
/// optional `note`).
fn edge_val(task: &str, kind: &str) -> HVal {
    dict(vec![
        ("task", HVal::Str(task.to_owned())),
        ("kind", HVal::Str(kind.to_owned())),
        ("note", HVal::Null),
    ])
}

/// Build a task, at `t.typ`, from a bare `links:` array — the shared
/// marker-wrapping boilerplate every `links` parsing test needs.
fn task_with_links(links: Vec<HVal>) -> Task {
    let fields = dict(vec![("links", HVal::Array(links))]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
    ]);
    Task::from_marker("t".to_owned(), "t.typ".into(), &marker)
}

#[test]
fn history_pointer_links_are_not_graph_edges() {
    // A SWHID tombstone in a `depends-on` edge is a history pointer, not a
    // task edge: it must drop out of `links` so validation never tries to
    // resolve it against a loaded task.
    let task = task_with_links(vec![
        edge_val(
            "swh:1:rev:0123456789abcdef0123456789abcdef01234567;path=tasks/gone.typ",
            "depends-on",
        ),
        edge_val("live.typ", "depends-on"),
    ]);
    assert_eq!(
        task.link_targets("depends-on").collect::<Vec<_>>(),
        vec!["live.typ"]
    );
    assert!(is_history_pointer(
        "swh:1:rev:0123456789abcdef0123456789abcdef01234567;path=tasks/gone.typ"
    ));
    assert!(!is_history_pointer("tasks/gone.typ"));
}

#[test]
fn from_marker_reads_supersedes_and_duplicates() {
    let task = task_with_links(vec![
        edge_val("old-task.typ", "supersedes"),
        edge_val("twin.typ", "duplicates"),
    ]);
    assert_eq!(
        task.link_targets("supersedes").collect::<Vec<_>>(),
        vec!["old-task.typ"]
    );
    assert_eq!(
        task.link_targets("duplicates").collect::<Vec<_>>(),
        vec!["twin.typ"]
    );
}

#[test]
fn from_marker_reads_bare_string_link_as_unknown_kind() {
    // No helper used: the harvested value is a bare string, not a dict. It
    // must still surface (not be silently dropped) so validation can flag it.
    let task = task_with_links(vec![HVal::Str("bare.typ".to_owned())]);
    assert_eq!(task.links.len(), 1);
    assert_eq!(task.links[0].task, "bare.typ");
    assert_eq!(task.links[0].kind, "");
}

/// Build a task from a marker whose `id:` field is `id` (the stem is `"stem"`).
fn task_from_id_field(id: &str) -> Task {
    let fields = dict(vec![
        ("title", HVal::Str("Task".to_owned())),
        ("id", HVal::Str(id.to_owned())),
    ]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
        ("extras", dict(vec![])),
    ]);
    Task::from_marker("stem".to_owned(), "tasks/stem.typ".into(), &marker)
}

#[test]
fn explicit_id_field_wins_over_stem() {
    assert_eq!(task_from_id_field("custom").id, "custom");
}

#[test]
fn empty_id_field_falls_back_to_stem() {
    assert_eq!(task_from_id_field("").id, "stem");
}

#[test]
fn is_terminal_matches_set() {
    let mut task = bare_task("t");
    task.status = Some(Status {
        name: "done".to_owned(),
        date: None,
        note: None,
    });
    assert!(task.is_terminal(&["done".to_owned()]));
    assert!(!task.is_terminal(&["cancelled".to_owned()]));
}

#[test]
fn resolve_link_path_sibling_file() {
    // From tasks/a.typ, "b.typ" resolves to tasks/b.typ
    let result = resolve_link_path("tasks/a.typ".as_ref(), "b.typ");
    assert_eq!(result, Some("tasks/b.typ".into()));
}

#[test]
fn resolve_link_path_parent_dir() {
    // From sub/a.typ, "../other/b.typ" resolves to other/b.typ
    let result = resolve_link_path("sub/a.typ".as_ref(), "../other/b.typ");
    assert_eq!(result, Some("other/b.typ".into()));
}

#[test]
fn resolve_link_path_current_dir_component() {
    // From tasks/a.typ, "./b.typ" resolves to tasks/b.typ
    let result = resolve_link_path("tasks/a.typ".as_ref(), "./b.typ");
    assert_eq!(result, Some("tasks/b.typ".into()));
}

#[test]
fn resolve_link_path_escapes_root_returns_none() {
    // From a.typ (at root level), "../escape.typ" would go above root
    let result = resolve_link_path("a.typ".as_ref(), "../escape.typ");
    assert_eq!(result, None);
}

#[test]
fn parse_links_stores_raw_paths() {
    // parse_links must NOT normalize; raw target strings are preserved
    let task = task_with_links(vec![
        edge_val("sibling.typ", "depends-on"),
        edge_val("../up/task.typ", "depends-on"),
    ]);
    assert_eq!(
        task.link_targets("depends-on").collect::<Vec<_>>(),
        vec!["sibling.typ", "../up/task.typ"]
    );
}

#[test]
fn all_tags_includes_authored_then_derived_no_duplicates() {
    let mut task = bare_task("t");
    task.tags = vec!["backend".into(), "db".into()];
    // "backend" also appears in derived_tags — must not duplicate
    task.derived_tags = vec!["auth".into(), "backend".into()];
    assert_eq!(task.all_tags(), vec!["backend", "db", "auth"]);
}

/// A `Task` loaded from a marker whose `priority` field is `priority` — the
/// shared setup for the framework-priority `from_marker` tests below.
fn task_with_priority(priority: HVal) -> Task {
    let fields = dict(vec![("priority", priority)]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".into())),
        ("fields", fields),
    ]);
    Task::from_marker("t".into(), "t.typ".into(), &marker)
}

/// Unwraps `task.priority` as a framework priority, panicking with the same
/// message the inlined `let else` used to give when it wasn't one.
fn expect_framework_priority(task: Task) -> (String, BTreeMap<String, InputValue>) {
    let NumValue::Framework { name, inputs } = task.priority.unwrap() else {
        panic!("expected framework priority");
    };
    (name, inputs)
}

#[test]
fn from_marker_reads_framework_priority_with_bool_inputs() {
    let priority = dict(vec![
        ("framework", HVal::Str("eisenhower".into())),
        (
            "inputs",
            dict(vec![
                ("urgent", HVal::Bool(true)),
                ("important", HVal::Bool(false)),
            ]),
        ),
    ]);
    let task = task_with_priority(priority);
    let (name, inputs) = expect_framework_priority(task);
    assert_eq!(name, "eisenhower");
    assert_eq!(inputs.get("urgent"), Some(&InputValue::Bool(true)));
    assert_eq!(inputs.get("important"), Some(&InputValue::Bool(false)));
}

#[test]
fn from_marker_reads_framework_priority_with_text_input() {
    let priority = dict(vec![
        ("framework", HVal::Str("moscow".into())),
        ("inputs", dict(vec![("level", HVal::Str("must".into()))])),
    ]);
    let task = task_with_priority(priority);
    let (name, inputs) = expect_framework_priority(task);
    assert_eq!(name, "moscow");
    assert_eq!(inputs.get("level"), Some(&InputValue::Text("must".into())));
}

/// Guards `file_key`'s separator normalisation: it must rejoin path
/// components with `/`, not defer to `Path::display` (which prints the
/// OS-native separator, `\` on Windows). Building `file` from per-component
/// `join` calls — rather than a literal `/`-joined string — exercises the
/// same code path `Path::components` would walk on either platform.
#[test]
fn file_key_joins_components_with_forward_slash() {
    let mut task = bare_task("nested");
    task.file = Path::new("tasks").join("sub").join("nested.typ");
    assert_eq!(task.file_key(), "tasks/sub/nested.typ");
}
