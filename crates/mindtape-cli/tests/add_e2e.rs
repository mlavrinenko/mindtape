//! End-to-end tests for the `add` command that shell out to the `mt` binary:
//! core wiring (positional title, `--to` nesting, YAML stdin), the slot
//! schema, error surfaces (collisions, unknown keys), markup-hazard escaping,
//! and the kept-on-verification-failure contract.
//!
//! Config-derived splicing (a required priority slot, a custom status legend,
//! a required estimate default) lives in `add_config_scaffold_e2e.rs`, a
//! separate binary along that subject seam.
//!
//! These live here (not as inline `#[path]` unit tests) because `cargo_bin`
//! needs the compiled binary: Cargo builds it before running integration tests
//! and exports `CARGO_BIN_EXE_mt`, a guarantee `--lib` unit tests don't get.
#![allow(clippy::unwrap_used)]

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

/// Prose carrying every Typst markup leader must file a task that loads and
/// whose values read back exactly as typed. The `text` slots reach markup and
/// are escaped on the way in; the `string` title and `int` priority render as
/// literals and must not be, or the backslashes would become part of the value.
#[test]
fn markup_hazards_in_slot_values_round_trip_literally() {
    let dir = support::project_with_slot_template();
    let hazards = support::MARKUP_HAZARDS;

    support::mt(dir.path())
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

/// A `raw`-kind slot splices its value verbatim: a code span filled through
/// `--set` survives as a code span, not an escaped literal. Round trip: a
/// code span in, a code span out, no backslashes — the opt-out `text` slots
/// deferred (`add-a-typst-slot-kind-for-pre-formatted-markup.typ`).
#[test]
fn raw_slot_splices_markup_without_escaping() {
    let dir = support::project_with_raw_slot_template();

    support::mt(dir.path())
        .args(["add", "--name", "raw-body", "--set", "title=Raw body"])
        .args(["--set", "summary=See `mt add` for details."])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/raw-body.typ")).unwrap();
    assert!(
        written.contains("See `mt add` for details."),
        "the code span must survive verbatim: {written}"
    );
    assert!(
        !written.contains(r"\`"),
        "a raw slot must not be markup-escaped: {written}"
    );

    // Reading back proves it landed as live markup, not inert text: `mt eval`
    // sees the same code span in the harvested body.
    support::mt(dir.path()).arg("check").assert().success();
}

/// A `raw` slot's filler is trusted to be markup, not guaranteed to be
/// *valid* markup: a value that fails to parse must fail `mt add` itself,
/// before anything is written, with an error naming the slot rather than a
/// bare compiler diagnostic about a file the caller never hand-wrote.
#[test]
fn raw_slot_unparseable_markup_fails_naming_the_slot_before_writing() {
    let dir = support::project_with_raw_slot_template();

    let output = support::mt(dir.path())
        .args(["add", "--name", "broken-raw", "--set", "title=Broken"])
        .args(["--set", "summary=#unclosed["])
        .assert()
        .failure()
        .get_output()
        .clone();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("\"summary\""),
        "the error must name the slot: {stderr}"
    );
    assert!(
        !dir.path().join("tasks/broken-raw.typ").exists(),
        "a raw slot's markup is checked before anything is written"
    );
}

/// `--describe=json` must report `raw` as the kind so a caller can tell
/// which slots accept raw markup before building a payload.
#[test]
fn describe_reports_raw_kind() {
    let dir = support::project_with_raw_slot_template();
    assert_eq!(support::describe_slot_kind(dir.path(), "summary"), "raw");
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
    let dir = support::project_with_template(BROKEN_TEMPLATE);

    let output = support::mt(dir.path())
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
