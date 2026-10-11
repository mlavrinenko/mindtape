//! End-to-end tests for what `init` actually writes to disk: the default and
//! `--no-input` shapes, quiet rendering vs `--annotate`, a custom preset
//! file's `[overlay]` composition, the `#:schema` directive flags, `--force`
//! overwrite semantics, and an unknown preset name failing without writing.
//!
//! Whether the resulting project is actually usable — the full `add`/`ls`/
//! `check`/status-walk battery over every built-in preset and composed answer
//! set, plus the `gated` preset's runtime review gate — lives in
//! `init_e2e.rs`, a separate binary along that subject seam.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

/// A batch-composed config records its axis answers as a comment block, and the
/// `spec` headers answer ships the template that produces its sections.
#[test]
fn composed_config_records_answers_and_ships_templates() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    support::run_ok(
        dir.path(),
        &[
            "init",
            "--no-input",
            "--workflow",
            "gated",
            "--headers",
            "spec",
        ],
    );
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("# answers:"), "{config}");
    assert!(config.contains("workflow = gated"), "{config}");
    assert!(config.contains("headers = spec"), "{config}");
    assert!(dir.path().join(".mindtape/templates/task.typ").exists());
}

/// `--no-input` with no axis flags falls through to the default preset — the
/// same shape a bare non-terminal `mt init` writes — with no wizard answer block
/// and no prompt.
#[test]
fn no_input_without_flags_writes_the_default_preset() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    support::run_ok(dir.path(), &["init", "--no-input"]);
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("[[tasks.status]]"), "{config}");
    assert!(!config.contains("# answers:"), "{config}");
    support::add_then_ls_and_check(dir.path(), "wire-default", &[]);
}

/// `mt init` with no `--preset` writes the default shape, and the written file
/// carries the schema directive and a real status legend.
#[test]
fn init_default_preset_supports_add_ls_check() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    support::run_ok(dir.path(), &["init"]);

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    // First line is the schema-association directive. It defaults to the
    // published URL: the config is checked in, so a machine-local `file://`
    // path would be meaningless to anyone who clones the repo.
    assert!(
        config.starts_with(&format!("#:schema {}\n", mindtape::SCHEMA_URL)),
        "{config}"
    );
    assert!(!config.contains("file://"), "{config}");
    assert!(config.contains("[[tasks.status]]"), "{config}");

    support::add_then_ls_and_check(dir.path(), "wire-init", &[]);
}

/// The default rendering is quiet: apart from the schema directive, which is
/// prepended after stripping, no comment lines survive.
#[test]
fn default_rendering_carries_no_commentary() {
    let dir = support::init_preset("flow");
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    let comments: Vec<&str> = config
        .lines()
        .skip(1)
        .filter(|line| line.trim_start().starts_with('#'))
        .collect();
    assert!(comments.is_empty(), "{comments:?}");
}

/// `--annotate` keeps the reasoning, and the annotated file still works. The
/// two flags are independent: one picks the shape, the other the commentary.
#[test]
fn annotate_keeps_commentary_and_still_works() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    support::run_ok(dir.path(), &["init", "--preset", "todone", "--annotate"]);

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("The smallest shape"), "{config}");
    assert!(
        config.contains("[[tasks.status]]"),
        "annotation must not change the content: {config}"
    );
    support::add_then_ls_and_check(dir.path(), "wire-annotated", &[]);
}

