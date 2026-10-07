#![allow(clippy::unwrap_used)]

use super::check_all;
use crate::config::{Config, parse_config};
use crate::link::{Link, LinkKind, NoteRule, TargetValidator};
use crate::model::Task;
use crate::test_support::bare_task;
use crate::validate::Violation;
use crate::validate::violation::ViolationKind;

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
fn names_only_an_unresolved_target_as_unresolved() {
    let mut a = bare_task("a");
    a.links = vec![edge("missing.typ", "depends-on"), edge("b.typ", "")];
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
    let mut a = bare_task("a");
    a.links = vec![edge("missing.typ", "depends-on")];
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

/// A config with one kind pointing outside the backlog, validated by `url`.
fn config_with_external_kind(kind: LinkKind) -> Config {
    config_with_custom_kind(LinkKind {
        validate: TargetValidator::Url,
        ..kind
    })
}

/// A well-formed external target does not go through stem resolution: it names
/// no task file, and that is not an error.
#[test]
fn external_target_is_not_resolved_against_tasks() {
    let config = config_with_external_kind(LinkKind {
        name: "discharged-by".to_owned(),
        ..LinkKind::default()
    });
    let mut a = bare_task("a");
    a.links = vec![edge("https://example.com/pr/7", "discharged-by")];
    let violations = check(&[a], &config);
    assert!(violations.is_empty(), "got {violations:?}");
}

/// A malformed external target IS an error, and the message names the kind,
/// the validator, and the offending value.
#[test]
fn malformed_external_target_names_kind_validator_and_value() {
    let config = config_with_external_kind(LinkKind {
        name: "discharged-by".to_owned(),
        ..LinkKind::default()
    });
    let mut a = bare_task("a");
    a.links = vec![edge("not a url", "discharged-by")];
    let violations = check(&[a], &config);
    let hit = violations
        .iter()
        .find(|v| v.field == "discharged-by")
        .unwrap_or_else(|| panic!("expected a violation, got {violations:?}"));
    assert!(hit.message.contains("discharged-by"), "{}", hit.message);
    assert!(hit.message.contains("url"), "{}", hit.message);
    assert!(hit.message.contains("not a url"), "{}", hit.message);
}

/// Cycle detection never sees an external kind, even one flagged `acyclic`:
/// two tasks naming each other through it are not a loop, because neither edge
/// is a task-graph edge.
#[test]
fn external_kind_participates_in_no_cycle() {
    let config = config_with_external_kind(LinkKind {
        name: "mirrors".to_owned(),
        acyclic: true,
        ..LinkKind::default()
    });
    let mut a = bare_task("a");
    a.links = vec![edge("https://example.com/a", "mirrors")];
    let mut b = bare_task("b");
    b.links = vec![edge("https://example.com/b", "mirrors")];
    let violations = check(&[a, b], &config);
    assert!(
        !violations.iter().any(|v| v.message.contains("cycle")),
        "got {violations:?}"
    );
}

/// Even a target that happens to spell a loaded task's stem stays external: it
/// is validated for shape, never resolved into the graph.
#[test]
fn external_target_spelling_a_stem_is_still_shape_checked() {
    let config = config_with_external_kind(LinkKind {
        name: "discharged-by".to_owned(),
        ..LinkKind::default()
    });
    let mut a = bare_task("a");
    a.links = vec![edge("b.typ", "discharged-by")];
    let b = bare_task("b");
    let violations = check(&[a, b], &config);
    assert!(
        violations
            .iter()
            .any(|v| v.field == "discharged-by" && v.message.contains("url")),
        "got {violations:?}"
    );
}

/// A note rule still binds on an external kind — the escape hatch that lets a
/// project demand a reason for the artifact it points at.
#[test]
fn note_rule_still_applies_to_an_external_kind() {
    let config = config_with_external_kind(LinkKind {
        name: "discharged-by".to_owned(),
        note: NoteRule::Required,
        ..LinkKind::default()
    });
    let mut a = bare_task("a");
    a.links = vec![edge("https://example.com/pr/7", "discharged-by")];
    let violations = check(&[a], &config);
    assert!(
        violations
            .iter()
            .any(|v| v.field == "discharged-by" && v.message.contains("requires a note")),
        "got {violations:?}"
    );

    let mut ok = bare_task("a");
    ok.links = vec![noted(
        "https://example.com/pr/7",
        "discharged-by",
        "the PR that closed it",
    )];
    assert!(check(&[ok], &config).is_empty());
}

/// Cardinality counts task-graph edges, so an external kind's cap does not
/// fire — the axis excludes it along with resolution and cycles.
#[test]
fn external_kind_is_excluded_from_cardinality() {
    let config = config_with_external_kind(LinkKind {
        name: "discharged-by".to_owned(),
        max: Some(1),
        ..LinkKind::default()
    });
    let mut a = bare_task("a");
    a.links = vec![
        edge("https://example.com/pr/7", "discharged-by"),
        edge("https://example.com/pr/8", "discharged-by"),
    ];
    let violations = check(&[a], &config);
    assert!(violations.is_empty(), "got {violations:?}");
}

#[test]
fn custom_config_kind_resolves() {
    let config = config_with_custom_kind(LinkKind {
        name: "blocks-release".to_owned(),
        ..LinkKind::default()
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
        ..LinkKind::default()
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
        note: NoteRule::Forbidden,
        ..LinkKind::default()
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
        note: NoteRule::Forbidden,
        ..LinkKind::default()
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
