#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};

use super::{PackOrigin, RuleOrigin, merge_layers_with_packs, merge_layers_with_packs_using};
use crate::config_layers::{Layer, LayerSource, load_config_layered_from};
use crate::config_packs_toml::read_array;
use crate::test_support::home_and_project;

fn names_of(rules: &[crate::config_rules::RuleSpec]) -> Vec<Option<String>> {
    rules.iter().map(|r| r.name.clone()).collect()
}

// --- the task's Problem steps: an ancestor rule vanishing, fixed ---

#[test]
fn ancestor_rule_survives_when_project_adds_its_own() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(home.join(".mindtape")).unwrap();
    std::fs::write(
        home.join(".mindtape").join("defaults.toml"),
        "[[tasks.rules]]\nname = \"ancestor-rule\"\nassert = 'kind == \"bug\"'\n",
    )
    .unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[[tasks.rules]]\nname = \"project-rule\"\nassert = 'kind == \"bug\"'\n",
    )
    .unwrap();

    let (config, _sources, _origins) =
        load_config_layered_from(&project, Some(&home), None).unwrap();
    assert_eq!(
        names_of(&config.tasks.rules),
        vec![
            Some("ancestor-rule".to_owned()),
            Some("project-rule".to_owned())
        ]
    );
}

#[test]
fn same_name_project_rule_replaces_the_ancestors_in_place() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(home.join(".mindtape")).unwrap();
    std::fs::write(
        home.join(".mindtape").join("defaults.toml"),
        "[[tasks.rules]]\nname = \"shared\"\nassert = 'kind == \"bug\"'\nmessage = \"from-ancestor\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[[tasks.rules]]\nname = \"shared\"\nassert = 'kind == \"bug\"'\nmessage = \"from-project\"\n",
    )
    .unwrap();

    let (config, _sources, _origins) =
        load_config_layered_from(&project, Some(&home), None).unwrap();
    assert_eq!(config.tasks.rules.len(), 1);
    assert_eq!(
        config.tasks.rules[0].message.as_deref(),
        Some("from-project")
    );
}

#[test]
fn disable_removes_an_inherited_rule() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(home.join(".mindtape")).unwrap();
    std::fs::write(
        home.join(".mindtape").join("defaults.toml"),
        "[[tasks.rules]]\nname = \"foo\"\nassert = 'kind == \"bug\"'\n",
    )
    .unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "disable = [\"foo\"]\n",
    )
    .unwrap();

    let (config, _sources, _origins) =
        load_config_layered_from(&project, Some(&home), None).unwrap();
    assert!(names_of(&config.tasks.rules).is_empty());
}

#[test]
fn disable_naming_no_rule_anywhere_is_a_config_error() {
    let (_tmp, home, project) = home_and_project();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "disable = [\"nope\"]\n",
    )
    .unwrap();

    let err = load_config_layered_from(&project, Some(&home), None).unwrap_err();
    assert!(err.to_string().contains("nope"), "{err}");
}

// --- extends resolution, registry-injected so a test never needs a real
// embedded pack ---

fn layer(source: LayerSource, text: &str) -> Layer {
    Layer {
        source,
        text: text.to_owned(),
    }
}

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

// --- a `[[tasks.rules]] check` entry validates and resolves the same way
// regardless of which layer or pack declared it: against the PROJECT root,
// never the declaring layer's own directory (`crates/mindtape/src/
// config_rules_typst.rs::validate_load`, run from `config_layers::build_config`
// over the fully merged `config.tasks.rules`, so a pack's or an ancestor's
// `check` entry is exactly one more entry in that array by the time it runs).

/// The `[[tasks.rules]] check` entry every `an_ancestor_layers_check_rule_*`
/// test declares in the ancestor's own `defaults.toml`: `checks.typ`'s
/// `check-thing`.
const ANCESTOR_CHECK_RULE: &str = "[[tasks.rules]]\nname = \"ancestor-check\"\n\
     check = { typst = \"checks.typ\", function = \"check-thing\" }\n";

#[test]
fn an_ancestor_layers_check_rule_resolves_against_the_project_root_not_the_ancestors() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(home.join(".mindtape")).unwrap();
    std::fs::write(
        home.join(".mindtape").join("defaults.toml"),
        ANCESTOR_CHECK_RULE,
    )
    .unwrap();
    // A decoy at the ANCESTOR's own directory that does not export the
    // function — if load ever resolved `typst` against the declaring layer
    // instead of the project root, this is what it would find and fail on.
    std::fs::write(home.join("checks.typ"), "#let something-else() = ()\n").unwrap();
    // The real module lives at the project root instead.
    std::fs::write(project.join("checks.typ"), "#let check-thing(value) = ()\n").unwrap();

    let result = load_config_layered_from(&project, Some(&home), None);
    assert!(
        result.is_ok(),
        "an ancestor's check rule must resolve `typst` against the project \
         root: {:?}",
        result.err()
    );
}

