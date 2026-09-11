#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use mindtape::{Config, LayerSource, load_config, load_config_layered, parse_config};

use super::{render_effective, render_origin, render_schema};

/// A fresh tempdir with `.mindtape/` already created (so `find_root` and the
/// layer discovery walk see a project), optionally holding a `config.toml`.
fn bare_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    dir
}

#[test]
fn effective_config_round_trips_through_parse_config() {
    let dir = bare_project();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "[project]\ncode = \"demo\"\n",
    )
    .unwrap();

    let text = render_effective(dir.path(), false).unwrap();
    let reparsed = parse_config(&text).unwrap();
    let original = load_config(dir.path()).unwrap();
    assert_eq!(reparsed, original);
    assert_eq!(reparsed.project.code.as_deref(), Some("demo"));
}

#[test]
fn effective_config_json_round_trips_to_same_shape_as_toml() {
    let dir = bare_project();
    let toml_text = render_effective(dir.path(), false).unwrap();
    let json_text = render_effective(dir.path(), true).unwrap();

    let from_toml = parse_config(&toml_text).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_text).unwrap();
    // Round the JSON value back through a TOML table so the same typed
    // `Config` deserialization applies to both encodings.
    let table: toml::Table = serde_json::from_value(json_value).unwrap();
    let from_json: Config = toml::Value::Table(table).try_into().unwrap();
    assert_eq!(from_toml, from_json);
}

/// `mt config show` surfaces `[dicts.<name>]` like any other section — it goes
/// out on the merged document and comes back through the typed `Config`, which
/// is `deny_unknown_fields`, so an unreflected section would fail the re-parse
/// rather than print a table nothing reads.
#[test]
fn effective_config_surfaces_the_dicts_section() {
    let dir = bare_project();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "[dicts.verbs]\nvalues = [\"add\", \"wire\"]\ninclude = [\"mindtape-verbs\"]\n",
    )
    .unwrap();

    let text = render_effective(dir.path(), false).unwrap();
    assert!(text.contains("[dicts.verbs]"), "{text}");
    let reparsed = parse_config(&text).unwrap();
    let spec = reparsed.dicts.get("verbs").unwrap();
    assert_eq!(spec.values, ["add", "wire"]);
    assert_eq!(spec.include, ["mindtape-verbs"]);
    assert_eq!(reparsed, load_config(dir.path()).unwrap());
    // `Config` equality is blind to the compiled dictionary cache (a function
    // of the section plus a project root, which `Config` does not carry — see
    // `DictCache`), so the set itself is asserted here rather than left to the
    // comparison above: the rendered document has to carry enough for a reader
    // to rebuild the same dictionary, not merely the same spec table.
    assert!(reparsed.compiled_dicts().contains("verbs", "Add"));
}

#[test]
fn bare_project_show_reflects_the_embedded_floor() {
    let dir = bare_project();
    let text = render_effective(dir.path(), false).unwrap();
    let config = parse_config(&text).unwrap();

    // The floor ships a non-empty legend with at least one terminal value.
    assert!(!config.tasks.status.entries.is_empty());
    assert!(config.tasks.status.entries.iter().any(|e| e.terminal));
}

#[test]
fn origin_lists_exactly_the_layers_load_config_layered_returns() {
    let dir = bare_project();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "[project]\ncode = \"demo\"\n",
    )
    .unwrap();

    let (_config, sources) = load_config_layered(dir.path()).unwrap();
    let text = render_origin(dir.path()).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), sources.len());

    for (line, source) in lines.iter().zip(sources.iter()) {
        match source {
            mindtape::LayerSource::Embedded => assert!(line.starts_with("embedded:")),
            mindtape::LayerSource::Xdg(path) => {
                assert!(line.starts_with("xdg:"));
                assert!(line.contains(&path.display().to_string()));
            }
            mindtape::LayerSource::Dir(path) => {
                assert!(line.starts_with("dir:"));
                assert!(line.contains(&path.display().to_string()));
            }
            mindtape::LayerSource::Project(path) => {
                assert!(line.starts_with("project:"));
                assert!(line.contains(&path.display().to_string()));
            }
        }
    }
}

#[test]
fn origin_marks_the_project_layer_as_existing() {
    let dir = bare_project();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "[project]\ncode = \"demo\"\n",
    )
    .unwrap();

    let text = render_origin(dir.path()).unwrap();
    let project_line = text
        .lines()
        .find(|line| line.starts_with("project:"))
        .expect("a project layer must be present");
    assert!(project_line.contains("(exists)"));
}

/// Verify that `run()` with `--defaults` emits the embedded floor to stdout.
///
/// Uses a self-spawn subprocess so the parent can capture what the child prints
/// (println! goes to the child's stdout). A `run()` whose body is replaced with
/// `Ok(())` would print nothing, so the `!is_empty()` assertion fails and the
/// mutated test is caught.
#[test]
fn run_defaults_prints_embedded_floor() {
    if std::env::var("_MT_CONFIG_DELEGATE").is_err() {
        // Parent: spawn self, capture output, verify it's non-empty.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .env("_MT_CONFIG_DELEGATE", "1")
            .args(["run_defaults_prints_embedded_floor", "--nocapture"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "run() with --defaults must print to stdout"
        );
        assert!(
            stdout.contains("[tasks]"),
            "output should contain config TOML"
        );
        return;
    }
    // Child: call run() with --defaults.
    let args = super::ConfigArgs {
        action: Some(super::ConfigAction::Show(super::ShowArgs {
            origin: false,
            defaults: true,
            json: false,
        })),
    };
    super::run(&args).unwrap();
}

/// `render_schema` must reuse `mindtape::config_schema_json` verbatim — the
/// same generator `mt install` writes to disk — so the printed schema and the
/// installed `config.schema.json` can never diverge.
#[test]
fn render_schema_matches_the_install_generator() {
    assert_eq!(render_schema(), mindtape::config_schema_json());
}

/// Verify that `run()` with the `schema` action prints the JSON Schema to
/// stdout, verbatim.
///
/// Uses a self-spawn subprocess so the parent can capture what the child
/// prints (println! goes to the child's stdout). The child runs under
/// `--nocapture`, so the captured stream also carries the test harness's own
/// banner lines — check the exact generated schema appears as a substring
/// rather than parsing the whole capture as JSON. A `run()` whose body is
/// replaced with `Ok(())` would print nothing, so the `contains` assertion
/// fails and the mutated test is caught.
#[test]
fn run_schema_prints_json_schema_to_stdout() {
    if std::env::var("_MT_CONFIG_DELEGATE").is_err() {
        // Parent: spawn self, capture output, verify it carries the schema.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .env("_MT_CONFIG_DELEGATE", "1")
            .args(["run_schema_prints_json_schema_to_stdout", "--nocapture"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(&mindtape::config_schema_json()),
            "run() with schema must print the generated schema verbatim"
        );
        return;
    }
    // Child: call run() with the schema action.
    let args = super::ConfigArgs {
        action: Some(super::ConfigAction::Schema),
    };
    super::run(&args).unwrap();
}

#[test]
fn describe_source_covers_xdg_and_dir() {
    use super::describe_source;

    let xdg = describe_source(&LayerSource::Xdg(PathBuf::from("/tmp/x.toml")));
    assert!(xdg.starts_with("xdg:"));
    assert!(xdg.contains("(missing)"));

    let dir = describe_source(&LayerSource::Dir(PathBuf::from("/tmp/d.toml")));
    assert!(dir.starts_with("dir:"));
    assert!(dir.contains("(missing)"));
}
