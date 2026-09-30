//! End-to-end tests for the `add` command that shell out to the `mt` binary:
//! core wiring (positional title, `--to` nesting, YAML stdin), the slot
//! schema, error surfaces (collisions, unknown keys, a body-rule breach),
//! and the kept-on-verification-failure contract. Markup-hazard coverage
//! lives in `add_markup_e2e.rs`.
//!
//! Config-derived splicing (a required priority slot, a custom status legend,
//! a required estimate default) lives in `add_config_scaffold_e2e.rs`, a
//! separate binary along that subject seam.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

#[test]
fn end_to_end_files_and_validates_a_task() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());

    support::mt(dir.path())
        .args(["add", "--name", "wire-add", "--set", "title=Wire add"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-add.typ")).unwrap();
    assert!(written.contains("title: \"Wire add\""), "{written}");
    assert!(written.contains("status: proposed("), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task passes `check`.
    support::mt(dir.path()).arg("check").assert().success();
}

#[test]
fn end_to_end_files_from_positional_title() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());

    support::mt(dir.path())
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
    support::scaffold(dir.path());

    support::mt(dir.path())
        .args(["add", "--to", "shopping", "Potatoes"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/shopping/potatoes.typ")).unwrap();
    assert!(written.contains("title: \"Potatoes\""), "{written}");
}

#[test]
fn describe_json_emits_slot_schema() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    assert!(
        support::describe_slot_names(dir.path())
            .iter()
            .any(|n| n == "title")
    );
}

/// `mt add --list` prints the template stems and stops: no template is
/// resolved, no slot is filled, nothing is written.
#[test]
fn list_prints_template_stems_and_stops() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());

    let output = support::mt(dir.path())
        .args(["add", "--list"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.lines().any(|line| line == "task"), "{stdout}");
    assert!(
        std::fs::read_dir(dir.path().join("tasks"))
            .unwrap()
            .next()
            .is_none()
    );
}

#[test]
fn collision_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(dir.path().join("tasks/dup.typ"), "existing").unwrap();
    support::mt(dir.path())
        .args(["add", "--name", "dup", "--set", "title=Dup"])
        .assert()
        .failure();
}

/// `--set` naming no slot of the active template must fail loudly, naming the
/// key, rather than silently filing a task with the wrong (default) value.
#[test]
fn unknown_set_key_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());

    let output = support::mt(dir.path())
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

/// `mt add -f -` accepts YAML on stdin, not just JSON — real stdin can only
/// be exercised by shelling out, hence this lives here rather than as a unit
/// test (unit coverage for the dispatch/sniff logic itself lives in
/// `add_values_tests.rs`).
#[test]
fn end_to_end_files_from_yaml_on_stdin() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());

    support::mt(dir.path())
        .args(["add", "--name", "wire-yaml", "-f", "-"])
        .write_stdin("title: Wire YAML\n")
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-yaml.typ")).unwrap();
    assert!(written.contains("title: \"Wire YAML\""), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");
}

