#![allow(clippy::unwrap_used)]

use super::check_all;
use crate::config::{Config, parse_config};
use crate::link::{Link, LinkKind, NoteRule};
use crate::model::Task;
use crate::test_support::bare_task;
use crate::validate::Violation;

fn edge(target: &str, kind: &str) -> Link {
    Link {
        task: target.to_owned(),
        kind: kind.to_owned(),
        note: None,
    }
}

fn noted(target: &str, kind: &str, note: &str) -> Link {
    Link {
        note: Some(note.to_owned()),
        ..edge(target, kind)
    }
}

fn check(tasks: &[Task], config: &Config) -> Vec<Violation> {
    let mut out = Vec::new();
    check_all(tasks, config, &mut out);
    out
}

#[test]
fn resolves_known_edge() {
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "depends-on")];
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(violations.is_empty(), "got {violations:?}");
}

#[test]
fn flags_unresolved_edge_target() {
    let mut a = bare_task("a");
    a.links = vec![edge("missing.typ", "depends-on")];
    let violations = check(&[a], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "depends-on" && v.message.contains("not found"))
    );
}

#[test]
fn flags_bare_string_edge_as_unknown_kind() {
    // No helper used: `kind` is empty, so it never matches a built-in or
    // configured kind — the escape-hatch rule requires intent be explicit.
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "")];
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "links" && v.message.contains("has no kind"))
    );
}

#[test]
fn flags_unknown_kind_name() {
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "blocks")];
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "blocks" && v.message.contains("not a configured link kind"))
    );
}

#[test]
fn parent_cardinality_one_rejects_two_parents() {
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "parent"), edge("extra.typ", "parent")];
    let b = bare_task("b");
    let extra = bare_task("extra");
    let violations = check(&[a, b, extra], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "parent" && v.message.contains("at most 1")),
        "got {violations:?}"
    );
}

#[test]
fn single_parent_is_clean() {
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "parent")];
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "parent"));
}

#[test]
fn depends_on_has_no_cardinality_cap() {
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "depends-on"), edge("extra.typ", "depends-on")];
    let b = bare_task("b");
    let extra = bare_task("extra");
    let violations = check(&[a, b, extra], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "depends-on"));
}

#[test]
fn related_requires_a_note() {
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "related")];
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "related" && v.message.contains("requires a note"))
    );
}

#[test]
fn related_with_note_is_clean() {
    let mut a = bare_task("a");
    a.links = vec![noted("b.typ", "related", "why it matters")];
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "related"));
}

#[test]
fn related_with_blank_note_still_fails() {
    let mut a = bare_task("a");
    a.links = vec![noted("b.typ", "related", "   ")];
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "related" && v.message.contains("requires a note"))
    );
}

fn config_with_custom_kind(kind: LinkKind) -> Config {
    let mut config = Config::default();
    config.tasks.links.kinds = vec![kind];
    config
}

#[test]
fn custom_config_kind_resolves() {
    let config = config_with_custom_kind(LinkKind {
        name: "blocks-release".to_owned(),
        max: None,
        note: NoteRule::Optional,
        blocking: false,
        acyclic: false,
    });
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "blocks-release")];
    let b = bare_task("b");
    let violations = check(&[a, b], &config);
    assert!(violations.is_empty(), "got {violations:?}");
}

#[test]
fn custom_kind_cannot_override_builtin_cardinality() {
    // A config entry named `parent` is shadowed by the immutable built-in
    // (cardinality 1), so two parents still fail even though the custom entry
    // declares an unbounded max.
    let config = config_with_custom_kind(LinkKind {
        name: "parent".to_owned(),
        max: None,
        note: NoteRule::Optional,
        blocking: false,
        acyclic: false,
    });
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "parent"), edge("extra.typ", "parent")];
    let b = bare_task("b");
    let extra = bare_task("extra");
    let violations = check(&[a, b, extra], &config);
    assert!(violations.iter().any(|v| v.field == "parent"));
}

#[test]
fn flags_depends_on_cycle() {
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "depends-on")];
    let mut b = bare_task("b");
    b.links = vec![edge("a.typ", "depends-on")];
    let violations = check(&[a, b], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "depends-on" && v.message.contains("cycle"))
    );
}

#[test]
fn flags_parent_cycle_separately_from_depends_on() {
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "parent")];
    let mut b = bare_task("b");
    b.links = vec![edge("a.typ", "parent")];
    let violations = check(&[a, b], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "parent" && v.message.contains("cycle")),
        "got {violations:?}"
    );
}

#[test]
fn parent_and_depends_on_cycles_are_independent() {
    // a depends-on b (fine, no cycle there) but a<->b both declare each other
    // as parent: only the parent cycle should be flagged.
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "depends-on"), edge("b.typ", "parent")];
    let mut b = bare_task("b");
    b.links = vec![edge("a.typ", "parent")];
    let violations = check(&[a, b], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "depends-on"));
    assert!(violations.iter().any(|v| v.field == "parent"));
}

#[test]
fn recurred_from_cardinality_one_rejects_two() {
    let mut a = bare_task("a");
    a.links = vec![
        edge("b.typ", "recurred-from"),
        edge("extra.typ", "recurred-from"),
    ];
    let b = bare_task("b");
    let extra = bare_task("extra");
    let violations = check(&[a, b, extra], &Config::default());
    assert!(
        violations
            .iter()
            .any(|v| v.field == "recurred-from" && v.message.contains("at most 1"))
    );
}

#[test]
fn strict_config_still_declares_all_builtins() {
    // A project that hand-configures its own custom kind must still see the
    // built-ins fully enforced (not accidentally cleared).
    let config = parse_config("[[tasks.links.kinds]]\nname = \"blocks-release\"\n").unwrap();
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "related")]; // no note -> still required
    let b = bare_task("b");
    let violations = check(&[a, b], &config);
    assert!(violations.iter().any(|v| v.field == "related"));
}

#[test]
fn forbidden_kind_without_note_is_clean() {
    let config = config_with_custom_kind(LinkKind {
        name: "no-notes-allowed".to_owned(),
        max: None,
        note: NoteRule::Forbidden,
        blocking: false,
        acyclic: false,
    });
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "no-notes-allowed")];
    let b = bare_task("b");
    let violations = check(&[a, b], &config);
    assert!(!violations.iter().any(|v| v.field == "no-notes-allowed"));
}

#[test]
fn forbidden_kind_with_note_is_flagged() {
    let config = config_with_custom_kind(LinkKind {
        name: "no-notes-allowed".to_owned(),
        max: None,
        note: NoteRule::Forbidden,
        blocking: false,
        acyclic: false,
    });
    let mut a = bare_task("a");
    a.links = vec![noted("b.typ", "no-notes-allowed", "some note")];
    let b = bare_task("b");
    let violations = check(&[a, b], &config);
    assert!(
        violations
            .iter()
            .any(|v| v.field == "no-notes-allowed" && v.message.contains("must not carry a note"))
    );
}
