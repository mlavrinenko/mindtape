#![allow(clippy::unwrap_used)]

use mindtape::test_support::{tempdir, tempdir_with_dirs};
use std::path::Path;

use super::{
    SchemaArgs, both_terminals, run_interactively, scaffold_config, with_schema_directive,
    write_templates,
};
use crate::cmd::init::preset::{self, DEFAULT_PRESET};
use crate::diag::Kind;

/// The real `mt` binary, isolated from the developer's global config so `init`
/// writes only into `dir`.
fn mt(dir: &Path) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("mt").unwrap();
    cmd.current_dir(dir)
        .env(mindtape::CONFIG_HOME_ENV, dir.join("no-global-config"))
        .env("HOME", dir)
        .env_remove("MINDTAPE_NOW")
        .env_remove("MINDTAPE_WORKTREE")
        .env_remove("VISUAL");
    cmd
}

const SAMPLE: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// Build `SchemaArgs` for the directive paths.
fn init_args(schema: Option<&str>, no_schema: bool) -> SchemaArgs {
    SchemaArgs {
        schema: schema.map(str::to_owned),
        no_schema,
    }
}

#[test]
fn scaffold_creates_config_when_absent() {
    let dir = tempdir();
    scaffold_config(dir.path(), false, SAMPLE).unwrap();
    assert!(dir.path().join(".mindtape").join("config.toml").exists());
}

/// A project whose config holds `# existing`, and that config's path.
fn existing_config() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir_with_dirs(&[".mindtape"]);
    let config = dir.path().join(".mindtape").join("config.toml");
    std::fs::write(&config, "# existing").unwrap();
    (dir, config)
}

/// Without `--force` an existing config is refused and survives, so the
/// caller writes no templates either.
#[test]
fn scaffold_refuses_existing_without_force() {
    let (dir, config) = existing_config();
    let err = scaffold_config(dir.path(), false, SAMPLE).unwrap_err();
    assert_eq!(err.kind(), Kind::Refused);
    assert_eq!(std::fs::read_to_string(&config).unwrap(), "# existing");
}

#[test]
fn scaffold_overwrites_with_force() {
    let (dir, config) = existing_config();
    scaffold_config(dir.path(), true, SAMPLE).unwrap();
    assert_eq!(std::fs::read_to_string(&config).unwrap(), SAMPLE);
}

/// A preset's templates land in `.mindtape/templates`, where `[templates] dirs`
/// looks for them.
#[test]
fn templates_are_written_beside_the_config() {
    let dir = tempdir();
    let gated = preset::resolve("gated", None).unwrap();
    write_templates(dir.path(), &gated).unwrap();
    let written = dir
        .path()
        .join(".mindtape")
        .join("templates")
        .join("task.typ");
    assert!(written.exists());
    assert!(
        std::fs::read_to_string(written).unwrap().contains("slot("),
        "the template should carry fillable slots"
    );
}

/// A preset with no templates creates no template directory.
#[test]
fn no_templates_writes_no_directory() {
    let dir = tempdir();
    let plain = preset::resolve("todo-done", None).unwrap();
    write_templates(dir.path(), &plain).unwrap();
    assert!(!dir.path().join(".mindtape").join("templates").exists());
}

/// `--schema URL|PATH` writes the given value as the first-line `#:schema`
/// directive, leaving the template body intact below it.
#[test]
fn schema_directive_uses_explicit_value() {
    let out = with_schema_directive(
        SAMPLE,
        &init_args(Some("https://x/config.schema.json"), false),
    )
    .unwrap();
    assert!(
        out.starts_with("#:schema https://x/config.schema.json\n"),
        "{out}"
    );
    assert!(out.ends_with(SAMPLE));
}

/// `--no-schema` leaves the scaffold untouched (no directive prepended).
#[test]
fn no_schema_omits_directive() {
    let out = with_schema_directive(SAMPLE, &init_args(None, true)).unwrap();
    assert_eq!(out, SAMPLE);
}

/// A directive-carrying scaffold still parses and validates — `#:schema` is a
/// TOML comment, ignored by the config loader.
#[test]
fn directive_scaffold_still_parses() {
    let out = with_schema_directive(SAMPLE, &init_args(Some("config.schema.json"), false)).unwrap();
    let config = mindtape::parse_config(&out).unwrap();
    config.validate().unwrap();
}

/// The schema directive survives the stripped rendering: it is a comment, but
/// it is prepended after stripping, so it must still be the first line.
#[test]
fn directive_survives_the_stripped_rendering() {
    let preset = preset::resolve(DEFAULT_PRESET, None).unwrap();
    let out =
        with_schema_directive(&preset.render(false), &init_args(Some("s.json"), false)).unwrap();
    assert!(out.starts_with("#:schema s.json\n"), "{out}");
    let config = mindtape::parse_config(&out).unwrap();
    config.validate().unwrap();
}

