//! `extends` resolution: a layer's `extends` list, resolved against an
//! injected pack registry so a test never needs a real embedded pack —
//! `source:pack` names, path entries, and the two keys a pack may not
//! declare. Split out of `config/packs_tests.rs`, which was near the linecop
//! cap; the shared `layer` fixture stays in the parent test module and is
//! reached through `super::tests`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests::layer;
use super::{merge_layers_with_packs, merge_layers_with_packs_using};
use crate::config::layers::LayerSource;
use crate::config::pack_registry::{PackOrigin, RuleOrigin};
use crate::config::packs_toml::read_array;
use crate::test_support::tempdir;
use crate::test_support::write_file;

/// The error `extends`-ing a registry of one pack, `name` holding `text`,
/// from the embedded layer gives.
fn extends_error(name: &str, text: &str) -> crate::config::ConfigError {
    let registry: &[(&str, &str)] = &[(name, text)];
    let root = tempdir();
    let layers = [layer(
        LayerSource::Embedded,
        &format!("extends = [\"mindtape:{name}\"]\n"),
    )];
    merge_layers_with_packs_using(&layers, root.path(), registry).unwrap_err()
}

#[test]
fn extends_source_colon_pack_resolves_to_an_embedded_pack() {
    let registry: &[(&str, &str)] = &[(
        "test",
        "[[tasks.rules]]\nname = \"from-pack\"\nassert = 'kind == \"bug\"'\n",
    )];
    let root = tempdir();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape:test\"]\n",
    )];

    let merged = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap();
    let rules = read_array(&merged.table, &["tasks", "rules"]);
    assert_eq!(
        rules[0].get("name").and_then(toml::Value::as_str),
        Some("from-pack")
    );
    match &merged.rule_origins[0].origin {
        RuleOrigin::Pack(PackOrigin::Embedded(name)) => assert_eq!(name, "mindtape:test"),
        other => panic!("expected a pack origin, got {other:?}"),
    }
}

#[test]
fn extends_unknown_pack_lists_what_the_source_holds() {
    let registry: &[(&str, &str)] = &[("test", "")];
    let root = tempdir();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape:nope\"]\n",
    )];

    let err = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("mindtape:nope"), "{rendered}");
    assert!(
        matches!(&err, crate::config::ConfigError::Invalid { unknown: Some(unknown), .. }
            if unknown.names == ["mindtape:test"]),
        "{err:?}"
    );
}

/// The real registry, not an injected one: `mindtape:tasks` is the pack the
/// binary ships, and the old `mindtape-tasks` name is refused with the new one
/// as the suggestion.
fn real_registry_error(entry: &str) -> crate::config::ConfigError {
    let root = tempdir();
    let layers = [layer(
        LayerSource::Embedded,
        &format!("extends = [\"{entry}\"]\n"),
    )];
    merge_layers_with_packs(&layers, root.path()).unwrap_err()
}

#[test]
fn mindtape_tasks_resolves_from_the_real_registry() {
    let root = tempdir();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape:tasks\"]\n",
    )];

    let merged = merge_layers_with_packs(&layers, root.path()).unwrap();
    assert!(!merged.rule_origins.is_empty());
    match &merged.rule_origins[0].origin {
        RuleOrigin::Pack(PackOrigin::Embedded(name)) => assert_eq!(name, "mindtape:tasks"),
        other => panic!("expected a pack origin, got {other:?}"),
    }
}

#[test]
fn the_old_mindtape_tasks_name_is_refused_with_the_new_one_suggested() {
    let err = real_registry_error("mindtape-tasks");

    assert!(err.to_string().contains("unknown pack"), "{err}");
    assert!(
        matches!(&err, crate::config::ConfigError::Invalid { unknown: Some(unknown), .. }
            if unknown.nearest.as_deref() == Some("mindtape:tasks")),
        "{err:?}"
    );
}

#[test]
fn an_unknown_pack_in_the_mindtape_source_lists_what_it_holds() {
    let err = real_registry_error("mindtape:nope");

    assert!(err.to_string().contains("mindtape:nope"), "{err}");
    assert!(
        matches!(&err, crate::config::ConfigError::Invalid { unknown: Some(unknown), .. }
            if unknown.names.contains(&"mindtape:tasks".to_owned())),
        "{err:?}"
    );
}

#[test]
fn an_undeclared_source_is_refused_with_the_known_sources_listed() {
    let err = real_registry_error("acme:commits");

    assert!(err.to_string().contains("unknown pack source"), "{err}");
    assert!(err.to_string().contains("acme"), "{err}");
    assert!(
        matches!(&err, crate::config::ConfigError::Invalid { unknown: Some(unknown), .. }
            if unknown.names == ["mindtape"]),
        "{err:?}"
    );
}

