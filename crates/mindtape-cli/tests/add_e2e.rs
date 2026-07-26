//! End-to-end tests for the `add` command that shell out to the `mt` binary.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

/// Scaffold a temp project that can evaluate `@local/mindtape` via the in-tree
/// lib override the loader honours.
fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    support::install_lib(dir);
}

#[test]
fn end_to_end_files_and_validates_a_task() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "wire-add", "--set", "title=Wire add"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-add.typ")).unwrap();
    assert!(written.contains("title: \"Wire add\""), "{written}");
    assert!(written.contains("status: proposed("), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task passes `check`.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();
}

#[test]
fn end_to_end_files_from_positional_title() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "cleanup", "kitchen"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/cleanup-kitchen.typ")).unwrap();
    assert!(written.contains("title: \"cleanup kitchen\""), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");
}

#[test]
fn end_to_end_nests_into_subfolder() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--to", "shopping", "Potatoes"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/shopping/potatoes.typ")).unwrap();
    assert!(written.contains("title: \"Potatoes\""), "{written}");
}

/// Parsed `add --describe=json` schema rows run in `dir`.
fn describe_slots(dir: &Path) -> serde_json::Value {
    let output = assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir)
        .args(["add", "--describe=json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

/// Slot names from `add --describe=json` run in `dir`.
fn describe_slot_names(dir: &Path) -> Vec<String> {
    describe_slots(dir)
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

/// The `kind` reported for slot `name` in `add --describe=json` run in `dir`.
fn describe_slot_kind(dir: &Path, name: &str) -> String {
    describe_slots(dir)
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row.get("name").and_then(serde_json::Value::as_str) == Some(name))
        .and_then(|row| row.get("kind"))
        .and_then(serde_json::Value::as_str)
        .unwrap()
        .to_owned()
}

#[test]
fn describe_json_emits_slot_schema() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    assert!(describe_slot_names(dir.path()).iter().any(|n| n == "title"));
}

/// A config whose `[tasks.priority]` is required: the barebones template omits
/// priority, so `add` must splice it in.
const PRIORITY_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n";

#[test]
fn injects_config_required_priority_slot() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe` surfaces the spliced-in priority slot.
    let names = describe_slot_names(dir.path());
    assert!(names.iter().any(|n| n == "priority"), "{names:?}");

    // Filling it writes a valid task.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
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
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
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
    scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        REAL_PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe=json` reports the spliced-in priority slot as `real`.
    assert_eq!(describe_slot_kind(dir.path(), "priority"), "real");

    // Filling it with a decimal writes a float literal without truncation.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
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
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();
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
    scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        CUSTOM_LEGEND_CONFIG,
    )
    .unwrap();

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "custom-legend", "--set", "title=T"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/custom-legend.typ")).unwrap();
    assert!(written.contains("status: todo("), "{written}");
    assert!(!written.contains("proposed"), "{written}");

    // The filed task passes `check` even though the legend has no `proposed`.
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();
}

/// The real `.mindtape/templates/task.typ` this repo ships, so a drift in its
/// `estimate` slot default (e.g. drifting outside the configured bounds) fails
/// this test rather than only surfacing at dogfood `mt check` time.
const PRODUCTION_TEMPLATE: &str = include_str!("../../../.mindtape/templates/task.typ");

/// Mirrors the dogfood config's `[tasks.estimate]`: required on every
/// non-terminal task, bounded 1-5 files.
const ESTIMATE_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nframework = \"ice\"\n\
     [tasks.estimate]\nunits = [\"files\"]\nrequired = \"!terminal\"\n\
     [tasks.estimate.max]\nfiles = 5\n[tasks.estimate.min]\nfiles = 1\n";

