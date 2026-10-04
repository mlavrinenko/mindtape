//! Round-trip property tests for [`super::emit_task`]: for a representative
//! spread of `Task` values, emit then re-harvest through the real prelude
//! yields an equal task (modulo the loader-derived `file`/`derived_tags`/
//! `topo`/`blocks`, and `links`
//! order-normalized — grouping by kind/note can reorder same-kind edges),
//! and the emitted source passes [`crate::validate::validate`] clean for
//! that task's id.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::Path;

use typst_harvest::HVal;

use crate::config::parse_config;
use crate::link::Link;
use crate::loader::{LoadOutcome, load_project};
use crate::model::{Status, Task};
use crate::numeric::{InputValue, NumValue};
use crate::recur::{PeriodUnit, Recur, Rule};
use crate::test_support::bare_task;
use crate::validate::validate;

use super::emit_task;

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

/// Activates `ice` as the priority framework, leaving manual priority
/// allowed (the default), so both framework- and scalar-priority fixtures
/// validate against the same project.
const CONFIG: &str = "[tasks.priority]\nframework = \"ice\"\n";

/// `CONFIG` plus a project-declared custom link kind, for the `edge(...)`
/// escape-hatch coverage.
const LINKS_CONFIG: &str = "[tasks.priority]\nframework = \"ice\"\n\
     [[tasks.links.kinds]]\nname = \"blocks\"\nnote = \"optional\"\n";

/// A minimal, self-valid stub task written at a link target's path so
/// resolution succeeds; its own violations (if any) are irrelevant — tests
/// only assert the fixture-under-test's violations are empty.
const STUB: &str = "#import \"@local/mindtape:0.2.0\": *\n\n#show: task.with(\n  title: \"Stub\",\n  status: wip(2026, 1, 1),\n)\n";

/// Write the in-tree `lib/` package override, `config_text` to
/// `.mindtape/config.toml`, and every `(name, source)` under `tasks/`, then
/// load the project. Mirrors `crates/mindtape-cli/src/flip_tests.rs::setup`,
/// minus the CLI's `test_util::make_project` (unavailable from this crate).
fn setup(dir: &Path, config_text: &str, files: &[(String, String)]) -> LoadOutcome {
    crate::test_support::write_lib_package(dir, PRELUDE);
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), config_text).unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
    for (name, source) in files {
        std::fs::write(dir.join("tasks").join(name), source).unwrap();
    }
    let config = parse_config(config_text).unwrap();
    load_project(dir, &config)
}

/// A `kind, task, note` sort key, so link-order differences introduced by
/// grouping (same kind, different notes) don't fail the round-trip compare.
fn link_sort_key(link: &Link) -> (String, String, Option<String>) {
    (link.kind.clone(), link.task.clone(), link.note.clone())
}

