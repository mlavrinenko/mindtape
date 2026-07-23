//! End-to-end tests for the `init` command that shell out to the `mt` binary.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

/// Run `mt` in `dir` and assert it succeeds.
fn mt(dir: &Path, args: &[&str]) {
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir)
        .args(args)
        .assert()
        .success();
}

/// File a task in `dir` via `mt add --name <name> --set title=T <extra_set…>`,
/// then assert `mt ls` and `mt check` both succeed against it.
fn add_then_ls_and_check(dir: &Path, name: &str, extra_set: &[&str]) {
    let mut args = vec!["add", "--name", name, "--set", "title=T"];
    args.extend_from_slice(extra_set);
    mt(dir, &args);
    mt(dir, &["ls"]);
    mt(dir, &["check"]);
}

/// A scaffolded project for `preset`, with the Typst package installed.
fn init_preset(preset: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    mt(dir.path(), &["init", "--preset", preset]);
    dir
}

/// Every built-in preset: the walk from the status `mt add` stamps through to a
/// terminal one, plus the `--set` fills that walk needs. The path is spelled out
/// rather than derived so that a preset whose declared `to` edges do not
/// actually connect fails here.
///
/// `gated` gates `done` on the task carrying no leftover TODO placeholders, so
/// its walk fills the template's slots the way an author would — the walk then
/// proves the preset's own gate passes for a task that was actually specified.
const PRESET_PATHS: &[(&str, &[&str], &[&str])] = &[
    ("todone", &["done"], &[]),
    ("flow", &["wip", "done"], &[]),
    ("personal", &["next", "done"], &[]),
    (
        "gated",
        &["accepted", "wip", "needs-review", "done"],
        &GATED_FILLS,
    ),
];

/// Slot fills that leave a `gated` task free of the template's placeholders.
const GATED_FILLS: [&str; 8] = [
    "--set",
    "summary=- the thing that is wrong",
    "--set",
    "acceptance=- a condition someone else can check",
    "--set",
    "non-goals=- everything else",
    "--set",
    "files=- src/lib.rs",
];

/// The gate that makes a first run trustworthy: for every built-in preset,
/// scaffold, file a task with nothing but a title, and walk it to a terminal
/// status — validating at each step.
///
/// This is what catches a preset that requires something its own template does
/// not produce. Such a project scaffolds cleanly and then fails on the user's
/// very first `mt add`, which is the worst possible moment to discover it.
#[test]
fn every_preset_supports_add_check_and_a_full_walk() {
    for (preset, path, fills) in PRESET_PATHS {
        let dir = init_preset(preset);
        add_then_ls_and_check(dir.path(), "wire-preset", fills);
        for status in *path {
            mt(dir.path(), &["flip", status, "wire-preset"]);
            mt(dir.path(), &["check"]);
        }
    }
}

/// Reachable wizard answer sets — composed configs, not the four named presets.
/// Each row is the batch flags that compose the config, the `--set` fills its
/// first `mt add` needs, and the walk from the status `mt add` stamps through to
/// a terminal one. Spelled out (not derived) so a set whose declared `to` edges
/// do not connect, or whose config requires an unfillable field, fails here.
const ANSWER_SETS: &[(&[&str], &[&str], &[&str])] = &[
    (
        &[
            "--workflow",
            "todo-done",
            "--priority",
            "off",
            "--organisation",
            "flat",
        ],
        &[],
        &["done"],
    ),
    (
        &[
            "--workflow",
            "gated",
            "--priority",
            "ice",
            "--organisation",
            "folders",
            "--headers",
            "spec",
        ],
        &[],
        &["accepted", "wip", "needs-review", "done"],
    ),
    (
        &[
            "--workflow",
            "flow",
            "--required",
            "difficulty",
            "--recurrence",
            "rrule",
            "--estimate",
            "days-hours",
        ],
        &["--set", "difficulty=3"],
        &["wip", "done"],
    ),
    (&["--workflow", "personal"], &[], &["next", "done"]),
];

