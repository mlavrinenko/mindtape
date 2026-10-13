//! End-to-end tests for `mt import` that shell out to the `mt` binary — the
//! only reliable way to exercise real stdin (unit tests can't fake process
//! stdin). File-arg input is covered here too for parity; the mapping/
//! round-trip/dry-run/failure-cleanup logic itself is unit-tested in
//! `crates/mindtape-cli/src/cmd/import/tests.rs` via `cmd::import::import_records`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

fn scaffold() -> tempfile::TempDir {
    support::scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\
         [[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n",
    )
}

const ONE_TASK_JSON: &str = r#"[{"id":"","file":"","title":"Wire import","status":{"name":"wip","date":"2026-06-01","note":null},"priority":null,"difficulty":null,"kind":null,"estimate":{},"links":[],"tags":[],"derived_tags":[],"due":null,"start":null,"reviewed":null,"body":"","extras":{}}]"#;

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
fn imports_from_a_file_argument() {
    let dir = scaffold();
    let input = dir.path().join("in.json");
    std::fs::write(&input, ONE_TASK_JSON).unwrap();

    support::mt(dir.path())
        .args(["import", input.to_str().unwrap()])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-import.typ")).unwrap();
    assert!(written.contains("title: \"Wire import\""), "{written}");

    // The filed task passes `check`.
    support::mt(dir.path()).arg("check").assert().success();
}

/// One bad record does not refuse the batch — the good one still lands, and the
/// run exits 1 after the report — and the bad one is never written, so no file has to be removed afterwards. The
/// second record's status is outside the legend.
#[test]
fn a_rejected_record_is_never_written_and_the_rest_still_land() {
    let dir = scaffold();
    let bad = ONE_TASK_JSON
        .replace("Wire import", "Bad record")
        .replace(r#""name":"wip""#, r#""name":"nonesuch""#);
    let batch = format!(
        "[{},{}]",
        &ONE_TASK_JSON[1..ONE_TASK_JSON.len() - 1],
        &bad[1..bad.len() - 1]
    );

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
    support::mt(dir.path()).arg("check").assert().success();
}

/// `--dry-run` names what it would file and says it wrote nothing.
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

    assert!(dir.path().join("tasks/a/alpha.typ").exists());
    assert!(dir.path().join("tasks/b/beta.typ").exists());
    support::mt(dir.path()).arg("check").assert().success();
}

#[test]
fn an_explicit_destination_overrides_the_record_file() {
    let dir = scaffold();
    let batch = format!("[{}]", record("alpha", "tasks/a/alpha.typ", None));

    support::mt(dir.path())
        .args(["import", "--to", "moved"])
        .write_stdin(batch)
        .assert()
        .success();

    assert!(dir.path().join("tasks/moved/alpha.typ").exists());
    assert!(!dir.path().join("tasks/a").exists());
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

    assert!(dir.path().join("tasks/b/alpha-2.typ").exists());
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
