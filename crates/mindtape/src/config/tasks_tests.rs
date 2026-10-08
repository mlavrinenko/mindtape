#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
use std::collections::BTreeMap;

use super::{FieldConfig, TagsConfig, TasksConfig, expand_dict_refs};
use crate::config::dicts::{DictSpec, Dicts, compile_dicts_rootless};

/// Compile a `Dicts` from `(name, values)` pairs — inline `values` only,
/// which needs no project root.
fn dicts(spec: &[(&str, &[&str])]) -> Dicts {
    let map: BTreeMap<String, DictSpec> = spec
        .iter()
        .map(|(name, values)| {
            let spec = DictSpec {
                values: values.iter().map(|v| (*v).to_owned()).collect(),
                ..DictSpec::default()
            };
            ((*name).to_owned(), spec)
        })
        .collect();
    compile_dicts_rootless(&map).expect("inline-only dicts always compile")
}

/// No `@` entry anywhere leaves both lists exactly as declared.
#[test]
fn a_list_with_no_at_refs_is_untouched() {
    let mut tasks = TasksConfig {
        tags: TagsConfig {
            allowed: vec!["swept".to_owned(), "size".to_owned()],
            ..TagsConfig::default()
        },
        ..TasksConfig::default()
    };
    expand_dict_refs(&mut tasks, &dicts(&[])).unwrap();
    assert_eq!(tasks.tags.allowed, ["swept", "size"]);
}

/// `[tasks.tags] allowed`'s expansion applies equally to a
/// `[tasks.fields.<name>] values` list.
#[test]
fn a_dict_ref_in_a_field_s_values_expands() {
    let mut tasks = TasksConfig::default();
    tasks.fields.insert(
        "kind".to_owned(),
        FieldConfig {
            values: vec!["@kinds".to_owned()],
            ..FieldConfig::default()
        },
    );
    expand_dict_refs(&mut tasks, &dicts(&[("kinds", &["bug", "chore"])])).unwrap();
    assert_eq!(tasks.fields["kind"].values, ["bug", "chore"]);
}

/// An unknown ref in `tags.allowed` and one in a field's `values` are
/// both reported, joined into the one returned message.
#[test]
fn unknown_refs_in_both_lists_are_joined_into_one_error() {
    let mut tasks = TasksConfig {
        tags: TagsConfig {
            allowed: vec!["@ghost".to_owned()],
            ..TagsConfig::default()
        },
        ..TasksConfig::default()
    };
    tasks.fields.insert(
        "kind".to_owned(),
        FieldConfig {
            values: vec!["@phantom".to_owned()],
            ..FieldConfig::default()
        },
    );
    let err = expand_dict_refs(&mut tasks, &dicts(&[])).unwrap_err();
    assert!(
        err.contains("tasks.tags.allowed: unknown dict: 'ghost'"),
        "{err}"
    );
    assert!(
        err.contains("tasks.fields.kind.values: unknown dict: 'phantom'"),
        "{err}"
    );
}