/// Round-trip `task` through `emit_task` + reload against `config_text`,
/// writing `stub_ids` as extra minimal tasks so `task`'s link targets
/// resolve. Asserts: the source parses to exactly one task, it equals `task`
/// field-by-field (modulo the loader-derived `file`/`derived_tags`/`topo`/
/// `blocks` and link order), and it passes validation with no violation
/// attributed to `task.id`.
fn assert_round_trips(task: &Task, config_text: &str, stub_ids: &[&str]) {
    let dir = tempfile::tempdir().unwrap();
    let source = emit_task(task);
    let mut files = vec![(format!("{}.typ", task.id), source.clone())];
    for id in stub_ids {
        files.push((format!("{id}.typ"), STUB.to_owned()));
    }
    let outcome = setup(dir.path(), config_text, &files);
    assert!(
        outcome.errors.is_empty(),
        "{:?}\n--- source ---\n{source}",
        outcome.errors
    );
    let mut reloaded = outcome
        .tasks
        .into_iter()
        .find(|found| found.id == task.id)
        .unwrap_or_else(|| panic!("task `{}` did not load\n--- source ---\n{source}", task.id));
    reloaded.file.clone_from(&task.file);
    reloaded.derived_tags.clone_from(&task.derived_tags);
    // `topo`/`blocks` are derived from the whole loaded set (`crate::dep_rank`)
    // exactly as `derived_tags` is, and nothing about them is written to the
    // file — so the stub link targets this fixture plants give the reloaded
    // task a graph position the authored one never carried. Not a round-trip
    // property; `dep_rank_tests.rs` is what pins their values.
    reloaded.topo = task.topo;
    reloaded.blocks = task.blocks;
    // Likewise `spans`: the loader reads it off the source it is handed,
    // and the authored fixture was built in memory with no source at all. What
    // a round trip owes is the fields the emitter writes — a line number is a
    // property of the file, not of the task.
    reloaded.spans = task.spans.clone();
    reloaded.links.sort_by_key(link_sort_key);

    let mut expected = task.clone();
    expected.links.sort_by_key(link_sort_key);
    assert_eq!(reloaded, expected, "--- source ---\n{source}");

    let config = parse_config(config_text).unwrap();
    let all_tasks: Vec<Task> = {
        let mut fresh = setup(dir.path(), config_text, &files).tasks;
        fresh.retain(|found| found.id == task.id || stub_ids.contains(&found.id.as_str()));
        fresh
    };
    let violations = validate(&all_tasks, &config);
    let for_task: Vec<_> = violations
        .iter()
        .filter(|violation| violation.task_id == task.id)
        .collect();
    assert!(
        for_task.is_empty(),
        "{for_task:?}\n--- source ---\n{source}"
    );
}

fn status(name: &str, date: &str) -> Status {
    Status {
        name: name.to_owned(),
        date: Some(date.to_owned()),
        note: None,
    }
}

