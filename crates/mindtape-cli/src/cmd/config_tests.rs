#![allow(clippy::unwrap_used)]

use mindtape::test_support::{tempdir_with_config, tempdir_with_dirs};
use std::path::PathBuf;

use mindtape::{Config, LayerSource, RuleOrigin, load_config, load_config_layered, parse_config};

use super::{render_effective, render_origin, render_schema};

/// A fresh tempdir with `.mindtape/` already created (so `find_root` and the
/// layer discovery walk see a project), optionally holding a `config.toml`.
fn bare_project() -> tempfile::TempDir {
    tempdir_with_dirs(&[".mindtape"])
}

#[test]
fn effective_config_round_trips_through_parse_config() {
    let dir = tempdir_with_config("[project]\ncode = \"demo\"\n");

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
    let dir = tempdir_with_config(
        "[dicts.verbs]\nvalues = [\"add\", \"wire\"]\ninclude = [\"mindtape-verbs\"]\n",
    );

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
    let dir = tempdir_with_config("[project]\ncode = \"demo\"\n");

    let (_config, sources, rule_origins) = load_config_layered(dir.path()).unwrap();
    assert!(
        rule_origins.is_empty(),
        "a bare project declares no rules, so the origin text is layers only"
    );
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
    let dir = tempdir_with_config("[project]\ncode = \"demo\"\n");

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
        let output = mindtape::test_support::run_self_test(
            &mindtape::test_support::current_test_name(),
            "_MT_CONFIG_DELEGATE",
            |_| {},
        );
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
        let output = mindtape::test_support::run_self_test(
            &mindtape::test_support::current_test_name(),
            "_MT_CONFIG_DELEGATE",
            |_| {},
        );
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

    let home = tempfile::tempdir().unwrap();
    let xdg = describe_source(&LayerSource::Xdg(home.path().join("x.toml")));
    assert!(xdg.starts_with("xdg:"));
    assert!(xdg.contains("(missing)"));

    let dir = describe_source(&LayerSource::Dir(home.path().join("d.toml")));
    assert!(dir.starts_with("dir:"));
    assert!(dir.contains("(missing)"));
}

// --- `mt config show --origin`'s `rule <label>: <origin>` lines: one test
// per label `describe_rule_origin` can print. Five reach it through
// `render_origin` over a real project (`project`, `dir`, `pack <path>`,
// `pack <name>`, `xdg`); `embedded` never can, for the reason argued at its
// test below, so it calls `describe_rule_origin` directly instead.

/// A rule declared in the project's own `.mindtape/config.toml` is labeled
/// `project`.
#[test]
fn origin_labels_the_projects_own_rule_as_project() {
    let dir = tempdir_with_config(
        "[[tasks.rules]]\nname = \"project-rule\"\nassert = 'kind == \"bug\"'\n",
    );

    let text = render_origin(dir.path()).unwrap();
    assert!(
        text.contains("rule tasks.rules.project-rule: project"),
        "{text}"
    );
}

/// A rule declared in an ancestor `.mindtape/defaults.toml` is labeled `dir`.
///
/// `render_origin` only calls the real-environment `load_config_layered`, not
/// the `home`-injectable variant the crate's own layer tests use, so this
/// can't bound the walk at a fake `$HOME`. It doesn't need to: `dir_layers`
/// keeps climbing past a `$HOME` the project isn't under all the way to the
/// filesystem root, so an ancestor directly above a project nested in a
/// tempdir is on that path regardless of where the real `$HOME` points.
#[test]
fn origin_labels_an_ancestor_defaults_rule_as_dir() {
    let base = tempdir_with_dirs(&[".mindtape"]);
    std::fs::write(
        base.path().join(".mindtape").join("defaults.toml"),
        "[[tasks.rules]]\nname = \"ancestor-rule\"\nassert = 'kind == \"bug\"'\n",
    )
    .unwrap();
    let project = base.path().join("project").join(".mindtape");
    std::fs::create_dir_all(&project).unwrap();

    let text = render_origin(project.parent().unwrap()).unwrap();
    assert!(
        text.contains("rule tasks.rules.ancestor-rule: dir"),
        "{text}"
    );
}