/// A todone-shaped base plus a top-level `[overlay]` the axes cannot express — a
/// named view. The overlay merges last, so the flattened config carries `[views]`
/// and no literal `[overlay]`.
const OVERLAY_PRESET: &str = "\
[tasks]
roots = [\"tasks\"]

[[tasks.status]]
value = \"todo\"
glyph = \"○\"
color = \"blue\"

[[tasks.status]]
value = \"done\"
glyph = \"●\"
color = \"green\"
terminal = true

[tasks.priority]
bands = []

[overlay.views.mine]
filter = \"!terminal\"
";

/// A preset whose entire body sits under `[overlay]`: the base is empty, so the
/// composition is exactly the overlay — the preset-with-only-an-overlay case.
const OVERLAY_ONLY_PRESET: &str = "\
[overlay.tasks]
roots = [\"tasks\"]

[[overlay.tasks.status]]
value = \"todo\"
glyph = \"○\"
color = \"blue\"

[[overlay.tasks.status]]
value = \"done\"
glyph = \"●\"
color = \"green\"
terminal = true
";

/// Scaffold from a path preset written to `<dir>/preset.toml`, returning the dir.
fn init_from_preset_file(body: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    std::fs::write(dir.path().join("preset.toml"), body).unwrap();
    support::run_ok(dir.path(), &["init", "--preset", "preset.toml"]);
    dir
}

/// A preset carrying a top-level `[overlay]` composes: the overlay merges last
/// over the base, the written config is flattened (no literal `[overlay]`) and
/// carries the overlay's own section, and it supports the add / ls / check cycle.
#[test]
fn preset_overlay_composes_a_flattened_valid_config() {
    let dir = init_from_preset_file(OVERLAY_PRESET);
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(!config.contains("[overlay]"), "{config}");
    assert!(!config.contains("[overlay."), "{config}");
    assert!(
        config.contains("[views.mine]"),
        "overlay section flattened in: {config}"
    );
    assert!(config.contains("[[tasks.status]]"), "{config}");

    support::add_then_ls_and_check(dir.path(), "wire-overlay", &[]);
    support::run_ok(dir.path(), &["flip", "done", "wire-overlay"]);
    support::run_ok(dir.path(), &["check"]);
}

/// A preset whose whole body is under `[overlay]` composes to just that overlay:
/// the base is empty, and the result is a valid config that supports the cycle.
#[test]
fn overlay_only_preset_composes_to_the_overlay() {
    let dir = init_from_preset_file(OVERLAY_ONLY_PRESET);
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(!config.contains("overlay"), "{config}");
    assert!(config.contains("[[tasks.status]]"), "{config}");

    support::add_then_ls_and_check(dir.path(), "wire-overlay-only", &[]);
    support::run_ok(dir.path(), &["flip", "done", "wire-overlay-only"]);
    support::run_ok(dir.path(), &["check"]);
}

/// An unknown preset name fails and lists what exists, rather than reaching for
/// a path or the network.
#[test]
fn unknown_preset_fails_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    support::mt(dir.path())
        .args(["init", "--preset", "nope"])
        .assert()
        .failure();
    assert!(!dir.path().join(".mindtape/config.toml").exists());
}

/// `--schema URL` writes that exact value as the first-line `#:schema`
/// directive, and the scaffold still supports the add/ls/check cycle.
#[test]
fn init_schema_override_writes_directive() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    support::run_ok(
        dir.path(),
        &["init", "--schema", "https://example.com/config.schema.json"],
    );

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(
        config.starts_with("#:schema https://example.com/config.schema.json\n"),
        "{config}"
    );
    support::add_then_ls_and_check(dir.path(), "wire-schema", &[]);
}

/// `--no-schema` omits the directive entirely.
#[test]
fn init_no_schema_omits_directive() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    support::run_ok(dir.path(), &["init", "--no-schema"]);

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(!config.contains("#:schema"), "{config}");
}

/// Without `--force` an existing config survives; with it, `init` overwrites.
/// A skipped write installs no templates either — a half-applied preset would
/// leave a project whose config and templates disagree.
#[test]
fn force_interplay_unchanged() {
    let dir = support::scaffold_project("# existing");

    support::run_ok(dir.path(), &["init", "--preset", "gated"]);
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap(),
        "# existing"
    );
    assert!(
        !dir.path().join(".mindtape/templates/task.typ").exists(),
        "a skipped scaffold must not install templates"
    );

    support::run_ok(dir.path(), &["init", "--preset", "gated", "--force"]);
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("needs-review"), "{config}");
    assert!(dir.path().join(".mindtape/templates/task.typ").exists());
}
