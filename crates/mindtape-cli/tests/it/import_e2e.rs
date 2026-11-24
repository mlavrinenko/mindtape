//! End-to-end tests for `mt import` that shell out to the `mt` binary, the
//! only reliable way to exercise real stdin (unit tests can't fake process
//! stdin). These tests cover file-arg input too, for parity. The mapping,
//! round-trip, dry-run and failure-cleanup logic itself has unit tests in
//! `crates/mindtape-cli/src/cmd/import/tests.rs` via `cmd::import::import_records`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;
use mindtape::test_support::write_file;

/// The status legend every scaffold in this file shares.
const LEGEND: &str = "[[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

fn scaffold() -> tempfile::TempDir {
    support::scaffold_project(&format!("[tasks]\nroots = [\"tasks\"]\n{LEGEND}"))
}

const ONE_TASK_JSON: &str = r#"[{"id":"","file":"","title":"Wire import","status":{"name":"wip","date":"2026-06-01","note":null},"priority":null,"difficulty":null,"kind":null,"estimate":{},"links":[],"tags":[],"derived_tags":[],"due":null,"start":null,"reviewed":null,"body":"","extras":{}}]"#;

/// [`scaffold`] with `.` as the only task root, so a path below the project
/// stays under a root while reserved or ignored.
fn root_scaffold() -> tempfile::TempDir {
    support::scaffold_project(&format!("[tasks]\nroots = [\".\"]\n{LEGEND}"))
}

/// The path `mt` shows for `rel` under `dir`, in the native separator.
fn shown(dir: &std::path::Path, rel: &str) -> String {
    mindtape::display_path(dir.join(rel))
}

#[test]
fn imports_from_stdin() {
    let dir = scaffold();

    support::mt(dir.path())
        .arg("import")
        .write_stdin(ONE_TASK_JSON)
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-import.typ")).unwrap();
    assert!(written.contains("title: \"Wire import\""), "{written}");
    assert!(written.contains("status: wip("), "{written}");
}

#[test]
fn an_ignored_destination_is_refused_and_writes_nothing() {
    let dir = root_scaffold();
    write_file(dir.path(), ".gitignore", "target/\n");
    support::git(dir.path(), &["init", "-q"]);

    let output = support::mt(dir.path())
        .args(["import", "--dir", "target"])
        .write_stdin(ONE_TASK_JSON)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        !dir.path().join("target").exists(),
        "a refused destination must not be created"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&shown(dir.path(), "target/wire-import.typ")),
        "the refusal must name the final path: {stderr}"
    );
    assert!(stderr.contains("would be invisible"), "{stderr}");
    assert!(
        stderr.contains("--dir"),
        "the fix must offer another destination: {stderr}"
    );
}

#[test]
fn a_reserved_destination_is_refused_and_writes_nothing() {
    let dir = root_scaffold();

    let output = support::mt(dir.path())
        .args(["import", "--dir", ".mindtape/templates"])
        .write_stdin(ONE_TASK_JSON)
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        !dir.path()
            .join(".mindtape/templates/wire-import.typ")
            .exists(),
        "a refused destination must not be created"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&shown(dir.path(), ".mindtape/templates/wire-import.typ")),
        "the refusal must name the final path: {stderr}"
    );
    assert!(stderr.contains("reserved"), "{stderr}");
}

/// A dry run previews exactly what a real run would land, including the
/// destination guard's refusal, and doesn't write either way.
#[test]
fn a_preview_still_guards_and_writes_nothing() {
    let dir = root_scaffold();
    write_file(dir.path(), ".gitignore", "target/\n");
    support::git(dir.path(), &["init", "-q"]);

    support::mt(dir.path())
        .args(["import", "--dry-run", "--dir", "target"])
        .write_stdin(ONE_TASK_JSON)
        .assert()
        .failure()
        .stderr(predicates::str::contains("would be invisible"));

    assert!(!dir.path().join("target").exists());
}