/// A composed answer set is as trustworthy as a named preset: for each one,
/// batch-compose the config from flags, file a task, and walk it to a terminal
/// status — validating at each step. Free composition multiplies the risk a
/// config requires a field its scaffolded template cannot fill, so this covers
/// reachable sets, not only the four presets.
#[test]
fn composed_answer_sets_support_add_check_and_a_full_walk() {
    for (flags, fills, walk) in ANSWER_SETS {
        let dir = tempfile::tempdir().unwrap();
        support::install_lib(dir.path());
        let mut init_args = vec!["init", "--no-input"];
        init_args.extend_from_slice(flags);
        mt(dir.path(), &init_args);
        add_then_ls_and_check(dir.path(), "wire-set", fills);
        for status in *walk {
            mt(dir.path(), &["flip", status, "wire-set"]);
            mt(dir.path(), &["check"]);
        }
    }
}

/// The wizard refuses an incoherent answer set — a strict-framework priority no
/// scaffolded template can supply — and writes nothing.
#[test]
fn strict_framework_priority_answer_set_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--no-input", "--priority", "ice-strict"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("strict framework"));
    assert!(!dir.path().join(".mindtape/config.toml").exists());
}

/// A batch-composed config records its axis answers as a comment block, and the
/// `spec` headers answer ships the template that produces its sections.
#[test]
fn composed_config_records_answers_and_ships_templates() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    mt(
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
    mt(dir.path(), &["init", "--no-input"]);
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("[[tasks.status]]"), "{config}");
    assert!(!config.contains("# answers:"), "{config}");
    add_then_ls_and_check(dir.path(), "wire-default", &[]);
}

/// `mt init` with no `--preset` writes the default shape, and the written file
/// carries the schema directive and a real status legend.
#[test]
fn init_default_preset_supports_add_ls_check() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    mt(dir.path(), &["init"]);

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    // First line is the schema-association directive (default: the installed schema).
    assert!(config.starts_with("#:schema "), "{config}");
    assert!(config.contains("[[tasks.status]]"), "{config}");

    add_then_ls_and_check(dir.path(), "wire-init", &[]);
}

/// The default rendering is quiet: apart from the schema directive, which is
/// prepended after stripping, no comment lines survive.
#[test]
fn default_rendering_carries_no_commentary() {
    let dir = init_preset("flow");
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
    mt(dir.path(), &["init", "--preset", "todone", "--annotate"]);

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("The smallest shape"), "{config}");
    assert!(
        config.contains("[[tasks.status]]"),
        "annotation must not change the content: {config}"
    );
    add_then_ls_and_check(dir.path(), "wire-annotated", &[]);
}

/// A preset that constrains task bodies also installs the template producing
/// them, so the scaffolded project's first `mt add` satisfies its own config.
#[test]
fn gated_preset_installs_its_template() {
    let dir = init_preset("gated");
    let template = dir.path().join(".mindtape/templates/task.typ");
    assert!(template.exists(), "gated must ship its task template");

    mt(dir.path(), &["add", "--name", "spec", "--set", "title=T"]);
    let task = std::fs::read_to_string(dir.path().join("tasks/spec.typ")).unwrap();
    for heading in ["Summary", "Acceptance criteria", "Non-goals", "Files"] {
        assert!(task.contains(heading), "missing {heading}:\n{task}");
    }
    mt(dir.path(), &["check"]);
}

/// The `gated` review gate is real: `done` is unreachable directly from `wip`,
/// so a run of work cannot close itself.
#[test]
fn gated_preset_blocks_done_straight_from_wip() {
    let dir = init_preset("gated");
    mt(dir.path(), &["add", "--name", "guard", "--set", "title=T"]);
    mt(dir.path(), &["flip", "accepted", "guard"]);
    mt(dir.path(), &["flip", "wip", "guard"]);

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["flip", "done", "guard"])
        .assert()
        .failure();
}

