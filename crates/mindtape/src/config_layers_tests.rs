#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use super::{
    Layer, LayerSource, build_config, discover_layers_with_xdg, merge_layers, merged_config_table,
};
use crate::config::parse_config;

fn layer(source: LayerSource, text: &str) -> Layer {
    Layer {
        source,
        text: text.to_owned(),
    }
}

/// A fresh tempdir with an (empty) `home/project` tree already created. Keep
/// the returned `TempDir` alive for as long as the paths are used.
fn home_and_project() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let project = home.join("project");
    std::fs::create_dir_all(&project).unwrap();
    (tmp, home, project)
}

// --- merge_layers: pure precedence folding, no IO ---

#[test]
fn merge_layers_nearest_dir_overrides_farther_dir() {
    let layers = vec![
        layer(LayerSource::Embedded, "[a]\nx = 1\n"),
        layer(LayerSource::Dir(PathBuf::from("/far")), "[a]\nx = 2\n"),
        layer(LayerSource::Dir(PathBuf::from("/near")), "[a]\nx = 3\n"),
    ];
    let merged = merge_layers(&layers).unwrap();
    assert_eq!(merged["a"]["x"].as_integer(), Some(3));
}

#[test]
fn merge_layers_project_overrides_all_defaults() {
    let layers = vec![
        layer(LayerSource::Embedded, "[a]\nx = 1\n"),
        layer(LayerSource::Xdg(PathBuf::from("/xdg")), "[a]\nx = 2\n"),
        layer(LayerSource::Dir(PathBuf::from("/dir")), "[a]\nx = 3\n"),
        layer(LayerSource::Project(PathBuf::from("/proj")), "[a]\nx = 4\n"),
    ];
    let merged = merge_layers(&layers).unwrap();
    assert_eq!(merged["a"]["x"].as_integer(), Some(4));
}

#[test]
fn merge_layers_xdg_overrides_embedded_but_loses_to_dir() {
    let embedded_and_xdg = vec![
        layer(LayerSource::Embedded, "[a]\nx = 1\n"),
        layer(LayerSource::Xdg(PathBuf::from("/xdg")), "[a]\nx = 2\n"),
    ];
    // XDG alone beats the embedded floor.
    let merged = merge_layers(&embedded_and_xdg).unwrap();
    assert_eq!(merged["a"]["x"].as_integer(), Some(2));

    // But a directory-scoped layer, being more specific, wins over XDG.
    let mut with_dir = embedded_and_xdg;
    with_dir.push(layer(
        LayerSource::Dir(PathBuf::from("/dir")),
        "[a]\nx = 3\n",
    ));
    let merged = merge_layers(&with_dir).unwrap();
    assert_eq!(merged["a"]["x"].as_integer(), Some(3));
}

#[test]
fn merge_layers_rejects_invalid_toml() {
    let layers = vec![layer(LayerSource::Embedded, "not = valid = toml")];
    assert!(merge_layers(&layers).is_err());
}

// --- discover_layers: filesystem walk-up + XDG + project, ordering ---

#[test]
fn discover_layers_orders_farthest_ancestor_first_and_respects_home_boundary() {
    let tmp = tempfile::tempdir().unwrap();
    let above_home = tmp.path().join("above-home");
    let home = tmp.path().join("home");
    let parent = home.join("parent");
    let project = parent.join("project");

    // Above $HOME: must NOT be read.
    std::fs::create_dir_all(above_home.join(".mindtape")).unwrap();
    std::fs::write(
        above_home.join(".mindtape").join("defaults.toml"),
        "[marker]\nwhich = \"above-home\"\n",
    )
    .unwrap();

    // At $HOME itself: inclusive boundary, must be read (farthest).
    std::fs::create_dir_all(home.join(".mindtape")).unwrap();
    std::fs::write(
        home.join(".mindtape").join("defaults.toml"),
        "[marker]\nwhich = \"home\"\n",
    )
    .unwrap();

    // An intermediate ancestor between $HOME and the project.
    std::fs::create_dir_all(parent.join(".mindtape")).unwrap();
    std::fs::write(
        parent.join(".mindtape").join("defaults.toml"),
        "[marker]\nwhich = \"parent\"\n",
    )
    .unwrap();

    // The project itself: both a defaults.toml (nearest Dir layer) and a
    // config.toml (the highest-precedence Project layer).
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("defaults.toml"),
        "[marker]\nwhich = \"project-defaults\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[marker]\nwhich = \"project-config\"\n",
    )
    .unwrap();

    let layers = discover_layers_with_xdg(&project, Some(&home), None);

    // Embedded, then the three Dir layers farthest-to-nearest, then Project.
    // No Xdg layer (none injected) and nothing from above $HOME.
    assert_eq!(layers.len(), 5);
    assert_eq!(layers[0].source, LayerSource::Embedded);

    let which = |idx: usize| -> String {
        let doc: toml::Table = toml::from_str(&layers[idx].text).unwrap();
        doc["marker"]["which"].as_str().unwrap().to_owned()
    };
    assert_eq!(which(1), "home");
    assert_eq!(which(2), "parent");
    assert_eq!(which(3), "project-defaults");
    assert_eq!(which(4), "project-config");

    assert!(matches!(layers[1].source, LayerSource::Dir(_)));
    assert!(matches!(layers[2].source, LayerSource::Dir(_)));
    assert!(matches!(layers[3].source, LayerSource::Dir(_)));
    assert!(matches!(layers[4].source, LayerSource::Project(_)));

    // Sanity: the "above home" text never appears anywhere in the layer set.
    assert!(
        layers
            .iter()
            .all(|layer| !layer.text.contains("above-home"))
    );
}