/// Same treatment for the `-f`/`--values` JSON entry point.
#[test]
fn unknown_values_json_key_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    let values = dir.path().join("v.json");
    std::fs::write(&values, r#"{"title":"T","priorty":3}"#).unwrap();

    let output = support::mt(dir.path())
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

/// A template whose whole `task.with(…)` call sits on one line, with no
/// trailing comma for `add`'s own `status:` to be separated by. Legal Typst,
/// and the one shape no in-tree template happens to be written in.
const SINGLE_LINE_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(title: slot("title"))

== Summary
"#;

/// `add` injects `status:` into whatever `task.with(…)` the template declares.
/// Against a single-line call it used to write `title: "…"status: proposed(…)`
/// — a file that cannot parse, reported only *after* it was on disk. The whole
/// point of this test is being the other shape: the multi-line one every
/// in-tree template uses is covered several times over above, which is how this
/// got through.
#[test]
fn single_line_template_files_a_task_that_loads() {
    let dir = support::project_with_template(SINGLE_LINE_TEMPLATE);

    support::mt(dir.path())
        .args(["add", "--name", "one-line", "--set", "title=One line"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/one-line.typ")).unwrap();
    assert!(written.contains("title: \"One line\""), "{written}");
    assert!(written.contains("status: proposed("), "{written}");
    support::mt(dir.path()).arg("check").assert().success();
}

/// A template that cannot evaluate no matter how its slots are filled, so the
/// verification always fails.
const BROKEN_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(title: slot("title"))

#no-such-function()
"#;

/// `add` validates the render before it commits it, so a rejection creates no
/// file at all. The board is readable straight afterwards — one mistyped slot
/// costs a retry, not a repair — and the error names the input rather than a
/// file to fix or delete.
#[test]
fn a_task_that_fails_verification_is_never_written() {
    let dir = support::project_with_template(BROKEN_TEMPLATE);

    let output = support::mt(dir.path())
        .args(["add", "--name", "broken", "--set", "title=Broken"])
        .assert()
        .failure()
        .get_output()
        .clone();

    let file = dir.path().join("tasks/broken.typ");
    assert!(!file.exists(), "a rejected task must never reach the disk");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("did not load"),
        "the underlying cause must still be reported: {stderr}"
    );
    assert!(
        !stderr.contains("kept on disk"),
        "there is no file to repair, so nothing may claim there is: {stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("wrote"),
        "a rejected add must not report a write: {output:?}"
    );

    // The next command runs clean: nothing was left behind to trip over.
    support::mt(dir.path()).arg("check").assert().success();
}

/// A template that evaluates fine but whose filled value breaks a *rule* — the
/// shape actually observed: an `estimate` slot filled with a bare number where
/// the field wants a dict. Distinct from [`BROKEN_TEMPLATE`], which fails at
/// eval; this one reaches the marker and is refused by the Rust layer.
const ESTIMATE_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title"),
  estimate: slot("estimate", kind: "raw", default: "(days: 1)"),
)

== Summary
"#;

/// The reported case, end to end: a slot filled with the wrong shape is caught
/// before the write, so no repair is needed before the next command.
#[test]
fn a_wrong_shaped_slot_value_is_refused_before_the_write() {
    let dir = support::project_with_template(ESTIMATE_TEMPLATE);

    let output = support::mt(dir.path())
        .args([
            "add",
            "--name",
            "wrong-shape",
            "--set",
            "title=Wrong shape",
            "--set",
            "estimate=5",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(!dir.path().join("tasks/wrong-shape.typ").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("wrong-shape: estimate:"),
        "the error must name the task and the offending field: {stderr}"
    );
    assert!(
        stderr.contains("nothing was written"),
        "the error must say the write did not happen: {stderr}"
    );
    support::mt(dir.path()).arg("check").assert().success();
}

/// `--no-verify` still writes whatever it was handed: the flag skips the check
/// wherever the check sits, and moving it before the write must not quietly
/// turn it into a gate the flag cannot open.
#[test]
fn no_verify_writes_a_task_that_would_be_refused() {
    let dir = support::project_with_template(BROKEN_TEMPLATE);

    support::mt(dir.path())
        .args([
            "add",
            "--no-verify",
            "--name",
            "broken",
            "--set",
            "title=Broken",
        ])
        .assert()
        .success();

    assert!(dir.path().join("tasks/broken.typ").exists());
}

/// A project-configured content rule (this repo's own "every `TO-BE` bullet
/// opens with 🟢") — mirroring
/// `truth/tasks/validate-body-rules-before-mt-add-writes-a-task.typ`'s own
/// Problem: `mt check` already refuses this shape, but a filled task used to
/// reach disk anyway and wait for that later check to catch it.
const TO_BE_CONTENT_RULE_CONFIG: &str = r#"[tasks]
roots = ["tasks"]

[[tasks.body.content.rules]]
path = ["TO-BE"]
assert = { must-not-match = "(?m)^\\s*- [^🟢]" }
message = "open every TO-BE bullet with 🟢"
"#;

/// A template whose `TO-BE` bullet does not open with 🟢 — breaks
/// [`TO_BE_CONTENT_RULE_CONFIG`]'s rule on every fill.
const TO_BE_TEMPLATE: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title"),
  status: proposed(2026, 6, 14),
)

== TO-BE

- ships the feature
"#;

/// The Problem this task exists to fix, end to end: `mt add` now runs the same
/// content gate `mt check` does, refuses the task before writing it, and
/// leaves nothing behind for a later `mt check` to catch.
#[test]
fn refuses_a_task_whose_to_be_bullet_breaks_a_content_rule() {
    let dir = support::project_with_template_and_config(TO_BE_CONTENT_RULE_CONFIG, TO_BE_TEMPLATE);

    let output = support::mt(dir.path())
        .args(["add", "--name", "ships-it", "--set", "title=Ships it"])
        .assert()
        .failure()
        .get_output()
        .clone();

    let file = dir.path().join("tasks/ships-it.typ");
    assert!(!file.exists(), "a rejected task must never reach the disk");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("open every TO-BE bullet with 🟢"),
        "the body-rule message must be reported: {stderr}"
    );

    // Nothing was left behind to trip over.
    support::mt(dir.path()).arg("check").assert().success();
}

/// The harness must never hand a spawned `mt` the developer's editor. A
/// `cargo-mutants` mutant that wrongly makes `add` interactive otherwise
/// spawns `$VISUAL` once per test and blocks until the test timeout —
/// a `--wait` GUI editor opens one window per affected test.
/// Every e2e spawn goes through `support::isolate`, so the lock is here and
/// not on `add`'s callers.
#[test]
fn isolated_spawns_never_inherit_the_developers_editor() {
    let dir = tempfile::tempdir().unwrap();
    let mut command = std::process::Command::new("mt");
    support::isolate(&mut command, dir.path());

    let envs: Vec<_> = command.get_envs().collect();
    assert!(
        envs.iter().any(|(key, value)| {
            *key == std::ffi::OsStr::new("EDITOR")
                && *value == Some(std::ffi::OsStr::new("mt-no-such-editor"))
        }),
        "an isolated spawn must get the stub editor: {envs:?}"
    );
    assert!(
        !envs
            .iter()
            .any(|(key, value)| *key == std::ffi::OsStr::new("VISUAL") && value.is_some()),
        "an isolated spawn must not carry VISUAL: {envs:?}"
    );
}
