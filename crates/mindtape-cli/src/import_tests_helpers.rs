//! Unit tests for `import`'s internal helpers: source reading, record parsing,
//! the `--map` option parser, record labelling, and the report writer.
//!
//! Per-record problem attribution used to be tested here, against a
//! `problems_for` helper that read a post-write reload. That reload is gone —
//! the batch is checked as candidates before anything is written — and the
//! attribution it did now belongs to `mindtape::check_candidates`, tested in
//! `candidate_tests.rs` and `pre_write_tests.rs`. So is the temp project mirror
//! `--dry-run` needed: a candidate is validated without a file, so there is
//! nothing left to mirror.
//!
//! The command-level round-trip, input shapes and failure behaviour live in
//! `import_tests.rs`; everything here calls one helper directly.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::HashSet;
use std::process::Command;

use mindtape::test_support::bare_task;
use mindtape::{CanonicalTask, EntryKind, ImportReport, ReportEntry};

use super::import_records;
use super::tests::{bare_args, setup_dest, to_canonical};
use super::write::TaskOutcome;

#[test]
fn read_source_reads_a_real_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("in.json");
    std::fs::write(&path, "[]").unwrap();
    let raw = super::input::read_source(path.to_str().unwrap()).unwrap();
    assert_eq!(raw, "[]");
}

#[test]
fn parse_records_accepts_bare_array_and_envelope() {
    let array = super::input::parse_records("[]").unwrap();
    assert!(array.is_empty());

    let envelope = super::input::parse_records(r#"{"tasks": [], "applied": null}"#).unwrap();
    assert!(envelope.is_empty());
}

#[test]
fn parse_records_rejects_neither_shape() {
    let err = super::input::parse_records(r#"{"nope": []}"#).unwrap_err();
    assert!(err.to_string().contains("tasks"), "{err}");
}

// cargo-mutants regression tests -- each assertion fails under one specific
// mutation in `import.rs` and passes on the real code.

#[test]
fn map_options_rejects_empty_old_or_new() {
    let empty_old = ["=wip".to_owned()];
    let empty_new = ["todo=".to_owned()];
    for entry in [empty_old, empty_new] {
        let mut args = bare_args();
        args.map = entry.to_vec();
        assert!(super::map_options(&args).is_err());
    }
}

#[test]
fn record_label_uses_id_when_present() {
    let record = CanonicalTask {
        id: "my-id".into(),
        title: "My Title".into(),
        ..Default::default()
    };
    assert_eq!(super::record_label(&record, 0), "my-id");
}

#[test]
fn record_label_uses_title_when_id_empty() {
    let record = CanonicalTask {
        id: String::new(),
        title: "My Title".into(),
        ..Default::default()
    };
    assert_eq!(super::record_label(&record, 0), "My Title");
}

#[test]
fn record_label_falls_back_to_position_when_both_empty() {
    let record = CanonicalTask {
        id: String::new(),
        title: String::new(),
        ..Default::default()
    };
    assert_eq!(super::record_label(&record, 42), "record #43");
}

#[test]
fn default_status_flag_outside_legend_fails_import() {
    let dir = tempfile::tempdir().unwrap();
    let project = setup_dest(dir.path());
    let mut task = bare_task("needs-flag");
    task.title = "Needs Flag".to_owned();
    task.status = None;
    let raw = to_canonical(&[task], &project.config);

    let mut args = bare_args();
    args.status = Some("todo".to_owned());
    let outcomes = import_records(&project, &args, &raw).unwrap();
    assert!(!outcomes[0].filed);
}

#[test]
fn existing_stems_returns_only_typ_stems() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("alpha.typ"), "").unwrap();
    std::fs::write(dir.path().join("beta.typ"), "").unwrap();
    std::fs::write(dir.path().join("notes.md"), "").unwrap();
    let stems = super::existing_stems(dir.path());
    let expected: HashSet<String> = ["alpha".into(), "beta".into()].into_iter().collect();
    assert_eq!(stems, expected);
}

#[test]
fn entry_kind_label_returns_expected_values() {
    assert_eq!(super::entry_kind_label(EntryKind::Mapped), "mapped");
    assert_eq!(super::entry_kind_label(EntryKind::Defaulted), "defaulted");
    assert_eq!(super::entry_kind_label(EntryKind::Dropped), "dropped");
    assert_eq!(super::entry_kind_label(EntryKind::Violated), "violated");
}

// -- mutants at 224,232,242 -- suppress/corrupt report stderr output.
#[test]
fn report_emits_per_task_detail_and_extras() {
    if std::env::var("_MT_IMP").is_err() {
        let o = Command::new(std::env::current_exe().unwrap())
            .env("_MT_IMP", "1")
            .args(["report_emits_per_task_detail_and_extras", "--nocapture"])
            .stderr(std::process::Stdio::piped())
            .output()
            .unwrap();
        let s = String::from_utf8_lossy(&o.stderr);
        assert!(s.contains("fail-task: skipped"), "{s}");
        assert!(s.contains("[mapped] status: renamed todo -> wip"), "{s}");
        assert!(s.contains("extras carried:"), "{s}");
        return;
    }
    let o = TaskOutcome {
        label: "fail-task".into(),
        filed: false,
        problems: vec!["err".into()],
        report: ImportReport {
            entries: vec![ReportEntry {
                field: "status".into(),
                kind: EntryKind::Mapped,
                message: "renamed todo -> wip".into(),
            }],
            carried_extras: vec!["x-foo".into()],
        },
    };
    super::report(&[o], false);
}

fn outcome(label: &str, filed: bool) -> TaskOutcome {
    TaskOutcome {
        label: label.into(),
        filed,
        problems: vec![],
        report: ImportReport::default(),
    }
}

#[test]
fn dry_run_report_says_would_file_and_ends_with_nothing_written() {
    let outcomes = [outcome("a", true), outcome("b", false)];
    let text = super::render_report(&outcomes, true);
    assert_eq!(
        text,
        "1 would file, 1 skipped\n  a: would file\n  b: skipped\ndry run: nothing written\n"
    );
}

#[test]
fn real_report_says_filed_and_has_no_dry_run_line() {
    let outcomes = [outcome("a", true), outcome("b", false)];
    let text = super::render_report(&outcomes, false);
    assert_eq!(text, "1 filed, 1 skipped\n  a: filed\n  b: skipped\n");
}
