#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use super::validate;
use crate::config::{Config, parse_config};
use crate::model::{Status, Task};
use crate::numeric::NumValue;
use crate::test_support::{bare_task, dep_edge, link_edge, noted_edge};

/// Status legend and priority/estimate/tags scale shared by every test in this
/// file. Field-specific validation (id format, enum fields, priority
/// frameworks, numeric scales) lives in `validate_tests_fields.rs`; this file
/// covers status reachability, the typed `links:` edges, and id uniqueness —
/// concerns that span more than one task or sit outside a single field.
pub(super) fn config() -> crate::config::Config {
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
required = "!terminal"
[[tasks.priority.bands]]
label = "low"
min = 0
max = 4
[[tasks.priority.bands]]
label = "high"
min = 5
max = 9

[tasks.estimate]
units = ["hours", "days"]

[tasks.tags]
allowed = ["backend"]
"#,
    )
    .unwrap()
}

pub(super) fn task(id: &str, status: &str) -> Task {
    let mut t = bare_task(id);
    t.status = Some(Status {
        name: status.to_owned(),
        date: None,
        note: None,
    });
    t.priority = Some(NumValue::Num(8.0));
    t.estimate = BTreeMap::from([("hours".to_owned(), 8.0)]);
    t
}

#[test]
fn clean_task_has_no_violations() {
    let tasks = vec![task("a", "wip")];
    assert!(validate(&tasks, &config()).is_empty());
}

#[test]
fn flags_unknown_status() {
    let tasks = vec![task("a", "bogus")];
    let violations = validate(&tasks, &config());
    assert!(violations.iter().any(|v| v.field == "status"));
}

/// Config with a transition graph: proposed → wip → done, plus `blocked` as an
/// orphan state (no incoming edge, so unreachable from the initial `proposed`).
fn transition_config() -> crate::config::Config {
    parse_config(
        r#"
[[tasks.status]]
value = "proposed"
to = ["wip"]
[[tasks.status]]
value = "wip"
to = ["done"]
[[tasks.status]]
value = "blocked"
[[tasks.status]]
value = "done"
terminal = true

[tasks.priority]
min = 0
max = 9
required = "!terminal"
[[tasks.priority.bands]]
label = "low"
min = 0
max = 4
[[tasks.priority.bands]]
label = "high"
min = 5
max = 9
"#,
    )
    .unwrap()
}

#[test]
fn flags_unreachable_status_under_transitions() {
    let tasks = vec![task("a", "blocked")];
    let violations = validate(&tasks, &transition_config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "status" && v.message.contains("unreachable")),
        "orphan status should be flagged, got: {violations:?}"
    );
}

#[test]
fn reachable_status_under_transitions_is_clean() {
    let tasks = vec![task("a", "wip")];
    assert!(validate(&tasks, &transition_config()).is_empty());
}

#[test]
fn unconstrained_legend_skips_reachability() {
    // The base config declares no `to`, so no status is ever unreachable.
    let tasks = vec![task("a", "wip")];
    assert!(validate(&tasks, &config()).is_empty());
}

/// `blocked` declares `note = "required"`; `cancelled` declares `note =
/// "forbidden"`; `wip`/`done` are left at the `Optional` default.
fn note_required_config() -> crate::config::Config {
    parse_config(
        r#"
[[tasks.status]]
value = "wip"
[[tasks.status]]
value = "blocked"
note = "required"
[[tasks.status]]
value = "cancelled"
terminal = true
note = "forbidden"
[[tasks.status]]
value = "done"
terminal = true
"#,
    )
    .unwrap()
}

#[test]
fn required_note_status_rejects_a_blank_note() {
    let mut item = task("a", "blocked");
    item.status.as_mut().unwrap().note = None;
    let violations = validate(&[item], &note_required_config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "status" && v.message.contains("requires a note")),
        "expected a required-note violation, got: {violations:?}"
    );
}

#[test]
fn required_note_status_rejects_a_whitespace_only_note() {
    let mut item = task("a", "blocked");
    item.status.as_mut().unwrap().note = Some("   ".to_owned());
    let violations = validate(&[item], &note_required_config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "status" && v.message.contains("requires a note"))
    );
}

#[test]
fn required_note_status_accepts_a_filled_note() {
    let mut item = task("a", "blocked");
    item.status.as_mut().unwrap().note = Some("waiting on upstream".to_owned());
    assert!(validate(&[item], &note_required_config()).is_empty());
}

/// A status left at the default `Optional` note rule never flags a note
/// violation, whether or not a note is present.
#[test]
fn optional_note_rule_is_unchanged() {
    let mut noted = task("a", "wip");
    noted.status.as_mut().unwrap().note = Some("fyi".to_owned());
    let bare = task("b", "wip");
    let violations = validate(&[noted, bare], &note_required_config());
    assert!(violations.iter().all(|v| v.field != "status"));
}

/// A terminal status left at the default note rule is likewise unaffected.
#[test]
fn terminal_status_without_note_requirement_is_unchanged() {
    let tasks = vec![task("a", "done")];
    assert!(validate(&tasks, &note_required_config()).is_empty());
}

