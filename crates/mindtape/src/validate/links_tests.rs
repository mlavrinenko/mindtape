#![allow(clippy::unwrap_used)]

use super::check_all;
use crate::config::{Config, parse_config};
use crate::link::{Link, LinkKind, NoteRule};
use crate::model::Task;
use crate::test_support::{bare_task, linked_task};
use crate::validate::Violation;
use crate::validate::tests::flags;
use crate::validate::violation::ViolationKind;

pub(super) fn edge(target: &str, kind: &str) -> Link {
    Link {
        task: target.to_owned(),
        kind: kind.to_owned(),
        note: None,
    }
}

pub(super) fn noted(target: &str, kind: &str, note: &str) -> Link {
    Link {
        note: Some(note.to_owned()),
        ..edge(target, kind)
    }
}

pub(super) fn check(tasks: &[Task], config: &Config) -> Vec<Violation> {
    let mut out = Vec::new();
    check_all(tasks, config, &mut out);
    out
}

#[test]
fn resolves_known_edge() {
    let a = linked_task("a", vec![edge("b.typ", "depends-on")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(violations.is_empty(), "got {violations:?}");
}

#[test]
fn flags_unresolved_edge_target() {
    let a = linked_task("a", vec![edge("missing.typ", "depends-on")]);
    let violations = check(&[a], &Config::default());
    assert!(
        flags(&violations, "depends-on", "not found"),
        "got {violations:?}"
    );
}

#[test]
fn names_only_an_unresolved_target_as_unresolved() {
    let a = linked_task(
        "a",
        vec![edge("missing.typ", "depends-on"), edge("b.typ", "")],
    );
    let violations = check(&[a, bare_task("b")], &Config::default());
    let unresolved: Vec<_> = violations
        .iter()
        .filter(|v| super::is_unresolved(v))
        .collect();
    assert_eq!(unresolved.len(), 1, "{violations:?}");
    assert!(
        unresolved
            .first()
            .is_some_and(|v| v.message.contains("missing.typ"))
    );
    assert!(violations.len() > 1, "{violations:?}");
}

#[test]
fn is_unresolved_reads_the_kind_and_ignores_the_wording() {
    let a = linked_task("a", vec![edge("missing.typ", "depends-on")]);
    let mut unresolved = check(&[a], &Config::default())
        .into_iter()
        .find(|v| v.field == "depends-on")
        .unwrap();
    assert_eq!(unresolved.kind, ViolationKind::UnresolvedLink);
    unresolved.message = "the target is gone".to_owned();
    assert!(super::is_unresolved(&unresolved));
    // The same words under another kind are not an unresolved target.
    unresolved.kind = ViolationKind::Other;
    unresolved.message = "link target `x.typ` not found".to_owned();
    assert!(!super::is_unresolved(&unresolved));
}

#[test]
fn flags_bare_string_edge_as_unknown_kind() {
    // No helper used: `kind` is empty, so it never matches a built-in or
    // configured kind — the escape-hatch rule requires intent be explicit.
    let a = linked_task("a", vec![edge("b.typ", "")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(
        flags(&violations, "links", "has no kind"),
        "got {violations:?}"
    );
}

#[test]
fn flags_unknown_kind_name() {
    let a = linked_task("a", vec![edge("b.typ", "blocks")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(
        flags(&violations, "blocks", "not a configured link kind"),
        "got {violations:?}"
    );
}

#[test]
fn parent_cardinality_one_rejects_two_parents() {
    let a = linked_task(
        "a",
        vec![edge("b.typ", "parent"), edge("extra.typ", "parent")],
    );
    let b = bare_task("b");
    let extra = bare_task("extra");
    let violations = check(&[a, b, extra], &Config::default());
    assert!(
        flags(&violations, "parent", "at most 1"),
        "got {violations:?}"
    );
}

#[test]
fn single_parent_is_clean() {
    let a = linked_task("a", vec![edge("b.typ", "parent")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "parent"));
}

#[test]
fn depends_on_has_no_cardinality_cap() {
    let a = linked_task(
        "a",
        vec![edge("b.typ", "depends-on"), edge("extra.typ", "depends-on")],
    );
    let b = bare_task("b");
    let extra = bare_task("extra");
    let violations = check(&[a, b, extra], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "depends-on"));
}

#[test]
fn related_requires_a_note() {
    let a = linked_task("a", vec![edge("b.typ", "related")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(
        flags(&violations, "related", "requires a note"),
        "got {violations:?}"
    );
}

#[test]
fn related_with_note_is_clean() {
    let a = linked_task("a", vec![noted("b.typ", "related", "why it matters")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "related"));
}

#[test]
fn related_with_blank_note_still_fails() {
    let a = linked_task("a", vec![noted("b.typ", "related", "   ")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &Config::default());
    assert!(
        flags(&violations, "related", "requires a note"),
        "got {violations:?}"
    );
}

pub(super) fn config_with_custom_kind(kind: LinkKind) -> Config {
    let mut config = Config::default();
    config.tasks.links.kinds = vec![kind];
    config
}

#[test]
fn custom_config_kind_resolves() {
    let config = config_with_custom_kind(LinkKind {
        name: "blocks-release".to_owned(),
        ..LinkKind::default()
    });
    let a = linked_task("a", vec![edge("b.typ", "blocks-release")]);
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
        ..LinkKind::default()
    });
    let a = linked_task(
        "a",
        vec![edge("b.typ", "parent"), edge("extra.typ", "parent")],
    );
    let b = bare_task("b");
    let extra = bare_task("extra");
    let violations = check(&[a, b, extra], &config);
    assert!(violations.iter().any(|v| v.field == "parent"));
}

#[test]
fn flags_depends_on_cycle() {
    let a = linked_task("a", vec![edge("b.typ", "depends-on")]);
    let b = linked_task("b", vec![edge("a.typ", "depends-on")]);
    let violations = check(&[a, b], &Config::default());
    assert!(
        flags(&violations, "depends-on", "cycle"),
        "got {violations:?}"
    );
}

#[test]
fn flags_parent_cycle_separately_from_depends_on() {
    let a = linked_task("a", vec![edge("b.typ", "parent")]);
    let b = linked_task("b", vec![edge("a.typ", "parent")]);
    let violations = check(&[a, b], &Config::default());
    assert!(flags(&violations, "parent", "cycle"), "got {violations:?}");
}

#[test]
fn parent_and_depends_on_cycles_are_independent() {
    // a depends-on b (fine, no cycle there) but a<->b both declare each other
    // as parent: only the parent cycle should be flagged.
    let a = linked_task(
        "a",
        vec![edge("b.typ", "depends-on"), edge("b.typ", "parent")],
    );
    let b = linked_task("b", vec![edge("a.typ", "parent")]);
    let violations = check(&[a, b], &Config::default());
    assert!(!violations.iter().any(|v| v.field == "depends-on"));
    assert!(violations.iter().any(|v| v.field == "parent"));
}

#[test]
fn recurred_from_cardinality_one_rejects_two() {
    let a = linked_task(
        "a",
        vec![
            edge("b.typ", "recurred-from"),
            edge("extra.typ", "recurred-from"),
        ],
    );
    let b = bare_task("b");
    let extra = bare_task("extra");
    let violations = check(&[a, b, extra], &Config::default());
    assert!(
        flags(&violations, "recurred-from", "at most 1"),
        "got {violations:?}"
    );
}

#[test]
fn strict_config_still_declares_all_builtins() {
    // A project that hand-configures its own custom kind must still see the
    // built-ins fully enforced (not accidentally cleared).
    let config = parse_config("[[tasks.links.kinds]]\nname = \"blocks-release\"\n").unwrap();
    let a = linked_task("a", vec![edge("b.typ", "related")]); // no note -> still required
    let b = bare_task("b");
    let violations = check(&[a, b], &config);
    assert!(violations.iter().any(|v| v.field == "related"));
}

#[test]
fn forbidden_kind_without_note_is_clean() {
    let config = config_with_custom_kind(LinkKind {
        name: "no-notes-allowed".to_owned(),
        note: NoteRule::Forbidden,
        ..LinkKind::default()
    });
    let a = linked_task("a", vec![edge("b.typ", "no-notes-allowed")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &config);
    assert!(!violations.iter().any(|v| v.field == "no-notes-allowed"));
}

#[test]
fn forbidden_kind_with_note_is_flagged() {
    let config = config_with_custom_kind(LinkKind {
        name: "no-notes-allowed".to_owned(),
        note: NoteRule::Forbidden,
        ..LinkKind::default()
    });
    let a = linked_task("a", vec![noted("b.typ", "no-notes-allowed", "some note")]);
    let b = bare_task("b");
    let violations = check(&[a, b], &config);
    assert!(
        flags(&violations, "no-notes-allowed", "must not carry a note"),
        "got {violations:?}"
    );
}

#[test]
fn flags_a_blocked_task_sitting_in_an_unblocked_status() {
    use crate::test_support::{dep_config, dep_task};
    // `unblocked = true` lands on `done`, the last entry dep_config writes.
    let config = dep_config("unblocked = true\n");
    let tasks = [
        dep_task("open", "wip", &[]),
        dep_task("closed", "done", &["open.typ"]),
        dep_task("free", "done", &[]),
        dep_task("working", "wip", &["open.typ"]),
    ];
    let violations = check(&tasks, &config);
    assert_eq!(
        crate::test_support::violation_triples(&violations),
        [(
            "closed",
            "status",
            "`done` requires an unblocked task; waits on: open (wip)"
        )]
    );
    assert_eq!(
        violations.first().map(|v| v.severity),
        Some(crate::Severity::Warn)
    );
    assert!(check(&tasks, &dep_config("")).is_empty());
}