#[test]
fn discover_layers_missing_xdg_and_missing_dir_defaults_are_skipped_silently() {
    let (tmp, home, project) = home_and_project();

    // No `.mindtape` anywhere, and an XDG dir that itself has no
    // `mindtape/defaults.toml` inside it.
    let xdg = tmp.path().join("xdg-config");
    std::fs::create_dir_all(&xdg).unwrap();

    let layers = discover_layers_with_xdg(&project, Some(&home), Some(&xdg));
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].source, LayerSource::Embedded);
}

#[test]
fn discover_layers_missing_xdg_dir_entirely_is_skipped_silently() {
    let (_tmp, home, project) = home_and_project();

    let layers = discover_layers_with_xdg(&project, Some(&home), None);
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].source, LayerSource::Embedded);
}

#[test]
fn discover_layers_reads_xdg_defaults_when_present() {
    let (tmp, home, project) = home_and_project();

    let xdg = tmp.path().join("xdg-config");
    std::fs::create_dir_all(xdg.join("mindtape")).unwrap();
    std::fs::write(xdg.join("mindtape").join("defaults.toml"), "[a]\nx = 9\n").unwrap();

    let layers = discover_layers_with_xdg(&project, Some(&home), Some(&xdg));
    assert_eq!(layers.len(), 2);
    assert!(matches!(layers[1].source, LayerSource::Xdg(_)));
}

#[cfg(unix)]
#[test]
fn dir_layers_dedupes_via_canonicalized_symlinked_home() {
    let tmp = tempfile::tempdir().unwrap();
    let home_real = tmp.path().join("home-real");
    let project = home_real.join("project");
    std::fs::create_dir_all(home_real.join(".mindtape")).unwrap();
    std::fs::write(
        home_real.join(".mindtape").join("defaults.toml"),
        "[a]\nx = 1\n",
    )
    .unwrap();
    std::fs::create_dir_all(&project).unwrap();

    let home_link = tmp.path().join("home-link");
    std::os::unix::fs::symlink(&home_real, &home_link).unwrap();

    let layers = discover_layers_with_xdg(&project, Some(&home_link), None);
    // Exactly one Dir layer (the home-real defaults.toml), not duplicated by
    // the symlink alias, and the walk stopped there rather than continuing.
    let dir_layers: Vec<_> = layers
        .iter()
        .filter(|layer| matches!(layer.source, LayerSource::Dir(_)))
        .collect();
    assert_eq!(dir_layers.len(), 1);
}

// --- full assembly: identical to parse_config("") with nothing on disk ---

#[test]
fn no_files_anywhere_matches_parse_config_empty() {
    let (_tmp, home, project) = home_and_project();

    let layers = discover_layers_with_xdg(&project, Some(&home), None);
    let config = build_config(&layers).unwrap();
    let expected = parse_config("").unwrap();
    assert_eq!(config, expected);
}

#[test]
fn merged_config_table_returns_content_from_project_layer() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("project");
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[mct_marker]\nfound = true\n",
    )
    .unwrap();

    let table = merged_config_table(&project).unwrap();
    assert!(
        table
            .get("mct_marker")
            .and_then(|t| t.get("found"))
            .and_then(toml::Value::as_bool)
            == Some(true)
    );
}
