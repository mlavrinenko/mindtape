//! Every key a pack declares but its rules: deep-merged into a sub-layer
//! just below the layer that extends it, with two packs of one layer
//! refused when they set one value differently. The shared `layer` fixture
//! stays in the parent test module and is reached through `super::tests`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use super::merge_layers_with_packs_using;
use super::tests::layer;
use crate::config::layers::{LayerSource, load_config_layered_from};
use crate::config::packs_toml::read_array;
use crate::test_support::{home_and_project, tempdir, write_file};

#[test]
fn a_packs_priority_merges_below_the_layer_and_the_floor_fills_the_rest() {
    let (_tmp, home, project) = home_and_project();
    write_file(
        &project,
        "team.toml",
        "[tasks.priority]\nframework = \"ice\"\nallow-manual = false\n",
    );
    write_file(
        &project,
        ".mindtape/config.toml",
        "extends = [\"./team.toml\"]\n\n[tasks.priority]\nallow-manual = true\n",
    );

    let priority = load_config_layered_from(&project, Some(&home), None)
        .unwrap()
        .0
        .tasks
        .priority;
    assert_eq!(
        priority.framework.as_deref(),
        Some("ice"),
        "the pack's key stays"
    );
    assert!(priority.allow_manual, "the layer beats the pack");
    assert!(
        (priority.field.max - 9.0).abs() < f64::EPSILON,
        "the floor fills max"
    );
}

/// Merge the embedded floor's `floor`, then a project layer's `project`,
/// with `registry` to resolve their `extends` against.
fn merge_floor_and_project(
    registry: &[(&str, &str)],
    floor: &str,
    project: &str,
) -> super::PackMerge {
    let root = tempdir();
    let layers = [
        layer(LayerSource::Embedded, floor),
        layer(
            LayerSource::Project(PathBuf::from("/proj/.mindtape/config.toml")),
            project,
        ),
    ];
    merge_layers_with_packs_using(&layers, root.path(), registry).unwrap()
}

/// Merge one embedded layer extending every pack of `registry`, in order.
/// Shared with `config/packs_tests_extends.rs`.
pub(crate) fn merge_packs(
    registry: &[(&str, &str)],
) -> Result<super::PackMerge, crate::config::ConfigError> {
    merge_packs_under(registry, "")
}

/// [`merge_packs`], with `own` as the extending layer's own keys.
fn merge_packs_under(
    registry: &[(&str, &str)],
    own: &str,
) -> Result<super::PackMerge, crate::config::ConfigError> {
    let names: Vec<String> = registry
        .iter()
        .map(|(name, _)| format!("\"mindtape:{name}\""))
        .collect();
    let root = tempdir();
    let layers = [layer(
        LayerSource::Embedded,
        &format!("extends = [{}]\n\n{own}", names.join(", ")),
    )];
    merge_layers_with_packs_using(&layers, root.path(), registry)
}

#[test]
fn a_packs_value_beats_an_earlier_layer_but_not_its_own() {
    let merged = merge_floor_and_project(
        &[("test", "[a]\nx = 2\ny = 2\n")],
        "[a]\nx = 1\ny = 1\nz = 1\n",
        "extends = [\"mindtape:test\"]\n\n[a]\ny = 3\n",
    );
    let a = &merged.table["a"];
    assert_eq!(a["x"].as_integer(), Some(2), "the pack beats the floor");
    assert_eq!(a["y"].as_integer(), Some(3), "its layer beats the pack");
    assert_eq!(a["z"].as_integer(), Some(1), "the floor shows through");
}

#[test]
fn packs_of_two_layers_never_conflict_the_later_layers_wins() {
    let merged = merge_floor_and_project(
        &[("a", "a = 1\n"), ("b", "a = 2\n")],
        "extends = [\"mindtape:a\"]\n",
        "extends = [\"mindtape:b\"]\n",
    );
    assert_eq!(merged.table["a"].as_integer(), Some(2));
}

