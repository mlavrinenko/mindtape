//! Link kinds that point outside the backlog (`validate = "url"`): their
//! targets are shape-checked and never resolved, cycle-checked or counted.

#![allow(clippy::unwrap_used)]

use super::tests::{check, config_with_custom_kind, edge, noted};
use crate::config::Config;
use crate::link::{LinkKind, NoteRule, TargetValidator};
use crate::test_support::{bare_task, linked_task};

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
    let a = linked_task("a", vec![edge("https://example.com/pr/7", "discharged-by")]);
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
    let a = linked_task("a", vec![edge("not a url", "discharged-by")]);
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
    let a = linked_task("a", vec![edge("https://example.com/a", "mirrors")]);
    let b = linked_task("b", vec![edge("https://example.com/b", "mirrors")]);
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
    let a = linked_task("a", vec![edge("b.typ", "discharged-by")]);
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
    let a = linked_task("a", vec![edge("https://example.com/pr/7", "discharged-by")]);
    let violations = check(&[a], &config);
    assert!(
        violations
            .iter()
            .any(|v| v.field == "discharged-by" && v.message.contains("requires a note")),
        "got {violations:?}"
    );

    let ok = linked_task(
        "a",
        vec![noted(
            "https://example.com/pr/7",
            "discharged-by",
            "the PR that closed it",
        )],
    );
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
    let a = linked_task(
        "a",
        vec![
            edge("https://example.com/pr/7", "discharged-by"),
            edge("https://example.com/pr/8", "discharged-by"),
        ],
    );
    let violations = check(&[a], &config);
    assert!(violations.is_empty(), "got {violations:?}");
}
