#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use typst_harvest::HVal;

use super::{Status, Task, is_history_pointer, normalize_link, resolve_link_path};
use crate::numeric::NumValue;

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
            "relates-to",
            HVal::Array(vec![HVal::Str("other.task-260101.typ".to_owned())]),
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
    assert_eq!(task.relates_to, vec!["other.task-260101.typ".to_owned()]);
    assert_eq!(task.tags, vec!["backend".to_owned()]);
    assert!(task.extras.contains_key("x-speconaut"));
}

#[test]
fn from_marker_tolerates_missing_fields() {
    let marker = dict(vec![("marker", HVal::Str("mindtape.task".to_owned()))]);
    let task = Task::from_marker("x".to_owned(), "x.typ".into(), &marker);
    assert_eq!(task.title, "");
    assert!(task.status.is_none());
    assert!(task.tags.is_empty());
}

#[test]
fn normalize_link_strips_path_and_extension() {
    assert_eq!(
        normalize_link("../tasks/foo.task-260101.typ"),
        "foo.task-260101"
    );
    assert_eq!(normalize_link("bar"), "bar");
}

#[test]
fn history_pointer_links_are_not_graph_edges() {
    // A SWHID tombstone in `depends-on` is a history pointer, not a task edge:
    // it must drop out of the normalized link list so validation never tries to
    // resolve it against a loaded task.
    let fields = dict(vec![(
        "depends-on",
        HVal::Array(vec![
            HVal::Str(
                "swh:1:rev:0123456789abcdef0123456789abcdef01234567;path=tasks/gone.typ".to_owned(),
            ),
            HVal::Str("live.typ".to_owned()),
        ]),
    )]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
    ]);
    let task = Task::from_marker("t".to_owned(), "t.typ".into(), &marker);
    assert_eq!(task.depends_on, vec!["live.typ".to_owned()]);
    assert!(is_history_pointer(
        "swh:1:rev:0123456789abcdef0123456789abcdef01234567;path=tasks/gone.typ"
    ));
    assert!(!is_history_pointer("tasks/gone.typ"));
}

#[test]
fn from_marker_reads_supersedes_and_duplicates() {
    let fields = dict(vec![
        (
            "supersedes",
            HVal::Array(vec![HVal::Str("old-task.typ".to_owned())]),
        ),
        (
            "duplicates",
            HVal::Array(vec![HVal::Str("twin.typ".to_owned())]),
        ),
    ]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
    ]);
    let task = Task::from_marker("t".to_owned(), "t.typ".into(), &marker);
    assert_eq!(task.supersedes, vec!["old-task.typ".to_owned()]);
    assert_eq!(task.duplicates, vec!["twin.typ".to_owned()]);
}

#[test]
fn explicit_id_field_wins_over_stem() {
    let fields = dict(vec![
        ("title", HVal::Str("Task".to_owned())),
        ("id", HVal::Str("custom".to_owned())),
    ]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
        ("extras", dict(vec![])),
    ]);
    let task = Task::from_marker("stem".to_owned(), "tasks/stem.typ".into(), &marker);
    assert_eq!(task.id, "custom");
}

#[test]
fn empty_id_field_falls_back_to_stem() {
    let fields = dict(vec![
        ("title", HVal::Str("Task".to_owned())),
        ("id", HVal::Str(String::new())),
    ]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
        ("extras", dict(vec![])),
    ]);
    let task = Task::from_marker("stem".to_owned(), "tasks/stem.typ".into(), &marker);
    assert_eq!(task.id, "stem");
}

#[test]
fn is_terminal_matches_set() {
    let task = Task {
        id: "t".to_owned(),
        file: "t.typ".into(),
        title: String::new(),
        status: Some(Status {
            name: "done".to_owned(),
            date: None,
            note: None,
        }),
        priority: None,
        difficulty: None,
        kind: None,
        estimate: BTreeMap::new(),
        depends_on: Vec::new(),
        relates_to: Vec::new(),
        supersedes: Vec::new(),
        duplicates: Vec::new(),
        tags: Vec::new(),
        due: None,
        start: None,
        reviewed: None,
        extras: BTreeMap::new(),
    };
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
    let fields = dict(vec![(
        "depends-on",
        HVal::Array(vec![
            HVal::Str("sibling.typ".to_owned()),
            HVal::Str("../up/task.typ".to_owned()),
        ]),
    )]);
    let marker = dict(vec![
        ("marker", HVal::Str("mindtape.task".to_owned())),
        ("fields", fields),
    ]);
    let task = Task::from_marker("t".to_owned(), "sub/t.typ".into(), &marker);
    assert_eq!(
        task.depends_on,
        vec!["sibling.typ".to_owned(), "../up/task.typ".to_owned()]
    );
}