/// An outside-root destination stays what it always was: a write with a
/// warning, not a refusal.
#[test]
fn an_outside_root_destination_still_only_warns() {
    let dir = scaffold();
    let outside = mindtape::test_support::tempdir();

    let output = support::mt(dir.path())
        .args(["import", "--dir", outside.path().to_str().unwrap()])
        .write_stdin(ONE_TASK_JSON)
        .assert()
        .success()
        .get_output()
        .clone();

    support::assert_exists(outside.path(), "wire-import.typ");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("outside the configured roots"), "{stderr}");
}

#[test]
fn imports_from_a_file_argument() {
    let dir = scaffold();
    let input = dir.path().join("in.json");
    std::fs::write(&input, ONE_TASK_JSON).unwrap();

    support::run_ok(dir.path(), &["import", input.to_str().unwrap()]);

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-import.typ")).unwrap();
    assert!(written.contains("title: \"Wire import\""), "{written}");

    // The filed task passes `check`.
    support::check_ok(dir.path());
}

const HELPER: &str = "#let emphasize(body) = text(weight: \"bold\", body)\n";

const HELPER_TASK: &str = "#import \"/truth/prelude/2/lib.typ\": *\n\
     #import \"@local/mindtape:0.5.0\": *\n\n\
     #show: task.with(\n  title: \"Uses the helper\",\n  status: wip(2026, 6, 1),\n)\n\n\
     = Summary\n\n#emphasize[The project helper resolves.]\n";

/// A scaffold with `emit-imports` naming `module`, holding the task
/// `(stem, source)` written under `tasks/`.
fn emitted_scaffold(module: (&str, &str), task: (&str, &str)) -> tempfile::TempDir {
    let dir = support::scaffold_project(&format!(
        "[tasks]\nroots = [\"tasks\"]\n\
         emit-imports = [\"/{}\"]\n{LEGEND}",
        module.0
    ));
    write_file(dir.path(), module.0, module.1);
    write_file(dir.path(), format!("tasks/{}.typ", task.0), task.1);
    dir
}

/// A project module that exports its own `task` and `status`.
const SHADOW: &str = "#let task(..) = [shadow]\n#let status(..) = [shadow]\n";

/// A task authored with the module imported first. The prelude keeps `task`
/// and `status`, and the file loads.
const SHADOW_TASK: &str = "#import \"/helpers/shadow.typ\": *\n\
     #import \"@local/mindtape:0.5.0\": *\n\n\
     #show: task.with(\n  title: \"Shadowed by a project helper\",\n  status: wip(2026, 6, 1),\n)\n";

/// Run `mt import` in `dir` over `input`, requiring success.
fn import_stdin(dir: &Path, input: impl Into<Vec<u8>>) {
    support::mt(dir)
        .arg("import")
        .write_stdin(input)
        .assert()
        .success();
}

