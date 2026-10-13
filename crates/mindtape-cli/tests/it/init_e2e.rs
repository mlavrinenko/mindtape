//! End-to-end tests that every built-in `init` preset, and every answer set
//! the wizard's batch flags can compose, scaffold a project that is actually
//! usable: `add`, `ls`, `check`, and a full status walk to a terminal value —
//! including the `gated` preset's own runtime review gate — plus discovering
//! presets via `--list-presets`.
//!
//! What `init` writes to disk for a given input (rendering, `--annotate`, a
//! custom preset file's `[overlay]` composition, the `#:schema` flags,
//! `--force`, an unknown preset name) lives in `init_config_write_e2e.rs`, a
//! separate binary along that subject seam.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;

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
/// status — validating at each step. It allows the preset's commands first, so
/// `gated`'s own gate runs.
///
/// This is what catches a preset that requires something its own template does
/// not produce. Such a project scaffolds cleanly and then fails on the user's
/// very first `mt add`, which is the worst possible moment to discover it.
#[test]
fn every_preset_supports_add_check_and_a_full_walk() {
    for (preset, path, fills) in PRESET_PATHS {
        let dir = support::init_preset(preset);
        support::run_ok(dir.path(), &["allow"]);
        support::add_then_ls_and_check(dir.path(), "wire-preset", fills);
        for status in *path {
            support::run_ok(dir.path(), &["flip", status, "wire-preset"]);
            support::check_ok(dir.path());
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
    (
        &["--priority", "ice-strict"],
        &[
            "--set",
            "impact=9",
            "--set",
            "confidence=0.9",
            "--set",
            "ease=6",
        ],
        &["wip", "done"],
    ),
];

/// A composed answer set is as trustworthy as a named preset: for each one,
/// batch-compose the config from flags, file a task, and walk it to a terminal
/// status — validating at each step. Free composition multiplies the risk a
/// config requires a field its scaffolded template cannot fill, so this covers
/// reachable sets, not only the four presets.
#[test]
fn composed_answer_sets_support_add_check_and_a_full_walk() {
    for (flags, fills, walk) in ANSWER_SETS {
        let dir = tempdir();
        support::install_lib(dir.path());
        let mut init_args = vec!["init", "--no-input"];
        init_args.extend_from_slice(flags);
        support::run_ok(dir.path(), &init_args);
        support::add_then_ls_and_check(dir.path(), "wire-set", fills);
        for status in *walk {
            support::run_ok(dir.path(), &["flip", status, "wire-set"]);
            support::check_ok(dir.path());
        }
    }
}

/// `--recurrence rrule` scaffolds a project whose first `recur: rrule(...)`
/// task passes `mt check`: the config opens the gate, and the binary carries
/// the engine that checks the rule.
#[test]
fn init_accepts_the_rrule_answer_and_checks_an_rrule_task() {
    let dir = tempdir();
    support::install_lib(dir.path());
    support::run_ok(dir.path(), &["init", "--no-input", "--recurrence", "rrule"]);
    write_file(
        dir.path(),
        "tasks/cost-report.typ",
        mindtape::typst_task!(
            "(\n  title: \"Send the cost report\",\n  \
         status: proposed(2026, 9, 14),\n  due: on(2026, 10, 5),\n  \
         recur: rrule(\"FREQ=MONTHLY;BYDAY=1MO\"),\n)\n"
        ),
    );
    support::check_ok(dir.path());
}

/// A strict-framework answer set scaffolds like any other: the fallback
/// template supplies the required priority through the spliced framework leaf
/// slots (covered end-to-end in `ANSWER_SETS` above), so init writes the
/// config instead of refusing the set.
#[test]
fn strict_framework_priority_answer_set_scaffolds() {
    let dir = tempdir();
    support::install_lib(dir.path());
    support::run_ok(
        dir.path(),
        &["init", "--no-input", "--priority", "ice-strict"],
    );
    let config = std::fs::read_to_string(dir.path().join(".mindtape/config.toml")).unwrap();
    assert!(config.contains("allow-manual = false"), "{config}");
}

/// A preset that constrains task bodies also installs the template producing
/// them, so the scaffolded project's first `mt add` satisfies its own config.
#[test]
fn gated_preset_installs_its_template() {
    let dir = support::init_preset("gated");
    let template = dir.path().join(".mindtape/templates/task.typ");
    assert!(template.exists(), "gated must ship its task template");

    let task = support::add_and_read(dir.path(), "spec", &["--set", "title=T"]);
    for heading in ["Summary", "Acceptance criteria", "Non-goals", "Files"] {
        assert!(task.contains(heading), "missing {heading}:\n{task}");
    }
    support::check_ok(dir.path());
}

/// The `gated` review gate is real: `done` is unreachable directly from `wip`,
/// so a run of work cannot close itself.
#[test]
fn gated_preset_blocks_done_straight_from_wip() {
    let dir = support::init_preset("gated");
    support::run_ok(dir.path(), &["add", "--name", "guard", "--set", "title=T"]);
    support::run_ok(dir.path(), &["flip", "accepted", "guard"]);
    support::run_ok(dir.path(), &["flip", "wip", "guard"]);

    support::mt(dir.path())
        .args(["flip", "done", "guard"])
        .assert()
        .failure();
}

/// The `gated` preset's shipped `[[on.flip]]` gate is real: a task still
/// carrying the template's TODO placeholders cannot reach `done`, and `--force`
/// is the way past — which then records itself in the status note.
#[test]
fn gated_preset_gates_done_on_an_unfilled_task() {
    let dir = support::init_preset("gated");
    support::run_ok(dir.path(), &["allow"]);
    support::run_ok(dir.path(), &["add", "--name", "hollow", "--set", "title=T"]);
    for status in ["accepted", "wip", "needs-review"] {
        support::run_ok(dir.path(), &["flip", status, "hollow"]);
    }

    support::mt(dir.path())
        .args(["flip", "done", "hollow"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("TODO placeholders"));

    support::run_ok(dir.path(), &["flip", "done", "hollow", "--force"]);
    let task = std::fs::read_to_string(dir.path().join("tasks/hollow.typ")).unwrap();
    assert!(task.contains("forced past failing gate"), "{task}");
}

/// `--list-presets` names every built-in and writes nothing.
#[test]
fn list_presets_writes_nothing() {
    let dir = tempdir();
    let output = support::mt(dir.path())
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

/// `--list-presets` names the user preset directory `mt` resolves, and no
/// directory when none resolves.
#[test]
fn list_presets_names_the_resolved_user_preset_dir() {
    let dir = tempdir();
    let config_home = dir.path().join("config");
    let output = support::mt(dir.path())
        .env(mindtape::CONFIG_HOME_ENV, &config_home)
        .args(["init", "--list-presets"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let presets = config_home.join("mindtape").join("presets");
    assert!(stdout.contains(&*presets.to_string_lossy()), "{stdout}");

    let output = support::mt(dir.path())
        .env(mindtape::CONFIG_HOME_ENV, "")
        .args(["init", "--list-presets"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("user presets"), "{stdout}");
}
