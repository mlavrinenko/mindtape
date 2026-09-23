//! `extends` resolution: a layer's `extends` list, resolved against an
//! injected pack registry so a test never needs a real embedded pack — bare
//! names, path entries, the reserved prefix, and the two keys a pack may not
//! declare. Split out of `config_packs_tests.rs`, which was near the linecop
//! cap; the shared `layer` fixture stays in the parent test module and is
//! reached through `super::tests`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests::layer;
use super::{PackOrigin, RuleOrigin, merge_layers_with_packs, merge_layers_with_packs_using};
use crate::config_layers::LayerSource;
use crate::config_packs_toml::read_array;

#[test]
fn extends_bare_name_resolves_to_an_embedded_pack() {
    let registry: &[(&str, &str)] = &[(
        "mindtape-test",
        "[[tasks.rules]]\nname = \"from-pack\"\nassert = 'kind == \"bug\"'\n",
    )];
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape-test\"]\n",
    )];

    let merged = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap();
    let rules = read_array(&merged.table, &["tasks", "rules"]);
    assert_eq!(
        rules[0].get("name").and_then(toml::Value::as_str),
        Some("from-pack")
    );
    match &merged.rule_origins[0].origin {
        RuleOrigin::Pack(PackOrigin::Embedded(name)) => assert_eq!(name, "mindtape-test"),
        other => panic!("expected a pack origin, got {other:?}"),
    }
}

#[test]
fn extends_unknown_bare_name_lists_available_packs() {
    let registry: &[(&str, &str)] = &[("mindtape-test", "")];
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape-nope\"]\n",
    )];

    let err = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("mindtape-nope"), "{rendered}");
    assert!(rendered.contains("mindtape-test"), "{rendered}");
}

#[test]
fn extends_a_path_with_no_file_there_is_a_config_error() {
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"nested/missing-pack.toml\"]\n",
    )];

    let err = merge_layers_with_packs_using(&layers, root.path(), &[]).unwrap_err();
    assert!(err.to_string().contains("missing-pack.toml"), "{err}");
}

#[test]
fn a_mindtape_prefixed_name_with_no_separator_is_still_a_bare_name() {
    // No path separator, so this is a bare-name lookup — and fails as an
    // unknown pack, never as a missing file, even though it carries the
    // reserved prefix and looks like a filename.
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape-tasks.toml\"]\n",
    )];

    let err = merge_layers_with_packs_using(&layers, root.path(), &[]).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("unknown pack"), "{rendered}");
    assert!(!rendered.contains("not found"), "{rendered}");
}

#[test]
fn a_path_entry_in_the_project_layer_resolves_against_the_project_root() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("project");
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    let shared = tmp.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(
        shared.join("team-pack.toml"),
        "[[tasks.rules]]\nname = \"team-rule\"\nassert = 'kind == \"bug\"'\n",
    )
    .unwrap();

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
    let registry: &[(&str, &str)] = &[("mindtape-nested", "extends = [\"mindtape-test\"]\n")];
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape-nested\"]\n",
    )];

    let err = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("mindtape-nested"), "{rendered}");
    assert!(rendered.contains("extends"), "{rendered}");
}

#[test]
fn a_pack_declaring_a_disallowed_key_is_a_config_error() {
    let registry: &[(&str, &str)] = &[("mindtape-bad", "[project]\ncode = \"x\"\n")];
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape-bad\"]\n",
    )];

    let err = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("mindtape-bad"), "{rendered}");
    assert!(rendered.contains("project"), "{rendered}");
}