#[test]
fn extends_a_path_with_no_file_there_is_a_config_error() {
    let root = tempdir();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"nested/missing-pack.toml\"]\n",
    )];

    let err = merge_layers_with_packs_using(&layers, root.path(), &[]).unwrap_err();
    assert!(err.to_string().contains("missing-pack.toml"), "{err}");
}

#[test]
fn a_filename_with_no_separator_is_an_unknown_pack_not_a_missing_file() {
    // No path separator and no `<source>:` part, so this is no path: it fails
    // as an unknown pack, never as a missing file, though it looks like one.
    let root = tempdir();
    let layers = [layer(LayerSource::Embedded, "extends = [\"tasks.toml\"]\n")];

    let err = merge_layers_with_packs_using(&layers, root.path(), &[]).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("unknown pack"), "{rendered}");
    assert!(!rendered.contains("not found"), "{rendered}");
}

#[test]
fn a_path_entry_in_the_project_layer_resolves_against_the_project_root() {
    let tmp = tempdir();
    let project = tmp.path().join("project");
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    let shared = tmp.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    write_file(
        &shared,
        "team-pack.toml",
        "[[tasks.rules]]\nname = \"team-rule\"\nassert = 'kind == \"bug\"'\n",
    );

    let layers = [layer(
        LayerSource::Project(project.join(".mindtape").join("config.toml")),
        "extends = [\"../shared/team-pack.toml\"]\n",
    )];
    let merged = merge_layers_with_packs(&layers, &project).unwrap();
    let rules = read_array(&merged.table, &["tasks", "rules"]);
    assert_eq!(
        rules[0].get("name").and_then(toml::Value::as_str),
        Some("team-rule")
    );
}

#[test]
fn a_pack_declaring_extends_is_a_config_error() {
    let err = extends_error("nested", "extends = [\"mindtape:test\"]\n");
    let rendered = err.to_string();
    assert!(rendered.contains("mindtape:nested"), "{rendered}");
    assert!(rendered.contains("extends"), "{rendered}");
}

#[test]
fn a_pack_declaring_a_disallowed_key_is_a_config_error() {
    let err = extends_error("bad", "[project]\ncode = \"x\"\n");
    let rendered = err.to_string();
    assert!(rendered.contains("mindtape:bad"), "{rendered}");
    assert!(rendered.contains("project"), "{rendered}");
}

/// `[[commits.rules]]` merges by name like the task rule arrays: a pack's
/// rule is replaced in place by a layer's same-named one, and `disable`
/// drops another.
#[test]
fn commit_rules_merge_by_name_across_packs() {
    let registry: &[(&str, &str)] = &[(
        "test",
        "[[commits.rules]]\nname = \"names-a-task\"\nassert = 'has(refs)'\n\n\
         [[commits.rules]]\nname = \"subject-set\"\nassert = 'has(subject)'\n",
    )];
    let root = tempdir();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape:test\"]\ndisable = [\"subject-set\"]\n\n\
         [[commits.rules]]\nname = \"names-a-task\"\nwhen = '!tasks_only'\nassert = 'has(refs)'\n",
    )];

    let merged = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap();
    let rules = read_array(&merged.table, &["commits", "rules"]);
    assert_eq!(rules.len(), 1, "{rules:?}");
    assert_eq!(
        rules[0].get("when").and_then(toml::Value::as_str),
        Some("!tasks_only"),
        "the layer's same-named rule replaces the pack's"
    );
    let labels: Vec<&str> = merged
        .rule_origins
        .iter()
        .map(|entry| entry.label.as_str())
        .collect();
    assert_eq!(labels, ["commits.rules.names-a-task"]);
}

#[test]
fn a_pack_may_declare_only_rules_under_commits() {
    let err = extends_error("test", "[commits]\ntrailer = \"Refs\"\n");
    assert!(err.to_string().contains("'trailer'"), "{err}");
}

#[test]
fn a_syntax_error_in_an_embedded_pack_names_the_pack() {
    let err = extends_error("broken", "oops =\n");

    assert!(
        err.to_string()
            .starts_with("parse error in pack mindtape:broken: "),
        "{err}"
    );
}

#[test]
fn a_syntax_error_in_a_global_layer_names_its_file() {
    let root = tempdir();
    let path = root
        .path()
        .join("cfg")
        .join("mindtape")
        .join("defaults.toml");
    let layers = [layer(LayerSource::Xdg(path.clone()), "oops =\n")];

    let err = merge_layers_with_packs_using(&layers, root.path(), &[]).unwrap_err();

    assert!(
        err.to_string()
            .starts_with(&format!("parse error in {}: ", path.display())),
        "{err}"
    );
}