/// `note = "forbidden"` is not enforced for a status (unlike a link kind):
/// a task landing there validates clean whether or not it carries a note.
#[test]
fn forbidden_note_rule_is_unchanged_for_a_status() {
    let mut noted = task("a", "cancelled");
    noted.status.as_mut().unwrap().note = Some("closing this out".to_owned());
    let bare = task("b", "cancelled");
    let violations = validate(&[noted, bare], &note_required_config());
    assert!(violations.iter().all(|v| v.field != "status"));
}

#[test]
fn flags_dependency_cycle() {
    let mut a = task("a", "wip");
    let mut b = task("b", "wip");
    a.links = vec![dep_edge("b.typ")];
    b.links = vec![dep_edge("a.typ")];
    let violations = validate(&[a, b], &config());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "depends-on" && v.message.contains("cycle"))
    );
}

#[test]
fn flags_unresolved_link() {
    let mut item = task("a", "wip");
    item.links = vec![dep_edge("missing.typ")];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "depends-on"));
}

#[test]
fn flags_unresolved_supersedes() {
    let mut item = task("a", "wip");
    item.links = vec![link_edge("gone.typ", "supersedes")];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "supersedes"));
}

#[test]
fn flags_unresolved_duplicates() {
    let mut item = task("a", "wip");
    item.links = vec![link_edge("ghost.typ", "duplicates")];
    let violations = validate(&[item], &config());
    assert!(violations.iter().any(|v| v.field == "duplicates"));
}

#[test]
fn resolves_supersedes_and_duplicates() {
    let mut first = task("a", "wip");
    // sibling file: tasks/b.typ — resolvable from tasks/a.typ as "b.typ"
    first.links = vec![
        link_edge("b.typ", "supersedes"),
        link_edge("b.typ", "duplicates"),
    ];
    let tasks = vec![first, task("b", "wip")];
    let violations = validate(&tasks, &config());
    assert!(!violations.iter().any(|v| v.field == "supersedes"));
    assert!(!violations.iter().any(|v| v.field == "duplicates"));
}

#[test]
fn resolves_existing_link() {
    let mut first = task("a", "wip");
    // sibling file: tasks/b.typ — resolvable from tasks/a.typ as "b.typ"
    first.links = vec![noted_edge("b.typ", "related", "why")];
    let tasks = vec![first, task("b", "wip")];
    assert!(
        !validate(&tasks, &config())
            .iter()
            .any(|v| v.field == "related")
    );
}

#[test]
fn broken_link_violation_names_resolved_path() {
    let mut item = task("a", "wip");
    item.links = vec![dep_edge("nonexistent.typ")];
    let violations = validate(&[item], &config());
    let v = violations
        .iter()
        .find(|v| v.field == "depends-on")
        .expect("expected a depends-on violation");
    assert!(
        v.message.contains("tasks/nonexistent.typ"),
        "violation message should name resolved path, got: {}",
        v.message
    );
}

#[test]
fn link_across_nested_directories_resolves() {
    // Task a lives at sub/a.typ; task b lives at other/b.typ.
    // From sub/a.typ, "b.typ" in other/ is reached via "../other/b.typ".
    let mut a = task("a", "wip");
    a.file = "sub/a.typ".into();
    a.links = vec![dep_edge("../other/b.typ")];

    let mut b = task("b", "wip");
    b.file = "other/b.typ".into();

    let tasks = vec![a, b];
    let violations = validate(&tasks, &config());
    assert!(
        !violations.iter().any(|v| v.field == "depends-on"),
        "cross-directory link should resolve, got: {violations:?}"
    );
}

#[test]
fn explicit_id_task_still_reachable_by_path() {
    // A task whose id differs from its stem is still found as a link target
    // because resolution is now path-based, not id-based.
    let mut target = task("stem", "wip");
    target.id = "custom-uuid-style-id".to_owned();
    target.file = "tasks/stem.typ".into();

    let mut linker = task("linker", "wip");
    linker.file = "tasks/linker.typ".into();
    linker.links = vec![dep_edge("stem.typ")];

    let tasks = vec![linker, target];
    let violations = validate(&tasks, &config());
    assert!(
        !violations.iter().any(|v| v.field == "depends-on"),
        "task with explicit id should still resolve by file path, got: {violations:?}"
    );
}

#[test]
fn broken_link_across_nested_directories_flagged() {
    let mut item = task("a", "wip");
    item.file = "sub/a.typ".into();
    item.links = vec![dep_edge("../missing/nowhere.typ")];
    let violations = validate(&[item], &config());
    assert!(
        violations.iter().any(|v| v.field == "depends-on"),
        "non-existent cross-directory link should be flagged"
    );
}

#[test]
fn flags_duplicate_ids() {
    let mut second = task("a", "wip");
    second.file = "tasks/a2.typ".into();
    let tasks = vec![task("a", "wip"), second];
    let violations = validate(&tasks, &Config::default());
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "expected uniqueness violation"
    );
}

#[test]
fn no_uniqueness_violation_for_distinct_ids() {
    let tasks = vec![task("a", "wip"), task("b", "wip")];
    assert!(
        !validate(&tasks, &Config::default())
            .iter()
            .any(|v| v.field == "id")
    );
}