/// Merge one layer extending `mindtape:a` then `mindtape:b`.
fn merge_two_packs(a: &str, b: &str) -> Result<super::PackMerge, crate::config::ConfigError> {
    merge_packs(&[("a", a), ("b", b)])
}

#[test]
fn two_packs_setting_one_value_differently_conflict() {
    let err = merge_two_packs(
        "[tasks.priority]\nframework = \"ice\"\n",
        "[tasks.priority]\nframework = \"rice\"\n",
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid config: extends: packs 'mindtape:a' and 'mindtape:b' set different values for: \
         'tasks.priority.framework'"
    );
}

#[test]
fn a_table_and_a_scalar_at_one_key_conflict_at_that_key() {
    let err = merge_two_packs("[a]\nx = 1\n", "a = 2\n").unwrap_err();
    assert!(err.to_string().ends_with("for: 'a'"), "{err}");
}

#[test]
fn two_packs_setting_one_value_equally_agree() {
    let dict = "[dicts.source]\ncase = \"exact\"\nvalues = [\"crates/**\"]\n";
    let merged = merge_two_packs(
        &format!("{dict}\n[a]\nx = 1\n"),
        &format!("{dict}\n[a]\ny = 2\n"),
    )
    .unwrap();
    let values = merged.table["dicts"]["source"]["values"]
        .as_array()
        .unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(merged.table["a"]["x"].as_integer(), Some(1));
    assert_eq!(merged.table["a"]["y"].as_integer(), Some(2));
}

#[test]
fn two_packs_rules_sharing_a_name_still_replace_rather_than_conflict() {
    let rule = |message: &str| {
        format!("[[tasks.rules]]\nname = \"r\"\nassert = 'has(title)'\nmessage = \"{message}\"\n")
    };
    let merged = merge_two_packs(&rule("from-a"), &rule("from-b")).unwrap();
    let messages: Vec<_> = read_array(&merged.table, &["tasks", "rules"])
        .iter()
        .map(|rule| rule["message"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(messages, [Some("from-b".to_owned())]);
}

/// Two packs that set `tasks.priority.framework` and `a.x` differently.
const TWO_CONFLICTS: [(&str, &str); 2] = [
    ("a", "[tasks.priority]\nframework = \"ice\"\n\n[a]\nx = 1\n"),
    (
        "b",
        "[tasks.priority]\nframework = \"rice\"\n\n[a]\nx = 2\n",
    ),
];

#[test]
fn the_extending_layer_settles_a_conflict_by_setting_the_key() {
    let merged = merge_packs_under(
        &TWO_CONFLICTS,
        "[tasks.priority]\nframework = \"wsjf\"\n\n[a]\nx = 3\n",
    )
    .unwrap();
    assert_eq!(
        merged.table["tasks"]["priority"]["framework"].as_str(),
        Some("wsjf")
    );
    assert_eq!(merged.table["a"]["x"].as_integer(), Some(3));
}

#[test]
fn a_layer_scalar_over_a_conflicting_table_settles_it() {
    let merged =
        merge_packs_under(&[("a", "[a]\nx = 1\n"), ("b", "[a]\nx = 2\n")], "a = 0\n").unwrap();
    assert_eq!(merged.table["a"].as_integer(), Some(0));
}

#[test]
fn a_conflict_the_layer_leaves_open_still_fails_after_one_it_settles() {
    let err =
        merge_packs_under(&TWO_CONFLICTS, "[tasks.priority]\nframework = \"wsjf\"\n").unwrap_err();
    assert!(err.to_string().ends_with("for: 'a.x'"), "{err}");
}

#[test]
fn an_open_conflict_hints_at_setting_the_key_in_the_extending_layer() {
    let err = merge_packs_under(&TWO_CONFLICTS, "").unwrap_err();
    assert!(
        matches!(&err, crate::config::ConfigError::Invalid { hint: Some(hint), .. }
            if hint.contains("set the key")),
        "{err:?}"
    );
}