/// The `gated` preset's shipped `[[on.flip]]` gate is real: a task still
/// carrying the template's TODO placeholders cannot reach `done`, and `--force`
/// is the way past — which then records itself in the status note.
#[test]
fn gated_preset_gates_done_on_an_unfilled_task() {
    let dir = init_preset("gated");
    mt(dir.path(), &["add", "--name", "hollow", "--set", "title=T"]);
    for status in ["accepted", "wip", "needs-review"] {
        mt(dir.path(), &["flip", status, "hollow"]);
    }

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["flip", "done", "hollow"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("TODO placeholders"));

    mt(dir.path(), &["flip", "done", "hollow", "--force"]);
    let task = std::fs::read_to_string(dir.path().join("tasks/hollow.typ")).unwrap();
    assert!(task.contains("forced past failing gate"), "{task}");
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
    mt(dir.path(), &["init", "--preset", "preset.toml"]);
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

    add_then_ls_and_check(dir.path(), "wire-overlay", &[]);
    mt(dir.path(), &["flip", "done", "wire-overlay"]);
    mt(dir.path(), &["check"]);
}

/// A preset whose whole body is under `[overlay]` composes to just that overlay:
/// the base is empty, and the result is a valid config that supports the cycle.
#[test]
fn overlay_only_preset_composes_to_the_overlay() {
    let dir = init_from_preset_file(OVERLAY_ONLY_PRESET);
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(!config.contains("overlay"), "{config}");
    assert!(config.contains("[[tasks.status]]"), "{config}");

    add_then_ls_and_check(dir.path(), "wire-overlay-only", &[]);
    mt(dir.path(), &["flip", "done", "wire-overlay-only"]);
    mt(dir.path(), &["check"]);
}

/// An unknown preset name fails and lists what exists, rather than reaching for
/// a path or the network.
#[test]
fn unknown_preset_fails_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--preset", "nope"])
        .assert()
        .failure();
    assert!(!dir.path().join(".mindtape/config.toml").exists());
}

/// `--list-presets` names every built-in and writes nothing.
#[test]
fn list_presets_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let output = assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--list-presets"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    for (preset, _, _) in PRESET_PATHS {
        assert!(stdout.contains(preset), "{stdout}");
    }
    assert!(!dir.path().join(".mindtape/config.toml").exists());
}

/// `--schema URL` writes that exact value as the first-line `#:schema`
/// directive, and the scaffold still supports the add/ls/check cycle.
#[test]
fn init_schema_override_writes_directive() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    mt(
        dir.path(),
        &["init", "--schema", "https://example.com/config.schema.json"],
    );

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(
        config.starts_with("#:schema https://example.com/config.schema.json\n"),
        "{config}"
    );
    add_then_ls_and_check(dir.path(), "wire-schema", &[]);
}

/// `--no-schema` omits the directive entirely.
#[test]
fn init_no_schema_omits_directive() {
    let dir = tempfile::tempdir().unwrap();
    support::install_lib(dir.path());
    mt(dir.path(), &["init", "--no-schema"]);

    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(!config.contains("#:schema"), "{config}");
}

/// Without `--force` an existing config survives; with it, `init` overwrites.
/// A skipped write installs no templates either — a half-applied preset would
/// leave a project whose config and templates disagree.
#[test]
fn force_interplay_unchanged() {
    let dir = support::scaffold_project("# existing");

    mt(dir.path(), &["init", "--preset", "gated"]);
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap(),
        "# existing"
    );
    assert!(
        !dir.path().join(".mindtape/templates/task.typ").exists(),
        "a skipped scaffold must not install templates"
    );

    mt(dir.path(), &["init", "--preset", "gated", "--force"]);
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("needs-review"), "{config}");
    assert!(dir.path().join(".mindtape/templates/task.typ").exists());
}