#[test]
fn round_trips_scalar_priority_and_difficulty() {
    let mut task = bare_task("scalar-priority");
    task.status = Some(status("wip", "2026-06-01"));
    task.priority = Some(NumValue::Num(4.0));
    task.difficulty = Some(NumValue::Num(6.0));
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn round_trips_framework_priority() {
    let mut task = bare_task("framework-priority");
    task.status = Some(status("wip", "2026-06-01"));
    let mut inputs = BTreeMap::new();
    inputs.insert("impact".to_owned(), InputValue::Num(8.0));
    inputs.insert("confidence".to_owned(), InputValue::Num(0.75));
    inputs.insert("ease".to_owned(), InputValue::Num(4.0));
    task.priority = Some(NumValue::Framework {
        name: "ice".to_owned(),
        inputs,
    });
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn round_trips_multi_kind_links_with_notes() {
    let mut task = bare_task("multi-kind-links");
    task.status = Some(status("wip", "2026-06-01"));
    task.links = vec![
        Link {
            task: "alpha.typ".to_owned(),
            kind: "depends-on".to_owned(),
            note: None,
        },
        Link {
            task: "beta.typ".to_owned(),
            kind: "depends-on".to_owned(),
            note: Some("after the migration".to_owned()),
        },
        Link {
            task: "gamma.typ".to_owned(),
            kind: "related".to_owned(),
            note: Some("see also [prior] work \\ notes".to_owned()),
        },
    ];
    assert_round_trips(&task, CONFIG, &["alpha", "beta", "gamma"]);
}

#[test]
fn round_trips_custom_link_kind() {
    let mut task = bare_task("custom-link-kind");
    task.status = Some(status("wip", "2026-06-01"));
    task.links = vec![Link {
        task: "delta.typ".to_owned(),
        kind: "blocks".to_owned(),
        note: Some("waiting on delta".to_owned()),
    }];
    assert_round_trips(&task, LINKS_CONFIG, &["delta"]);
}

#[test]
fn round_trips_estimate_and_dates() {
    let mut task = bare_task("estimate-and-dates");
    task.status = Some(status("wip", "2026-06-01"));
    task.estimate = BTreeMap::from([("days".to_owned(), 1.0), ("hours".to_owned(), 2.5)]);
    task.due = Some("2026-07-01".to_owned());
    task.start = Some("2026-06-15".to_owned());
    task.reviewed = Some("2026-06-20".to_owned());
    task.tags = vec!["backend".to_owned()];
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn round_trips_multi_tag() {
    let mut task = bare_task("multi-tag");
    task.status = Some(status("wip", "2026-06-01"));
    task.tags = vec!["backend".to_owned(), "cli".to_owned()];
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn round_trips_extras_scalars_and_nested() {
    let mut task = bare_task("extras");
    task.status = Some(status("wip", "2026-06-01"));
    task.extras = BTreeMap::from([
        ("x-str".to_owned(), HVal::Str("hello \"world\"".to_owned())),
        ("x-int".to_owned(), HVal::Int(3)),
        ("x-float".to_owned(), HVal::Float(3.0)),
        ("x-bool".to_owned(), HVal::Bool(true)),
        (
            "x-list".to_owned(),
            HVal::Array(vec![HVal::Int(1), HVal::Str("two".to_owned())]),
        ),
        (
            "x-map".to_owned(),
            HVal::Dict(BTreeMap::from([
                ("a".to_owned(), HVal::Int(1)),
                ("in-review".to_owned(), HVal::Bool(false)),
            ])),
        ),
    ]);
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn round_trips_status_with_note() {
    let mut task = bare_task("status-note");
    task.status = Some(Status {
        name: "wip".to_owned(),
        date: Some("2026-06-05".to_owned()),
        note: Some("waiting on * review".to_owned()),
    });
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn round_trips_recur_every() {
    let mut task = bare_task("recur-every");
    task.status = Some(status("wip", "2026-06-01"));
    task.recur = Some(Recur {
        rule: Rule::Period {
            unit: PeriodUnit::Weeks,
            value: 1,
            weekdays: vec![chrono::Weekday::Mon, chrono::Weekday::Wed],
        },
        anchor: Some("due".to_owned()),
        until: Some("2027-01-01".to_owned()),
        count: Some(10),
        materialize: Some("spawn".to_owned()),
    });
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn round_trips_non_empty_body() {
    let mut task = bare_task("with-body");
    task.status = Some(status("wip", "2026-06-01"));
    task.body = "== Summary\n\nSome prose about the task.".to_owned();
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn emits_nothing_after_marker_when_body_empty() {
    let mut task = bare_task("empty-body");
    task.status = Some(status("wip", "2026-06-01"));
    let source = emit_task(&task);
    assert!(source.ends_with(")\n"), "{source}");
}

#[test]
fn round_trips_kind_and_assignee() {
    let mut task = bare_task("kind-and-assignee");
    task.status = Some(status("wip", "2026-06-01"));
    task.kind = Some("bug".to_owned());
    task.assignee = Some("ada".to_owned());
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn emits_label_priority_as_a_string() {
    let mut task = bare_task("label-priority");
    task.status = Some(status("wip", "2026-06-01"));
    task.priority = Some(NumValue::Label("high".to_owned()));
    let source = emit_task(&task);
    assert!(source.contains("priority: \"high\""), "{source}");
}

#[test]
fn emits_moscow_framework_as_a_positional_call() {
    // `moscow(level)` takes one positional argument, not a named one —
    // distinct emission shape from every other named framework.
    let mut task = bare_task("moscow-priority");
    task.status = Some(status("wip", "2026-06-01"));
    task.priority = Some(NumValue::Framework {
        name: "moscow".to_owned(),
        inputs: BTreeMap::from([("level".to_owned(), InputValue::Text("must".to_owned()))]),
    });
    let source = emit_task(&task);
    assert!(source.contains("priority: moscow(\"must\")"), "{source}");
}

#[test]
fn emits_unnamed_framework_via_the_generic_escape_hatch() {
    let mut task = bare_task("custom-framework-priority");
    task.status = Some(status("wip", "2026-06-01"));
    task.priority = Some(NumValue::Framework {
        name: "custom".to_owned(),
        inputs: BTreeMap::new(),
    });
    let source = emit_task(&task);
    assert!(
        source.contains("priority: framework(\"custom\")"),
        "{source}"
    );
}

#[test]
fn emits_unnamed_framework_with_inputs_via_the_generic_escape_hatch() {
    let mut task = bare_task("custom-framework-with-inputs");
    task.status = Some(status("wip", "2026-06-01"));
    task.priority = Some(NumValue::Framework {
        name: "custom".to_owned(),
        inputs: BTreeMap::from([
            ("flag".to_owned(), InputValue::Bool(true)),
            ("note".to_owned(), InputValue::Text("careful".to_owned())),
        ]),
    });
    let source = emit_task(&task);
    assert!(
        source.contains(r#"priority: framework("custom", flag: true, note: "careful")"#),
        "{source}"
    );
}

#[test]
fn emit_recur_rrule_escape_hatch_round_trips_through_reload() {
    // `check_policy` gates `rrule(...)` behind config and the `rrule` cargo
    // feature — orthogonal to emission, and this crate's own tests build
    // without the feature — so this reloads without calling `validate` (the
    // way `assert_round_trips` does), proving the marker-parse round trip on
    // its own rather than the policy check this build cannot honour.
    let dir = tempfile::tempdir().unwrap();
    let mut task = bare_task("rrule-recur");
    task.status = Some(status("wip", "2026-06-01"));
    task.recur = Some(Recur {
        rule: Rule::Rrule {
            raw: "FREQ=MONTHLY;BYDAY=1MO".to_owned(),
        },
        anchor: Some("due".to_owned()),
        until: None,
        count: None,
        materialize: None,
    });
    let source = emit_task(&task);
    assert!(
        source.contains(r#"recur: rrule("FREQ=MONTHLY;BYDAY=1MO", anchor: "due")"#),
        "{source}"
    );
    let files = vec![(format!("{}.typ", task.id), source.clone())];
    let outcome = setup(dir.path(), CONFIG, &files);
    assert!(
        outcome.errors.is_empty(),
        "{:?}\n--- source ---\n{source}",
        outcome.errors
    );
    let reloaded = outcome
        .tasks
        .into_iter()
        .find(|found| found.id == task.id)
        .unwrap_or_else(|| panic!("task `{}` did not load\n--- source ---\n{source}", task.id));
    assert_eq!(reloaded.recur, task.recur);
}

#[test]
fn round_trips_recur_without_specific_weekdays() {
    let mut task = bare_task("recur-no-weekdays");
    task.status = Some(status("wip", "2026-06-01"));
    task.recur = Some(Recur {
        rule: Rule::Period {
            unit: PeriodUnit::Days,
            value: 3,
            weekdays: Vec::new(),
        },
        anchor: None,
        until: None,
        count: None,
        materialize: None,
    });
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn round_trips_recur_with_every_weekday() {
    use chrono::Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed};
    let mut task = bare_task("recur-all-weekdays");
    task.status = Some(status("wip", "2026-06-01"));
    task.recur = Some(Recur {
        rule: Rule::Period {
            unit: PeriodUnit::Weeks,
            value: 1,
            weekdays: vec![Mon, Tue, Wed, Thu, Fri, Sat, Sun],
        },
        anchor: None,
        until: None,
        count: None,
        materialize: None,
    });
    assert_round_trips(&task, CONFIG, &[]);
}

#[test]
fn emits_deterministic_field_order() {
    let mut task = bare_task("deterministic");
    task.status = Some(status("wip", "2026-06-01"));
    task.priority = Some(NumValue::Num(4.0));
    task.tags = vec!["a".to_owned()];
    let first = emit_task(&task);
    let second = emit_task(&task);
    assert_eq!(first, second);
}
