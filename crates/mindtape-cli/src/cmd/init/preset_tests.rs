#![allow(clippy::unwrap_used)]

use mindtape::test_support::write_file;
use mindtape::test_support::{tempdir, tempdir_with_files};
use std::path::Path;

use super::{
    DEFAULT_PRESET, list_builtins, list_user_presets, resolve, split_overlay, strip_comments,
};

/// Write a nested user preset (`<dir>/<name>/config.toml`) and return its root.
fn write_nested_preset(dir: &Path, name: &str, config: &str, templates: &[(&str, &str)]) {
    let root = dir.join(name);
    std::fs::create_dir_all(&root).unwrap();
    write_file(&root, "config.toml", config);
    if templates.is_empty() {
        return;
    }
    let template_dir = root.join("templates");
    std::fs::create_dir_all(&template_dir).unwrap();
    for (file, body) in templates {
        write_file(&template_dir, file, body);
    }
}

/// Every built-in resolves, and its config parses and validates against the
/// embedded floor — both renderings, since `--annotate` must not change meaning.
#[test]
fn every_builtin_parses_in_both_renderings() {
    for (name, _) in list_builtins() {
        let preset = resolve(name, None).unwrap();
        for annotate in [true, false] {
            let text = preset.render(annotate);
            let config = mindtape::parse_config(&text)
                .unwrap_or_else(|err| panic!("preset {name} (annotate={annotate}): {err}"));
            config
                .validate()
                .unwrap_or_else(|err| panic!("preset {name} (annotate={annotate}): {err:?}"));
        }
    }
}

/// A built-in preset declares no command, so a fresh project runs nothing it
/// must `mt allow` first, and the same config works under `sh` and `cmd`. A
/// gate a preset ships is a filter condition. A user's own preset or config may
/// declare any `run`: this holds Mindtape's presets alone to the rule.
#[test]
fn no_builtin_declares_a_command() {
    for (name, _) in list_builtins() {
        let config = mindtape::parse_config(&resolve(name, None).unwrap().render(false)).unwrap();
        assert_eq!(
            mindtape::allow::commands(&config),
            Vec::<mindtape::shell::ConfigCommand>::new(),
            "preset {name}"
        );
    }
}

/// Stripping comments must not change what the config means: both renderings
/// deserialize to the same `Config`.
#[test]
fn stripping_comments_preserves_config() {
    for (name, _) in list_builtins() {
        let preset = resolve(name, None).unwrap();
        let annotated = mindtape::parse_config(&preset.render(true)).unwrap();
        let stripped = mindtape::parse_config(&preset.render(false)).unwrap();
        assert_eq!(annotated, stripped, "preset {name} differs when stripped");
    }
}

/// The annotated rendering carries comments and the default one does not.
#[test]
fn annotate_controls_comments() {
    let preset = resolve(DEFAULT_PRESET, None).unwrap();
    assert!(preset.render(true).contains('#'));
    let stripped = preset.render(false);
    assert!(
        !stripped
            .lines()
            .any(|line| line.trim_start().starts_with('#')),
        "{stripped}"
    );
}

/// The default preset is one of the built-ins.
#[test]
fn default_preset_is_a_builtin() {
    assert!(
        list_builtins()
            .iter()
            .any(|(name, _)| *name == DEFAULT_PRESET)
    );
}

/// No preset is named for a position in a hierarchy: a neutral-sounding name
/// hides a subjective choice, so the vocabulary is checked, not just reviewed.
#[test]
fn no_preset_is_named_for_its_position() {
    for (name, _) in list_builtins() {
        assert!(
            !matches!(name, "default" | "base" | "standard" | "recommended"),
            "preset {name} is named for its position, not its opinion"
        );
    }
}

/// A preset that constrains task bodies ships the template producing them.
#[test]
fn gated_preset_ships_a_template() {
    let preset = resolve("gated", None).unwrap();
    assert!(
        preset.templates.iter().any(|(file, _)| file == "task.typ"),
        "gated constrains body headers, so it must ship a task template"
    );
}

/// The work-shaped presets refuse starting or closing blocked work.
#[test]
fn work_presets_require_unblocked_wip_and_done() {
    for name in ["flow", "gated"] {
        let preset = resolve(name, None).unwrap();
        let legend = mindtape::parse_config(&preset.render(false))
            .unwrap()
            .tasks
            .status;
        for status in ["wip", "done"] {
            assert!(legend.requires_unblocked(status), "{name}: {status}");
        }
    }
}

/// An unknown name errors and says where to see the presets, rather than
/// falling through to a path or a fetch.
#[test]
fn unknown_preset_errors_and_says_where_to_list_them() {
    let err = crate::diag::plain(&resolve("nope", None).unwrap_err());
    assert_eq!(
        err,
        "error: unknown preset: 'nope'\nhint: run mt preset ls to see the presets\n"
    );
}

/// A near miss names the built-in it meant.
#[test]
fn a_misspelt_preset_names_the_nearest_builtin() {
    let err = crate::diag::plain(&resolve(&DEFAULT_PRESET.replace('o', "0"), None).unwrap_err());
    assert!(
        err.contains(&format!("hint: did you mean {DEFAULT_PRESET}?")),
        "{err}"
    );
}

/// A user preset shadows a built-in of the same name.
#[test]
fn user_preset_shadows_builtin() {
    let dir = tempdir_with_files(&[(
        &format!("{DEFAULT_PRESET}.toml"),
        "[tasks]\nroots = [\"mine\"]\n",
    )]);
    let preset = resolve(DEFAULT_PRESET, Some(dir.path())).unwrap();
    assert!(preset.config.contains("mine"), "{}", preset.config);
}