/// `estimate` required with no explicit `--set estimate=...`: the production
/// template's slot default must fill in a dict inside the configured bounds,
/// not leave a task that fails `check` the moment it lands.
#[test]
fn add_with_no_explicit_estimate_defaults_to_a_valid_task() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
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

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "no-estimate", "--set", "title=No estimate"])
        .args(["--set", "summary=A summary."])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/no-estimate.typ")).unwrap();
    assert!(written.contains("estimate: (files: 2)"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();
}

#[test]
fn collision_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::write(dir.path().join("tasks/dup.typ"), "existing").unwrap();
    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "dup", "--set", "title=Dup"])
        .assert()
        .failure();
}

/// `--set` naming no slot of the active template must fail loudly, naming the
/// key, rather than silently filing a task with the wrong (default) value.
#[test]
fn unknown_set_key_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    let output = assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "typo", "--set", "title=T"])
        .args(["--set", "priorty=3"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8_lossy(&output);
    assert!(stderr.contains("priorty"), "{stderr}");
    assert!(
        !dir.path().join("tasks/typo.typ").exists(),
        "a rejected --set must never write a file"
    );
}

/// Same treatment for the `-f`/`--values` JSON entry point.
#[test]
fn unknown_values_json_key_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    let values = dir.path().join("v.json");
    std::fs::write(&values, r#"{"title":"T","priorty":3}"#).unwrap();

    let output = assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "typo", "-f"])
        .arg(&values)
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8_lossy(&output);
    assert!(stderr.contains("priorty"), "{stderr}");
    assert!(!dir.path().join("tasks/typo.typ").exists());
}

/// Prose carrying every Typst markup leader must file a task that loads and
/// whose values read back exactly as typed. The `text` slots reach markup and
/// are escaped on the way in; the `string` title and `int` priority render as
/// literals and must not be, or the backslashes would become part of the value.
#[test]
fn markup_hazards_in_slot_values_round_trip_literally() {
    let dir = support::project_with_slot_template();
    let hazards = support::MARKUP_HAZARDS;

    assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "hazards"])
        .args(["--set", &format!("title={hazards}")])
        .args(["--set", &format!("why={hazards}")])
        .args(["--set", &format!("summary={hazards}")])
        .args(["--set", "weight=7"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/hazards.typ")).unwrap();
    // The two literal kinds keep the caller's bytes: an unquoted integer, and a
    // string literal with no escaping beyond what a Typst string already needs.
    assert!(written.contains("priority: 7,"), "{written}");
    assert!(
        written.contains(&format!("title: \"{hazards}\"")),
        "a string slot must not be markup-escaped: {written}"
    );
    // The markup kinds are escaped at every leader.
    assert!(
        written.contains(r"create-\*star\* \_u\_ \$x\$ \#h \[b\] \<a> >g \@at"),
        "text slots must be escaped: {written}"
    );

    // Reading back is the real assertion: unescaped, `*star*` loses its stars
    // and `<a>`/`@at` evaluate to a label and a reference and disappear.
    let task = support::task_json(dir.path(), "hazards");
    assert_eq!(support::at(&task, "/title").as_str(), Some(hazards));
    assert_eq!(support::at(&task, "/priority").as_f64(), Some(7.0));
    assert_eq!(support::at(&task, "/links/0/note").as_str(), Some(hazards));
}

/// A template that cannot evaluate no matter how its slots are filled, so the
/// post-write verification always fails.
const BROKEN_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(title: slot("title"))

#no-such-function()
"#;

/// `add` writes before it verifies, so a rejected task is already on disk when
/// the error surfaces. It stays there — and the error says so, instead of
/// leaving "wrote <path>" to imply a file that loads.
#[test]
fn a_task_that_fails_verification_is_kept_and_reported_as_kept() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/templates/task.typ"),
        BROKEN_TEMPLATE,
    )
    .unwrap();

    let output = assert_cmd::Command::cargo_bin("mt")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "broken", "--set", "title=Broken"])
        .assert()
        .failure()
        .get_output()
        .clone();

    let file = dir.path().join("tasks/broken.typ");
    assert!(file.exists(), "the written task must be kept, not removed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("was kept on disk"),
        "the error must say the file survives: {stderr}"
    );
    assert!(
        stderr.contains("failed to load"),
        "the underlying cause must still be reported: {stderr}"
    );
}