/// The default directive is the published URL, not a machine-local path.
///
/// `.mindtape/config.toml` is checked in, so a `file://` default meant every
/// scaffold shipped a path that resolved only on the machine that ran
/// `mt init`. Guards the regression directly: no `file://`, and the exact
/// published URL.
#[test]
fn schema_directive_defaults_to_published_url() {
    let out = with_schema_directive(SAMPLE, &init_args(None, false)).unwrap();
    assert!(
        out.starts_with(&format!("#:schema {}\n", mindtape::SCHEMA_URL)),
        "default should be the published URL, got: {out}"
    );
    assert!(
        !out.contains("file://"),
        "a checked-in config must not carry a machine-local path: {out}"
    );
}

/// Verify that `with_schema_directive` with `--schema local` prints a note when
/// the XDG schema path does not exist.
///
/// The `!` deletion mutant reverses the existence check: the real code prints
/// when the path is missing; the mutant prints when it exists. The self-spawn
/// subprocess captures what `println!` writes to stdout so the parent can
/// verify the note is present (real) or absent (mutant fails the assertion).
///
/// The child gets an empty `$MINDTAPE_CONFIG_HOME`, so "the schema is missing"
/// is a fact about the fixture rather than about the developer's machine:
/// without it, `mindtape::config_home()` resolves to the real config directory
/// and this test fails for anyone who has run `mt install --schema`.
#[test]
fn with_schema_directive_prints_note_when_schema_missing() {
    if std::env::var("_MT_INIT_DELEGATE").is_err() {
        // Parent: spawn self, capture stdout, verify note is present.
        let xdg = tempdir();
        let output = mindtape::test_support::run_self_test(
            &mindtape::test_support::current_test_name(),
            "_MT_INIT_DELEGATE",
            |child| {
                child.env(mindtape::CONFIG_HOME_ENV, xdg.path());
            },
        );
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("schema not installed"),
            "should print a note when the XDG schema path is missing"
        );
        return;
    }
    // Child: call with_schema_directive with `--schema local`. The function
    // calls xdg_schema_path(), checks its existence, and prints a note if the
    // path is missing.
    let out = super::with_schema_directive(SAMPLE, &init_args(Some("local"), false)).unwrap();
    assert!(out.starts_with("#:schema file://"));
}

#[test]
fn dogfood_config_parses_and_validates() {
    const DOGFOOD: &str = include_str!("../../../../../.mindtape/config.toml");
    let config = mindtape::parse_config(DOGFOOD).unwrap();
    config.validate().unwrap();
}

// --- both_terminals / run_interactively ---
//
// The terminal decision is pure here so both sides of each operator are
// reachable: a tty on one stream and a pipe on the other must not prompt, and
// `--no-input` must suppress the prompt even on a real terminal.

#[test]
fn both_terminals_needs_both_streams() {
    assert!(both_terminals(true, true));
    assert!(!both_terminals(true, false));
    assert!(!both_terminals(false, true));
    assert!(!both_terminals(false, false));
}

#[test]
fn run_interactively_is_suppressed_by_no_input_and_by_a_pipe() {
    assert!(run_interactively(false, true));
    assert!(!run_interactively(true, true));
    assert!(!run_interactively(false, false));
    assert!(!run_interactively(true, false));
}

// --- write_preset's hint / print_preset_list's default marker ---
//
// Both are driven through the real binary because each decides only what it
// prints.

/// Run `mt <args>` in a fresh directory, assert it succeeded, and return its
/// stdout.
fn init_stdout(args: &[&str]) -> String {
    let dir = tempdir();
    let assert = mt(dir.path()).args(args).assert().success();
    String::from_utf8_lossy(&assert.get_output().stdout).into_owned()
}

#[test]
fn explicit_preset_omits_the_other_shapes_hint() {
    let stdout = init_stdout(&["init", "--preset", "todo-done"]);
    assert!(
        !stdout.contains("--list-presets"),
        "an explicit preset must not be told to browse: {stdout}"
    );
}

#[test]
fn default_fallthrough_prints_the_other_shapes_hint() {
    let stdout = init_stdout(&["init", "--no-input"]);
    assert!(
        stdout.contains("want something else? mt init --list-presets shows the other presets\n"),
        "the fall-through should point at the other presets: {stdout}"
    );
}

/// Every path that writes a config ends on the command to run next: the
/// default preset, an explicit one, and the wizard composing from flags.
#[test]
fn every_init_path_ends_on_the_next_step() {
    for args in [
        &["init", "--no-input"][..],
        &["init", "--preset", "todo-done"][..],
        &["init", "--workflow", "personal"][..],
    ] {
        let stdout = init_stdout(args);
        assert!(
            stdout.ends_with("next: mt add \"My first task\"\n"),
            "{args:?}: {stdout}"
        );
        assert!(!stdout.contains("scaffolded"), "{args:?}: {stdout}");
    }
}

#[test]
fn preset_list_marks_only_the_default() {
    let stdout = init_stdout(&["init", "--list-presets"]);
    let flow = stdout
        .lines()
        .find(|line| line.trim_start().starts_with(DEFAULT_PRESET))
        .unwrap_or_else(|| panic!("no {DEFAULT_PRESET} line in: {stdout}"));
    assert!(
        flow.contains("(default)"),
        "the default preset must be marked: {flow:?}"
    );
    assert_eq!(
        stdout.matches("(default)").count(),
        1,
        "exactly one preset is the default: {stdout}"
    );
}
