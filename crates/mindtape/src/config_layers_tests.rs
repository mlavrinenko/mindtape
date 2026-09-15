#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{LayerSource, build_config, discover_layers_with_xdg, merged_config_table};
use crate::config::parse_config;
use crate::test_support::home_and_project;

// The pure precedence-folding tests that used to live here now exercise
// `crate::config_packs::merge_layers_with_packs_using` instead — see
// `config_packs_tests.rs` — since that function (not this module) now owns
// the actual fold `load_config_layered` runs.

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

/// Asserted through the injected seam, never through `load_config`: that entry
/// point reads the real `dirs::config_dir()`, so the same assertion made
/// against it passes or fails on whether the developer happens to keep a
/// `~/.config/mindtape/defaults.toml` — which is a supported layer, so the
/// failure would be the test's, not the loader's. A twin of this test did go
/// through `load_config` and started failing the moment one was written.
#[test]
fn no_files_anywhere_matches_parse_config_empty() {
    let (_tmp, home, project) = home_and_project();

    let layers = discover_layers_with_xdg(&project, Some(&home), None);
    let (config, _rule_origins) = build_config(&layers, &project).unwrap();
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

// --- `[dicts]`: the root-relative half, which only the layered load can see ---

/// A `[dicts.<name>] path` resolves against the project root, so the file is
/// found through a real layered load and not through `parse_config`, which has
/// no root to resolve against.
#[test]
fn a_dict_path_resolves_against_the_project_root() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[dicts.verbs]\npath = \"dicts/verbs.txt\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(project.join("dicts")).unwrap();
    std::fs::write(project.join("dicts").join("verbs.txt"), "# head\nadd\n").unwrap();

    let (config, _sources, _rule_origins) =
        super::load_config_layered_from(&project, Some(&home), None).unwrap();
    let dicts = crate::config::compile_dicts(&config.dicts, &project).unwrap();
    assert!(dicts.contains("verbs", "Add"));
}

/// An unreadable `path` fails the load itself — once, as a config error naming
/// the dict, rather than once per task that reads it.
#[test]
fn an_unreadable_dict_path_fails_the_load() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[dicts.verbs]\npath = \"dicts/missing.txt\"\n",
    )
    .unwrap();

    let err = super::load_config_layered_from(&project, Some(&home), None).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("dicts.verbs.path"), "{rendered}");
    assert!(rendered.contains("missing.txt"), "{rendered}");
}

// --- `@<dict>` references in `[tasks.tags] allowed` / `[tasks.fields.<name>]
// values` ---

/// A `@<dict>` entry expands to that dictionary's entries — in the
/// dictionary's own (alphabetical) order, not declaration order — so a
/// downstream reader of `tasks.tags.allowed` never sees the sigil.
#[test]
fn a_dict_ref_in_tags_allowed_expands_to_the_dict_s_entries() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[dicts.areas]\nvalues = [\"frontend\", \"backend\"]\n\n\
         [tasks.tags]\nallowed = [\"@areas\", \"swept\"]\n",
    )
    .unwrap();

    let (config, _sources, _rule_origins) =
        super::load_config_layered_from(&project, Some(&home), None).unwrap();
    assert_eq!(config.tasks.tags.allowed, ["backend", "frontend", "swept"]);
}

/// The same expansion applies to `[tasks.fields.<name>] values`.
#[test]
fn a_dict_ref_in_field_values_expands_to_the_dict_s_entries() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[dicts.kinds]\nvalues = [\"chore\", \"bug\"]\n\n\
         [tasks.fields.kind]\nvalues = [\"@kinds\"]\n",
    )
    .unwrap();

    let (config, _sources, _rule_origins) =
        super::load_config_layered_from(&project, Some(&home), None).unwrap();
    assert_eq!(config.tasks.fields["kind"].values, ["bug", "chore"]);
}

/// A list may mix literal entries and `@<dict>` references; the result is
/// their union in declaration order, deduplicated (first occurrence wins) —
/// the literal `"backend"` here is already contributed by `@areas`.
#[test]
fn a_mixed_list_unions_literals_and_dict_refs_in_order_deduped() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[dicts.areas]\nvalues = [\"frontend\", \"backend\"]\n\n\
         [tasks.tags]\nallowed = [\"swept\", \"@areas\", \"backend\"]\n",
    )
    .unwrap();

    let (config, _sources, _rule_origins) =
        super::load_config_layered_from(&project, Some(&home), None).unwrap();
    assert_eq!(config.tasks.tags.allowed, ["swept", "backend", "frontend"]);
}

/// An unknown `@<dict>` reference fails the load, naming the offending list
/// and offering a nearest-match suggestion — the same treatment an unknown
/// `include` target gets.
#[test]
fn an_unknown_dict_ref_fails_the_load_with_a_suggestion() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[dicts.areas]\nvalues = [\"backend\"]\n\n\
         [tasks.tags]\nallowed = [\"@ares\"]\n",
    )
    .unwrap();

    let err = super::load_config_layered_from(&project, Some(&home), None).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("tasks.tags.allowed"), "{rendered}");
    assert!(rendered.contains("unknown dict `ares`"), "{rendered}");
    assert!(rendered.contains("did you mean: areas?"), "{rendered}");
}

/// A literal entry that itself starts with `@` is never a plain string: the
/// sigil always means "dictionary reference", so an unresolvable one fails
/// the load exactly like a typo'd dict name — there is no escape.
#[test]
fn a_literal_at_prefixed_entry_with_no_matching_dict_fails_the_load() {
    let (_tmp, home, project) = home_and_project();
    std::fs::create_dir_all(project.join(".mindtape")).unwrap();
    std::fs::write(
        project.join(".mindtape").join("config.toml"),
        "[tasks.tags]\nallowed = [\"@urgent\"]\n",
    )
    .unwrap();

    let err = super::load_config_layered_from(&project, Some(&home), None).unwrap_err();
    assert!(err.to_string().contains("unknown dict `urgent`"), "{err}");
}
