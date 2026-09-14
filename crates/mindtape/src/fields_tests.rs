#![allow(clippy::unwrap_used)]

use super::{
    BODY_FIELD, FIELDS, FieldValue, NOTE_FIELD, TAGS_FIELD, contains_field_names, get,
    is_contains_field, rename_hint,
};
use crate::config::{Config, parse_config};
use crate::model::Status;
use crate::numeric::NumValue;
use crate::test_support::bare_task;

fn config() -> Config {
    parse_config(
        r#"
[[tasks.status]]
value = "proposed"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "done"
terminal = true

[tasks.priority]
min = 0
max = 9
"#,
    )
    .unwrap()
}

#[test]
fn get_resolves_every_registered_name() {
    let cfg = config();
    let task = bare_task("t");
    for def in FIELDS {
        assert!(get(def.name).is_some(), "{} must resolve", def.name);
        // Every accessor must run cleanly against a bare task (all fields unset).
        let _ = (def.get)(&task, &cfg);
    }
}

#[test]
fn unknown_name_is_none() {
    assert!(get("bogus").is_none());
}

#[test]
fn bare_task_fields_are_missing() {
    let cfg = config();
    let task = bare_task("t");
    for name in [
        "status",
        "priority",
        "difficulty",
        "kind",
        "due",
        "start",
        "reviewed",
        "assignee",
    ] {
        assert!(
            matches!((get(name).unwrap().get)(&task, &cfg), FieldValue::Missing),
            "{name} must be Missing on a bare task"
        );
    }
}

