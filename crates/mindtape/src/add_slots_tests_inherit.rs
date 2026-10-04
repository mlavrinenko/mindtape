//! Tests for the choices a template's own slot inherits from config.
#![allow(clippy::unwrap_used)]

use crate::config::parse_config;
use crate::template::{SlotKind, discover_slots};

use super::inherit_field_choices;

const KIND: &str = "[tasks.fields.kind]\nvalues = [\"feat\", \"test\"]\n";

/// The `(kind, choices)` of the slot named `name` once `template` inherits
/// from `toml`.
fn inherited(toml: &str, template: &str, name: &str) -> (SlotKind, Option<Vec<String>>) {
    let config = parse_config(toml).unwrap();
    let source = inherit_field_choices(template, &config).unwrap();
    let spec = discover_slots(&source)
        .into_iter()
        .find(|spec| spec.name == name)
        .unwrap();
    (spec.kind, spec.choices)
}

fn feat_test() -> Vec<String> {
    vec!["feat".to_owned(), "test".to_owned()]
}

#[test]
fn a_field_s_own_slot_becomes_a_choice_over_its_values() {
    let template = r#"#show: task.with(kind: slot("kind", example: "test"))"#;
    assert_eq!(
        inherited(KIND, template, "kind"),
        (SlotKind::Choice, Some(feat_test()))
    );
}

#[test]
fn the_field_is_the_argument_the_slot_sits_in_not_the_slot_s_name() {
    let template = r#"#show: task.with(kind: slot("type"), title: slot("kind"))"#;
    assert_eq!(
        inherited(KIND, template, "type"),
        (SlotKind::Choice, Some(feat_test()))
    );
    assert_eq!(inherited(KIND, template, "kind"), (SlotKind::String, None));
}

#[test]
fn a_declared_string_or_choice_kind_inherits_too() {
    for kind in ["string", "choice"] {
        let template = format!(r#"#show: task.with(kind: slot("kind", kind: "{kind}"))"#);
        assert_eq!(
            inherited(KIND, &template, "kind"),
            (SlotKind::Choice, Some(feat_test()))
        );
    }
}

#[test]
fn a_slot_s_own_choices_win() {
    let template = r#"#show: task.with(kind: slot("kind", kind: "choice", choices: ("feat",)))"#;
    assert_eq!(
        inherited(KIND, template, "kind"),
        (SlotKind::Choice, Some(vec!["feat".to_owned()]))
    );
}

#[test]
fn another_kind_or_a_nested_slot_is_left_alone() {
    let toml = "[tasks.fields.x-ns]\nvalues = [\"a\", \"b\"]\n";
    let template = r#"#show: task.with(x-ns: slot("x-ns", kind: "raw"))"#;
    assert_eq!(inherited(toml, template, "x-ns"), (SlotKind::Raw, None));
    let template = r#"#show: task.with(x-ns: (inner: slot("inner")))"#;
    assert_eq!(inherited(toml, template, "inner"), (SlotKind::String, None));
}

#[test]
fn an_extension_field_inherits_its_values() {
    let toml = "[tasks.fields.x-machine]\nvalues = [\"alpha\", \"desk\"]\n";
    let template = r#"#show: task.with(x-machine: slot("machine"))"#;
    assert_eq!(
        inherited(toml, template, "machine"),
        (
            SlotKind::Choice,
            Some(vec!["alpha".to_owned(), "desk".to_owned()])
        )
    );
}

#[test]
fn a_field_without_values_or_a_source_without_task_with_is_unchanged() {
    let config = parse_config(KIND).unwrap();
    for source in [
        r#"#show: task.with(title: slot("title"))"#,
        r#"#let x = slot("kind")"#,
    ] {
        assert_eq!(inherit_field_choices(source, &config).unwrap(), source);
    }
}

#[test]
fn an_assignee_slot_offers_the_allowed_names() {
    let template = r#"#show: task.with(assignee: slot("assignee"))"#;
    let toml = "[tasks.assignee]\nallowed = [\"ada\", \"lin\"]\n";
    assert_eq!(
        inherited(toml, template, "assignee"),
        (
            SlotKind::Choice,
            Some(vec!["ada".to_owned(), "lin".to_owned()])
        )
    );
}

#[test]
fn a_tags_slot_keeps_its_own_spec() {
    let template = r#"#show: task.with(tags: slot("tags"))"#;
    let toml = "[tasks.tags]\nallowed = [\"web\"]\n";
    assert_eq!(inherited(toml, template, "tags"), (SlotKind::String, None));
}