/// A user preset directory contributes its templates too.
#[test]
fn user_preset_directory_carries_templates() {
    let dir = tempdir();
    write_nested_preset(
        dir.path(),
        "mine",
        "[tasks]\nroots = [\"tasks\"]\n",
        &[("task.typ", "// t"), ("other.typ", "// o")],
    );
    let preset = resolve("mine", Some(dir.path())).unwrap();
    // Sorted, so a preset writes the same bytes everywhere.
    assert_eq!(
        preset
            .templates
            .iter()
            .map(|(file, _)| file.as_str())
            .collect::<Vec<_>>(),
        vec!["other.typ", "task.typ"]
    );
}

/// A user directory without the named preset falls through to the built-in.
#[test]
fn missing_user_preset_falls_through_to_builtin() {
    let dir = tempdir();
    let preset = resolve(DEFAULT_PRESET, Some(dir.path())).unwrap();
    assert_eq!(preset.name, DEFAULT_PRESET);
}

/// An explicit `.toml` path resolves even though it is no preset name.
#[test]
fn explicit_toml_path_resolves() {
    let dir = tempdir();
    let path = dir.path().join("custom.toml");
    std::fs::write(&path, "[tasks]\nroots = [\"x\"]\n").unwrap();
    let preset = resolve(path.to_str().unwrap(), None).unwrap();
    assert_eq!(preset.name, "custom");
    assert!(preset.config.contains("\"x\""));
}

/// An explicit directory path resolves through the nested layout.
#[test]
fn explicit_directory_path_resolves() {
    let dir = tempdir();
    write_nested_preset(
        dir.path(),
        "shape",
        "[tasks]\nroots = [\"y\"]\n",
        &[("task.typ", "// t")],
    );
    let preset = resolve(dir.path().join("shape").to_str().unwrap(), None).unwrap();
    assert_eq!(preset.templates.len(), 1);
}

/// A missing explicit path errors rather than silently writing a built-in.
#[test]
fn missing_explicit_path_errors() {
    let dir = tempdir();
    let missing = dir.path().join("absent.toml");
    assert!(resolve(missing.to_str().unwrap(), None).is_err());
}

/// A `#` inside a string value is not a comment and must survive stripping.
#[test]
fn strip_comments_keeps_hash_inside_values() {
    let text = "# gone\ncolor = \"#ff0000\"\n";
    assert_eq!(strip_comments(text), "color = \"#ff0000\"\n");
}

/// Stripping collapses the blank runs comments leave, and never opens with one.
#[test]
fn strip_comments_collapses_blank_runs() {
    let text = "# a\n# b\n\n\n[tasks]\n\n\n\nroots = []\n";
    assert_eq!(strip_comments(text), "[tasks]\n\nroots = []\n");
}

/// An indented comment is still a comment.
#[test]
fn strip_comments_drops_indented_comments() {
    assert_eq!(strip_comments("[a]\n   # note\nb = 1\n"), "[a]\nb = 1\n");
}

/// Stripping an already-quiet config is a no-op.
#[test]
fn strip_comments_is_idempotent() {
    let once = strip_comments("# c\n\n[tasks]\nroots = []\n");
    assert_eq!(strip_comments(&once), once);
}

/// Listing keeps only real preset shapes — a flat `.toml` and a directory with
/// a `config.toml` — and ignores a plain file and a bare directory. Reading the
/// flat-file guard as `||` would admit any `.toml`-named entry, and reading the
/// directory guard as `||` would admit a directory with no config.
#[test]
fn list_user_presets_keeps_only_preset_shapes() {
    let dir = tempdir_with_files(&[("flat.toml", "[tasks]\n")]);
    write_nested_preset(dir.path(), "nested", "[tasks]\n", &[]);
    write_file(dir.path(), "notes.txt", "not a preset");
    std::fs::create_dir_all(dir.path().join("empty")).unwrap();

    assert_eq!(
        list_user_presets(dir.path()).unwrap(),
        vec!["flat".to_owned(), "nested".to_owned()]
    );
}

/// A config with no `[overlay]` splits to `None`, so ordinary presets keep their
/// verbatim rendering.
#[test]
fn split_overlay_absent_is_none() {
    assert!(
        split_overlay("[tasks]\nroots = [\"tasks\"]\n")
            .unwrap()
            .is_none()
    );
}

/// A `[overlay]` splits into the base (without it) and the overlay's inner
/// contents promoted to top level.
#[test]
fn split_overlay_extracts_base_and_promoted_overlay() {
    let config = "[tasks]\nroots = [\"tasks\"]\n\n[overlay.views.mine]\nfilter = \"!terminal\"\n";
    let (base, overlay) = split_overlay(config).unwrap().unwrap();
    assert!(base.contains("roots"), "{base}");
    assert!(!base.contains("overlay"), "{base}");
    // The overlay's inner contents are promoted: a top-level [views.mine], not
    // a nested [overlay.views.mine].
    assert!(overlay.contains("[views.mine]"), "{overlay}");
    assert!(!overlay.contains("overlay"), "{overlay}");
}

/// An `[overlay]` that is not a table is rejected.
#[test]
fn split_overlay_rejects_a_non_table_overlay() {
    assert!(split_overlay("overlay = 7\n").is_err());
}
