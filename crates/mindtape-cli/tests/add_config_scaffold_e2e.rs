//! End-to-end tests for the config-derived values `add` splices into a task
//! when the project config is not the barebones default: a required priority
//! slot in its various shapes (raw int, real scale, strict framework), an
//! initial status derived from a custom legend, and a required estimate's
//! bounded default.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

/// A config whose `[tasks.priority]` is required: the barebones template omits
/// priority, so `add` must splice it in.
const PRIORITY_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n";

#[test]
fn injects_config_required_priority_slot() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe` surfaces the spliced-in priority slot.
    let names = support::describe_slot_names(dir.path());
    assert!(names.iter().any(|n| n == "priority"), "{names:?}");

    // Filling it writes a valid task.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "with-pri",
            "--set",
            "title=T",
            "--set",
            "priority=7",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-pri.typ")).unwrap();
    assert!(written.contains("priority: 7"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // Omitting a config-required field is the existing missing-slot hard error.
    support::mt(dir.path())
        .args(["add", "--name", "no-pri", "--set", "title=T"])
        .assert()
        .failure();
}

/// A config whose `[tasks.priority]` is real-scaled and required: the
/// barebones template omits priority, so `add` must splice in a `real`-kind
/// slot, not the `int` it would wrongly emit for an int-scaled field.
///
/// `bands = []` clears the embedded floor's int-contiguous bands (0-2/3-6/7-9),
/// which are non-contiguous under real semantics; a real-scale project either
/// defines real bands or drops them.
const REAL_PRIORITY_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"real\"\nmin = 0\nmax = 9\nrequired = true\nbands = []\n";

#[test]
fn injects_real_scale_priority_slot_and_round_trips_a_decimal() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        REAL_PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe=json` reports the spliced-in priority slot as `real`.
    assert_eq!(support::describe_slot_kind(dir.path(), "priority"), "real");

    // Filling it with a decimal writes a float literal without truncation.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "with-real-pri",
            "--set",
            "title=T",
            "--set",
            "priority=2.5",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-real-pri.typ")).unwrap();
    assert!(written.contains("priority: 2.5"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task with a real priority value passes `check`.
    support::mt(dir.path()).arg("check").assert().success();
}

/// A config whose priority is a required strict framework (`allow-manual =
/// false`): a scalar priority is invalid by construction, so `add` must splice
/// the framework call itself with one leaf slot per input — otherwise the
/// fallback template could never file a valid task in this project.
const STRICT_ICE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = \"!terminal\"\n\
     framework = \"ice\"\nallow-manual = false\n";

#[test]
fn injects_framework_leaf_slots_for_a_strict_framework_priority() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(dir.path().join(".mindtape/config.toml"), STRICT_ICE_CONFIG).unwrap();

    // `--describe` surfaces the framework leaves, not a scalar priority.
    let names = support::describe_slot_names(dir.path());
    for leaf in ["impact", "confidence", "ease"] {
        assert!(names.iter().any(|n| n == leaf), "{names:?}");
    }
    assert!(!names.iter().any(|n| n == "priority"), "{names:?}");

    // Filling the leaves writes a valid framework priority.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "with-ice",
            "--set",
            "title=T",
            "--set",
            "impact=9",
            "--set",
            "confidence=0.9",
            "--set",
            "ease=6",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-ice.typ")).unwrap();
    assert!(written.contains("priority: framework(\"ice\""), "{written}");
    assert!(written.contains("impact: 9.0"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task passes `check` (the framework score lands in the scale).
    support::mt(dir.path()).arg("check").assert().success();

    // Omitting a leaf is the existing missing-slot hard error, before any write.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "no-ease",
            "--set",
            "title=T",
            "--set",
            "impact=9",
            "--set",
            "confidence=0.9",
        ])
        .assert()
        .failure();
    assert!(!dir.path().join("tasks/no-ease.typ").exists());
}

/// A config whose legend has no `"proposed"` entry at all — the initial
/// status `add` stamps must be derived from this legend, not a hardcoded
/// literal that would fail `check`.
const CUSTOM_LEGEND_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"todo\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

#[test]
fn derives_initial_status_from_a_custom_legend() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        CUSTOM_LEGEND_CONFIG,
    )
    .unwrap();

    support::mt(dir.path())
        .args(["add", "--name", "custom-legend", "--set", "title=T"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/custom-legend.typ")).unwrap();
    assert!(written.contains("status: todo("), "{written}");
    assert!(!written.contains("proposed"), "{written}");

    // The filed task passes `check` even though the legend has no `proposed`.
    support::mt(dir.path()).arg("check").assert().success();
}

/// The real `.mindtape/templates/task.typ` this repo ships, so a drift in its
/// `estimate` slot default (e.g. drifting over the configured cap) fails
/// this test rather than only surfacing at dogfood `mt check` time.
const PRODUCTION_TEMPLATE: &str = include_str!("../../../.mindtape/templates/task.typ");

/// The template imports `../lib.typ` beside the tasks root. A stub that
/// accepts every verdict keeps this test about the estimate default, not about
/// which goals the repo currently holds.
const GOALS_STUB: &str =
    "#let helps(reason) = reason\n#let hurts(reason) = reason\n#let neutral = none\n";

/// Mirrors the dogfood config's `[tasks.estimate]`: required on every
/// non-terminal task, capped at 5 `src-files` by a rule, no minimum.
const ESTIMATE_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nframework = \"ice\"\n\
     [tasks.estimate]\nunits = [\"src-files\"]\nrequired = \"!terminal\"\n\
     [[tasks.rules]]\nassert = \"estimate.src-files <= 5\"\nseverity = \"error\"\n";

/// `estimate` required with no explicit `--set estimate=...`: the production
/// template's slot default must fill in a dict inside the configured cap,
/// not leave a task that fails `check` the moment it lands.
#[test]
fn add_with_no_explicit_estimate_defaults_to_a_valid_task() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        ESTIMATE_REQUIRED_CONFIG,
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        PRODUCTION_TEMPLATE,
    )
    .unwrap();
    std::fs::write(dir.path().join("lib.typ"), GOALS_STUB).unwrap();

    support::mt(dir.path())
        .args(["add", "--name", "no-estimate", "--set", "title=No estimate"])
        .args(["--set", "problem=+ A step."])
        .args(["--set", "to-be=- An end state."])
        .args(["--set", "changes=- `a.rs`: a change."])
        .args(["--set", "out-of-scope=- Nothing."])
        .args(["--set", "dogfood=Not applicable."])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/no-estimate.typ")).unwrap();
    assert!(written.contains("estimate: (src-files: 2)"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    support::mt(dir.path()).arg("check").assert().success();
}