#[test]
fn title_and_id_are_never_missing() {
    let cfg = config();
    let task = bare_task("t");
    assert!(matches!(
        (get("title").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "t"
    ));
    assert!(matches!(
        (get("id").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "t"
    ));
}

#[test]
fn depth_resolves_numeric() {
    let cfg = config();
    let mut task = bare_task("t");
    task.depth = 3;
    assert!(matches!(
        (get("depth").unwrap().get)(&task, &cfg),
        FieldValue::Num(n) if n == 3.0
    ));
}

#[test]
fn assignee_and_reviewed_resolve_as_strings() {
    let cfg = config();
    let mut task = bare_task("t");
    task.assignee = Some("ada".to_owned());
    task.reviewed = Some("2026-08-01".to_owned());
    assert!(matches!(
        (get("assignee").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "ada"
    ));
    assert!(matches!(
        (get("reviewed").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "2026-08-01"
    ));
}

#[test]
fn status_resolves_the_legend_name() {
    let cfg = config();
    let mut task = bare_task("t");
    task.status = Some(Status {
        name: "wip".to_owned(),
        date: None,
        note: None,
    });
    assert!(matches!(
        (get("status").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "wip"
    ));
}

#[test]
fn status_parts_resolve_the_nested_values() {
    let cfg = config(); // proposed(0), wip(1), done(2)
    let mut task = bare_task("t");
    task.status = Some(Status {
        name: "wip".to_owned(),
        date: Some("2026-06-14".to_owned()),
        note: Some("waiting on review".to_owned()),
    });
    let value = |name: &str| (get(name).unwrap().get)(&task, &cfg);
    assert!(matches!(value("status.name"), FieldValue::Str(s) if s == "wip"));
    assert!(matches!(value("status.date"), FieldValue::Str(s) if s == "2026-06-14"));
    assert!(matches!(value("status.note"), FieldValue::Str(s) if s == "waiting on review"));
    assert!(matches!(value("status.index"), FieldValue::Num(n) if (n - 1.0).abs() < f64::EPSILON));
}

#[test]
fn status_parts_are_missing_without_a_status() {
    let cfg = config();
    let task = bare_task("t");
    for name in [
        "status",
        "status.name",
        "status.date",
        "status.note",
        "status.index",
    ] {
        assert!(
            matches!((get(name).unwrap().get)(&task, &cfg), FieldValue::Missing),
            "{name} must be Missing on a statusless task"
        );
    }
}

#[test]
fn status_index_is_missing_outside_the_legend() {
    // The index is a derivation through the configured legend, so a name the
    // legend does not carry has no position to read. The bare field still
    // resolves the written name — that is the field's long-standing value.
    let cfg = config();
    let mut task = bare_task("t");
    task.status = Some(Status {
        name: "unlisted".to_owned(),
        date: None,
        note: None,
    });
    assert!(matches!(
        (get("status.index").unwrap().get)(&task, &cfg),
        FieldValue::Missing
    ));
    assert!(matches!(
        (get("status").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "unlisted"
    ));
}

#[test]
fn priority_raw_number_resolves_numeric() {
    let cfg = config();
    let mut task = bare_task("t");
    task.priority = Some(NumValue::Num(7.0));
    assert!(matches!(
        (get("priority").unwrap().get)(&task, &cfg),
        FieldValue::Num(n) if n == 7.0
    ));
}

#[test]
fn note_and_tags_are_contains_fields_but_not_sortable_fields() {
    // `note`/`tags` extend the text-only `contains()` tier alongside `body`;
    // they must never enter `FIELDS`, which backs comparison and `--sort`.
    for name in [BODY_FIELD, NOTE_FIELD, TAGS_FIELD] {
        assert!(is_contains_field(name), "{name} must be a contains field");
        assert!(get(name).is_none(), "{name} must stay out of FIELDS");
    }
}

#[test]
fn contains_field_names_lists_fields_then_the_text_only_tier() {
    let names = contains_field_names();
    for def in FIELDS {
        assert!(names.contains(&def.name), "{} missing", def.name);
    }
    for text_only in [BODY_FIELD, NOTE_FIELD, TAGS_FIELD] {
        assert!(names.contains(&text_only), "{text_only} missing");
    }
    assert_eq!(names.len(), FIELDS.len() + 3);
}

#[test]
fn priority_label_falls_back_to_string() {
    let cfg = config();
    let mut task = bare_task("t");
    task.priority = Some(NumValue::Label("high".to_owned()));
    assert!(matches!(
        (get("priority").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "high"
    ));
}

// --- The `file` → `path` rename ---

#[test]
fn path_resolves_the_root_relative_key_and_file_is_gone() {
    // The whole point of the rename: `file` names the ls column (a
    // working-directory-relative display) and nothing else, so it must not
    // resolve as a comparable field at all — including through `contains()`,
    // whose vocabulary is derived from this registry.
    let cfg = config();
    let mut task = bare_task("t");
    task.file = "tasks/sub/nested.typ".into();
    assert!(matches!(
        (get("path").unwrap().get)(&task, &cfg),
        FieldValue::Str(s) if s == "tasks/sub/nested.typ"
    ));
    assert!(get("file").is_none());
    assert!(!is_contains_field("file"));
}

#[test]
fn rename_hint_points_the_old_spelling_at_the_new_one() {
    let hint = rename_hint("file");
    assert!(hint.contains("`path`"), "{hint}");
    assert!(
        hint.starts_with("; "),
        "the hint is a trailing clause a caller appends unconditionally: {hint}"
    );
}

#[test]
fn rename_hint_is_empty_for_a_name_that_was_never_a_field() {
    // Silence for an ordinary typo, so appending the clause unconditionally
    // cannot bolt a rename story onto an unrelated unknown name.
    assert_eq!(rename_hint("bogus"), "");
    assert_eq!(rename_hint("path"), "");
}

// --- Graph-derived fields (`topo`, `blocks`) ---

#[test]
fn graph_ranks_read_back_as_numbers() {
    let cfg = config();
    let mut task = bare_task("t");
    task.topo = 2;
    task.blocks = 5;
    assert!(matches!(
        (get("topo").unwrap().get)(&task, &cfg),
        FieldValue::Num(n) if (n - 2.0).abs() < f64::EPSILON
    ));
    assert!(matches!(
        (get("blocks").unwrap().get)(&task, &cfg),
        FieldValue::Num(n) if (n - 5.0).abs() < f64::EPSILON
    ));
}

#[test]
fn graph_ranks_are_never_missing() {
    // Like `depth`, they are derived numbers rather than authored fields, so a
    // task that names no link still carries 0 rather than nothing. That is what
    // makes `blocks > 3` a plain numeric comparison with no missing-field case.
    let cfg = config();
    let task = bare_task("t");
    for name in ["topo", "blocks"] {
        assert!(
            matches!((get(name).unwrap().get)(&task, &cfg), FieldValue::Num(n) if n == 0.0),
            "{name} must read 0 on a bare task"
        );
    }
}

#[test]
fn graph_ranks_are_filterable() {
    let cfg = config();
    let mut task = bare_task("t");
    task.topo = 1;
    task.blocks = 5;
    let hit = |expr: &str| {
        crate::filter::Filter::parse(expr)
            .unwrap()
            .matches(&task, &cfg)
            .unwrap()
    };
    assert!(hit("blocks > 3"));
    assert!(!hit("blocks > 5"));
    assert!(hit("blocks == 5"));
    assert!(hit("topo == 1"));
    assert!(!hit("topo == 0"));
    assert!(hit("blocks > 3 && topo < 2"));
}