#[test]
fn an_ancestor_layers_check_rule_still_fails_load_when_absent_at_the_project_root() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(home.join(".mindtape")).unwrap();
    std::fs::write(
        home.join(".mindtape").join("defaults.toml"),
        ANCESTOR_CHECK_RULE,
    )
    .unwrap();
    // Only at the ancestor's own directory, proving that is not a fallback
    // location and load-time validation genuinely runs for this rule.
    std::fs::write(home.join("checks.typ"), "#let check-thing(value) = ()\n").unwrap();

    let err = load_config_layered_from(&project, Some(&home), None).unwrap_err();
    assert!(err.to_string().contains("checks.typ"), "{err}");
}

/// Build the typed [`crate::config::Config`] `project`'s own
/// `.mindtape/config.toml` extending `registry`'s `"mindtape-test"` pack (and
/// nothing else) would produce — merge, then the same typed-deserialization
/// tail [`crate::config_layers::build_config`] runs after its own merge — for
/// a test that needs the pack registry injected, which
/// `load_config_layered_from`'s real (possibly empty) [`super::EMBEDDED_PACKS`]
/// cannot give it.
fn config_extending_test_pack(project: &Path, registry: &[(&str, &str)]) -> crate::config::Config {
    let merged = merge_layers_with_packs_using(
        &[layer(
            LayerSource::Project(project.join(".mindtape").join("config.toml")),
            "extends = [\"mindtape-test\"]\n",
        )],
        project,
        registry,
    )
    .unwrap();
    let mut doc = merged.table;
    crate::framework::merge_builtin_frameworks(&mut doc);
    toml::Value::Table(doc).try_into().unwrap()
}

#[test]
fn a_packs_check_rule_is_validated_at_load_against_the_project_root() {
    let registry: &[(&str, &str)] = &[(
        "mindtape-test",
        "[[tasks.rules]]\nname = \"pack-check\"\n\
         check = { typst = \"missing.typ\", function = \"f\" }\n",
    )];
    let (_tmp, _home, project) = home_and_project();
    let config = config_extending_test_pack(&project, registry);

    let err = crate::config_rules_typst::validate_load(&config, &project).unwrap_err();
    assert!(err.contains("missing.typ"), "{err}");
}

#[test]
fn a_pack_declaring_two_same_named_rules_is_a_config_error() {
    let registry: &[(&str, &str)] = &[(
        "mindtape-test",
        "[[tasks.rules]]\nname = \"dup\"\nassert = 'kind == \"bug\"'\n\n\
         [[tasks.rules]]\nname = \"dup\"\nassert = 'kind == \"chore\"'\n",
    )];
    let (_tmp, _home, project) = home_and_project();
    let config = config_extending_test_pack(&project, registry);

    let err = config.validate().unwrap_err();
    assert!(err.contains("tasks.rules") && err.contains("dup"), "{err}");
}

#[test]
fn pack_rules_come_before_the_layers_own_in_the_merged_array() {
    let registry: &[(&str, &str)] = &[(
        "mindtape-test",
        "[[tasks.rules]]\nassert = 'kind == \"bug\"'\nmessage = \"from-pack\"\n",
    )];
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape-test\"]\n\n[[tasks.rules]]\nassert = 'kind == \"bug\"'\nmessage = \"from-layer\"\n",
    )];

    let merged = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap();
    let rules = read_array(&merged.table, &["tasks", "rules"]);
    assert_eq!(
        rules[0].get("message").and_then(toml::Value::as_str),
        Some("from-pack")
    );
    assert_eq!(
        rules[1].get("message").and_then(toml::Value::as_str),
        Some("from-layer")
    );
}

#[test]
fn a_packs_dicts_merge_in_and_the_layers_own_entry_wins_on_a_shared_name() {
    let registry: &[(&str, &str)] = &[(
        "mindtape-test",
        "[dicts.verbs]\nvalues = [\"ship\"]\n\n[dicts.nouns]\nvalues = [\"task\"]\n",
    )];
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "extends = [\"mindtape-test\"]\n\n[dicts.verbs]\nvalues = [\"ride\"]\n",
    )];

    let merged = merge_layers_with_packs_using(&layers, root.path(), registry).unwrap();
    let dicts = merged.table["dicts"].as_table().unwrap();
    assert_eq!(
        dicts["verbs"]["values"].as_array().unwrap()[0].as_str(),
        Some("ride")
    );
    assert_eq!(
        dicts["nouns"]["values"].as_array().unwrap()[0].as_str(),
        Some("task")
    );
}