/// Export the task at `tasks/<stem>.typ`, remove it, and rebuild it from the
/// export, returning the source `mt import` wrote back.
fn rebuild(dir: &Path, stem: &str) -> String {
    let exported = support::mt(dir)
        .args(["ls", "", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let path = dir.join("tasks").join(format!("{stem}.typ"));
    std::fs::remove_file(&path).unwrap();
    import_stdin(dir, exported);
    std::fs::read_to_string(&path).unwrap()
}

/// The first two lines `mt import` writes: the project import, then the
/// prelude.
fn assert_header(rebuilt: &str, project: &str) {
    let expected = [
        format!("#import \"{project}\": *"),
        "#import \"@local/mindtape:0.5.0\": *".to_owned(),
    ];
    assert_eq!(
        rebuilt
            .lines()
            .take(2)
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        expected,
        "{rebuilt}"
    );
}

/// Rebuild `tasks/<stem>.typ` from its export, assert its header names
/// `project`, assert `mt check` passes, and return the source.
fn rebuild_and_check(dir: &Path, stem: &str, project: &str) -> String {
    let rebuilt = rebuild(dir, stem);
    assert_header(&rebuilt, project);
    support::check_ok(dir);
    rebuilt
}

/// A task whose body calls a project helper keeps its `emit-imports` line
/// through `mt ls --json | mt import`, so the rebuilt file evaluates and
/// passes `mt check`.
#[test]
fn a_rebuilt_task_keeps_the_project_import_its_body_calls() {
    let dir = emitted_scaffold(
        ("truth/prelude/2/lib.typ", HELPER),
        ("uses-helper", HELPER_TASK),
    );

    let rebuilt = rebuild_and_check(dir.path(), "uses-helper", "/truth/prelude/2/lib.typ");
    assert!(
        rebuilt.contains("#emphasize[The project helper resolves.]"),
        "{rebuilt}"
    );
}

/// A project module that exports its own `task` and `status` keeps neither name
/// in a rebuilt file. The emitter writes the project imports first and the
/// prelude last, and the rebuilt task still loads with its own title.
#[test]
fn a_project_module_exporting_task_cannot_shadow_the_core() {
    let dir = emitted_scaffold(("helpers/shadow.typ", SHADOW), ("shadowed", SHADOW_TASK));

    rebuild_and_check(dir.path(), "shadowed", "/helpers/shadow.typ");
    let row = support::task_json(dir.path(), "shadowed");
    assert_eq!(
        support::at(&row, "/title").as_str(),
        Some("Shadowed by a project helper")
    );
}

/// A clean record followed by one whose status is outside the legend.
fn mixed_batch() -> String {
    let bad = ONE_TASK_JSON
        .replace("Wire import", "Bad record")
        .replace(r#""name":"wip""#, r#""name":"nonesuch""#);
    format!(
        "[{},{}]",
        &ONE_TASK_JSON[1..ONE_TASK_JSON.len() - 1],
        &bad[1..bad.len() - 1]
    )
}

/// One bad record does not refuse the batch. The good one still lands, and the
/// run exits 1 after the report. The bad one never reaches a file, so nothing
/// needs removing afterwards. The second record's status is outside the
/// legend.
#[test]
fn a_rejected_record_is_never_written_and_the_rest_still_land() {
    let dir = scaffold();
    let batch = mixed_batch();

    support::mt(dir.path())
        .arg("import")
        .write_stdin(batch)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("1 filed, 1 skipped"));

    assert!(
        dir.path().join("tasks/wire-import.typ").exists(),
        "the clean record must still land"
    );
    assert!(
        !dir.path().join("tasks/bad-record.typ").exists(),
        "a rejected record must never reach the disk"
    );
    support::check_ok(dir.path());
}

/// `--dry-run` names what it would file and says it didn't write.
#[test]
fn dry_run_says_would_file_and_writes_nothing() {
    let dir = scaffold();

    support::mt(dir.path())
        .args(["import", "--dry-run"])
        .write_stdin(ONE_TASK_JSON)
        .assert()
        .success()
        .stderr(predicates::str::contains("Wire import: would file"))
        .stderr(predicates::str::ends_with("dry run: nothing written\n"));

    assert!(!dir.path().join("tasks/wire-import.typ").exists());
}

/// A dry run that skips a record ends by saying what it would file, never
/// that it filed. A real import keeps "filed".
#[test]
fn the_failure_line_says_would_file_under_a_dry_run() {
    let dir = scaffold();
    let batch = mixed_batch();

    support::mt(dir.path())
        .args(["import", "--dry-run"])
        .write_stdin(batch.clone())
        .assert()
        .code(1)
        .stderr(predicates::str::ends_with(
            "error: would file 1 of 2 tasks; 1 failed\n",
        ));
    support::mt(dir.path())
        .arg("import")
        .write_stdin(batch)
        .assert()
        .code(1)
        .stderr(predicates::str::ends_with(
            "error: filed 1 of 2 tasks; 1 failed\n",
        ));
}

/// A content rule's detail prints under the skipped record's breach, indented,
/// as `mt check` prints it on a `note:` line.
#[test]
fn a_skipped_record_keeps_the_content_rule_detail_under_its_breach() {
    let dir = support::scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\
         [[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.body.content.rules]]\nname = \"leave-no-question-open\"\npath = [\"TO-BE\"]\n\
         assert = { density = { pattern = '\\?(\\s|$)', max = 0, per = \"sentence\" } }\n\
         message = \"ask the user and write the answer in\"\n",
    );
    let asks = ONE_TASK_JSON.replace(
        r#""body":"""#,
        r#""body":"= TO-BE\n\nShould it cover the board?\n""#,
    );

    support::mt(dir.path())
        .args(["import", "--dry-run"])
        .write_stdin(asks)
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "ask the user and write the answer in\n      note: section `TO-BE` sentence 1 \
             (\"Should it cover the board?\") must carry at most 0 matches",
        ));
}