/// A rule pulled in through a path pack (an `extends` entry with a path
/// separator, resolved relative to the layer that names it — here the
/// project itself) is labeled `pack <path>`.
#[test]
fn origin_labels_a_path_packs_rule_as_pack_path() {
    let dir = bare_project();
    std::fs::create_dir_all(dir.path().join("packs")).unwrap();
    let pack_path = dir.path().join("packs").join("pack.toml");
    std::fs::write(
        &pack_path,
        "[[tasks.rules]]\nname = \"from-pack\"\nassert = 'kind == \"bug\"'\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "extends = [\"packs/pack.toml\"]\n",
    )
    .unwrap();

    let text = render_origin(dir.path()).unwrap();
    let expected = format!("rule tasks.rules.from-pack: pack {}", pack_path.display());
    assert!(text.contains(&expected), "{text}");
}

/// A rule declared in the XDG global `defaults.toml` is labeled `xdg`.
///
/// `render_origin` resolves that layer through `$MINDTAPE_CONFIG_HOME` (the
/// one override that lands on every platform — `dirs` reads `$HOME` on Unix
/// but a known-folder API on Windows, which no env var moves; see
/// `mindtape::loader::app_dirs`). Setting it on the current process would leak into
/// every other test sharing it, so — like `run_defaults_prints_embedded_floor`
/// above — this scopes the override to a spawned copy of the test binary
/// instead of mutating this process's environment.
#[test]
fn origin_labels_an_xdg_defaults_rule_as_xdg() {
    const PROJECT_ENV: &str = "_MT_CONFIG_XDG_PROJECT";
    if std::env::var_os(PROJECT_ENV).is_none() {
        // Parent: scaffold a bare project and an XDG config home holding a
        // rule, then spawn a child with `$MINDTAPE_CONFIG_HOME` pointed at it.
        let project = bare_project();
        let xdg = tempdir_with_dirs(&["mindtape"]);
        std::fs::write(
            xdg.path().join("mindtape").join("defaults.toml"),
            "[[tasks.rules]]\nname = \"xdg-rule\"\nassert = 'kind == \"bug\"'\n",
        )
        .unwrap();

        let output = mindtape::test_support::run_self_test(
            &mindtape::test_support::current_test_name(),
            PROJECT_ENV,
            |child| {
                child
                    .env(PROJECT_ENV, project.path())
                    .env(mindtape::CONFIG_HOME_ENV, xdg.path());
            },
        );
        mindtape::test_support::assert_child_ok(&output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("rule tasks.rules.xdg-rule: xdg"),
            "{stdout}"
        );
        return;
    }
    // Child: render the origin for the project the parent scaffolded, under
    // the injected `$MINDTAPE_CONFIG_HOME`.
    let root = PathBuf::from(std::env::var_os(PROJECT_ENV).unwrap());
    print!("{}", render_origin(&root).unwrap());
}

/// A rule pulled in through a bare `extends` name resolved against the
/// compiled-in `EMBEDDED_PACKS` registry (`crates/mindtape/src/config/packs.
/// rs`) is labeled `pack <name>`.
#[test]
fn origin_labels_an_embedded_packs_rule_as_pack_name() {
    let dir = tempdir_with_config("extends = [\"mindtape-tasks\"]\n");

    let text = render_origin(dir.path()).unwrap();
    assert!(
        text.contains("rule tasks.rules.imperative-title: pack mindtape-tasks"),
        "{text}"
    );
}

/// `embedded` — a rule declared directly in the compiled-in floor
/// (`RuleOrigin::Layer(LayerSource::Embedded)`) — can never be reached
/// through `render_origin` at all: the embedded `defaults.toml` declares no
/// `[[tasks.rules]]`, `[[tasks.body.headers.rules]]` or
/// `[[tasks.body.content.rules]]` entry, so no real merge ever produces this
/// origin. Call `describe_rule_origin` directly to pin the label regardless.
#[test]
fn describe_rule_origin_labels_the_embedded_floor() {
    use super::describe_rule_origin;

    let origin = RuleOrigin::Layer(LayerSource::Embedded);
    assert_eq!(describe_rule_origin(&origin), "embedded");
}