#[test]
fn a_same_file_duplicate_name_is_left_for_validation_not_collapsed_at_merge() {
    // Two entries sharing a name WITHIN one layer's own array are an
    // authoring mistake `validate_header_rules` catches downstream (see
    // `config_tests_headers.rs::rejects_duplicate_header_rule_names`) — the
    // merge itself must not silently keep only one, or that check could never
    // fire again.
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(
        LayerSource::Embedded,
        "[[tasks.body.headers.rules]]\nname = \"dup\"\npath = [\"A\"]\naction = \"require\"\n\n\
         [[tasks.body.headers.rules]]\nname = \"dup\"\npath = [\"B\"]\naction = \"require\"\n",
    )];

    let merged = merge_layers_with_packs_using(&layers, root.path(), &[]).unwrap();
    let rules = read_array(&merged.table, &["tasks", "body", "headers", "rules"]);
    assert_eq!(rules.len(), 2);
}

/// The full layered load actually refuses the scenario the merge-level test
/// above only shows survives the merge: two `[[tasks.body.headers.rules]]`
/// entries sharing a name within the project's own file fail
/// `Config::validate` (`validate_header_rules`), the same treatment
/// `rejects_duplicate_rule_names_within_one_array`
/// (`config_rules_tests.rs`) now gives `[[tasks.rules]]`. Cross-file
/// replacement by name is a different scenario entirely — see
/// `same_name_project_rule_replaces_the_ancestors_in_place` — and stays
/// legal.
#[test]
fn a_same_file_duplicate_header_rule_name_fails_the_full_load() {
    let (_tmp, home, project) = home_and_project();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[[tasks.body.headers.rules]]\nname = \"dup\"\npath = [\"A\"]\naction = \"require\"\n\n\
         [[tasks.body.headers.rules]]\nname = \"dup\"\npath = [\"B\"]\naction = \"require\"\n",
    )
    .unwrap();

    let err = load_config_layered_from(&project, Some(&home), None).unwrap_err();
    assert!(err.to_string().contains("dup"), "{err}");
}

// --- everything else: still a plain wholesale-replace fold, unaffected by
// packs — the tests `config_layers.rs::merge_layers` used to carry, now
// against this module's own fold (`merge_layers_with_packs_using` with an
// empty registry is exactly that fold, for any field but the three rule
// arrays) ---

#[test]
fn a_scalar_field_wholesale_replaces_nearest_dir_over_farther_dir() {
    let root = tempfile::tempdir().unwrap();
    let layers = [
        layer(LayerSource::Embedded, "[a]\nx = 1\n"),
        layer(LayerSource::Dir(PathBuf::from("/far")), "[a]\nx = 2\n"),
        layer(LayerSource::Dir(PathBuf::from("/near")), "[a]\nx = 3\n"),
    ];
    let merged = merge_layers_with_packs_using(&layers, root.path(), &[]).unwrap();
    assert_eq!(merged.table["a"]["x"].as_integer(), Some(3));
}

#[test]
fn a_scalar_field_wholesale_replaces_project_over_every_other_layer() {
    let root = tempfile::tempdir().unwrap();
    let layers = [
        layer(LayerSource::Embedded, "[a]\nx = 1\n"),
        layer(LayerSource::Xdg(PathBuf::from("/xdg")), "[a]\nx = 2\n"),
        layer(LayerSource::Dir(PathBuf::from("/dir")), "[a]\nx = 3\n"),
        layer(LayerSource::Project(PathBuf::from("/proj")), "[a]\nx = 4\n"),
    ];
    let merged = merge_layers_with_packs_using(&layers, root.path(), &[]).unwrap();
    assert_eq!(merged.table["a"]["x"].as_integer(), Some(4));
}

#[test]
fn a_scalar_field_wholesale_replaces_xdg_over_embedded_but_loses_to_dir() {
    let root = tempfile::tempdir().unwrap();
    let embedded_and_xdg = vec![
        layer(LayerSource::Embedded, "[a]\nx = 1\n"),
        layer(LayerSource::Xdg(PathBuf::from("/xdg")), "[a]\nx = 2\n"),
    ];
    // XDG alone beats the embedded floor.
    let merged = merge_layers_with_packs_using(&embedded_and_xdg, root.path(), &[]).unwrap();
    assert_eq!(merged.table["a"]["x"].as_integer(), Some(2));

    // But a directory-scoped layer, being more specific, wins over XDG.
    let mut with_dir = embedded_and_xdg;
    with_dir.push(layer(
        LayerSource::Dir(PathBuf::from("/dir")),
        "[a]\nx = 3\n",
    ));
    let merged = merge_layers_with_packs_using(&with_dir, root.path(), &[]).unwrap();
    assert_eq!(merged.table["a"]["x"].as_integer(), Some(3));
}

#[test]
fn merging_rejects_invalid_toml_in_any_layer() {
    let root = tempfile::tempdir().unwrap();
    let layers = [layer(LayerSource::Embedded, "not = valid = toml")];
    assert!(merge_layers_with_packs_using(&layers, root.path(), &[]).is_err());
}