/// A canonical record for `stem` at `file`, depending on `dep` when given.
fn record(stem: &str, file: &str, dep: Option<&str>) -> String {
    let links = dep.map_or_else(String::new, |target| {
        format!(r#"{{"kind":"depends-on","note":null,"task":"{target}"}}"#)
    });
    ONE_TASK_JSON[1..ONE_TASK_JSON.len() - 1]
        .replace(r#""id":"""#, &format!(r#""id":"{stem}""#))
        .replace(r#""file":"""#, &format!(r#""file":"{file}""#))
        .replace(r#""links":[]"#, &format!(r#""links":[{links}]"#))
        .replace("Wire import", stem)
}

#[test]
fn a_two_folder_export_imports_back_whole() {
    let dir = scaffold();
    let batch = format!(
        "[{},{}]",
        record("alpha", "tasks/a/alpha.typ", Some("../b/beta.typ")),
        record("beta", "tasks/b/beta.typ", None),
    );

    support::mt(dir.path())
        .arg("import")
        .write_stdin(batch)
        .assert()
        .success();

    support::assert_exists(dir.path(), "tasks/a/alpha.typ");
    support::assert_exists(dir.path(), "tasks/b/beta.typ");
    support::check_ok(dir.path());
}

#[test]
fn an_explicit_destination_overrides_the_record_file_for_the_whole_batch() {
    let dir = scaffold();
    let batch = format!(
        "[{},{}]",
        record("alpha", "tasks/a/alpha.typ", None),
        record("beta", "tasks/b/beta.typ", None),
    );

    support::ok_stdin(dir.path(), &["import", "--dir", "tasks/moved"], &batch);

    support::assert_exists(dir.path(), "tasks/moved/alpha.typ");
    support::assert_exists(dir.path(), "tasks/moved/beta.typ");
    assert!(!dir.path().join("tasks/a").exists());
    assert!(!dir.path().join("tasks/b").exists());
}

/// The removed flags still parse, and the command refuses them with the
/// replacement named.
#[test]
fn the_old_destination_flags_are_refused_with_a_dir_hint() {
    for (flag, value) in [("--into", "tasks"), ("--to", "target")] {
        let dir = root_scaffold();
        let output = support::mt(dir.path())
            .args(["import", flag, value])
            .write_stdin(ONE_TASK_JSON)
            .assert()
            .failure()
            .get_output()
            .clone();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(flag), "{flag}: {stderr}");
        assert!(stderr.contains("--dir"), "{flag}: {stderr}");
        assert!(!dir.path().join("tasks/wire-import.typ").exists());
    }
}

#[test]
fn a_stem_taken_in_another_folder_gets_a_suffix_and_the_report_says_so() {
    let dir = scaffold();
    let first = format!("[{}]", record("alpha", "tasks/a/alpha.typ", None));
    support::mt(dir.path())
        .arg("import")
        .write_stdin(first)
        .assert()
        .success();

    let batch = format!("[{}]", record("alpha", "tasks/b/alpha.typ", None));
    support::mt(dir.path())
        .arg("import")
        .write_stdin(batch)
        .assert()
        .success()
        .stderr(predicates::str::contains("alpha: filed as alpha-2"));

    support::assert_exists(dir.path(), "tasks/b/alpha-2.typ");
}

#[test]
fn a_file_outside_every_root_or_climbing_out_lands_in_the_first_root() {
    let dir = scaffold();
    let batch = format!(
        "[{},{},{}]",
        record("alpha", "elsewhere/alpha.typ", None),
        record("beta", "tasks/../beta.typ", None),
        record("gamma", "/tmp/gamma.typ", None),
    );

    support::mt(dir.path())
        .arg("import")
        .write_stdin(batch)
        .assert()
        .success();

    for stem in ["alpha", "beta", "gamma"] {
        assert!(
            dir.path().join(format!("tasks/{stem}.typ")).exists(),
            "{stem}"
        );
    }
}
