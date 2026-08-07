//! cargo-mutants regression tests for the `--advance` chain:
//! `check_advance_preconditions`, `do_skip`, `do_advance`,
//! `build_advance_chain`, and `write_advance_chain`. Each test targets one or
//! more surviving mutants listed in the task. Core planning-helper tests
//! (`plan_next`, `occurrence_index`, the double-spawn guard) live in
//! `recur_tests.rs`, whose `weekly()` fixture this shares.
#![allow(
    clippy::unwrap_used,
    clippy::many_single_char_names,
    clippy::indexing_slicing
)]

use std::path::Path;

use mindtape::test_support::bare_task;
use mindtape::{LoadOutcome, parse_config};

use super::tests::weekly;
use super::*;
use crate::util::Project;

// Constants and helpers shared across the targeted tests:

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

const RECUR_TASK_SRC: &str = "#import \"@local/mindtape:0.2.0\": *\n\
     #show: task.with(\n  title: \"Chore\",\n  status: wip(2026, 6, 1),\n  \
     due: on(2026, 6, 15),\n  start: on(2026, 6, 10),\n  \
     recur: every(weeks: 1, anchor: \"due\", count: 3),\n)\n\n== Summary\n\nBody.\n";

fn setup(dir: &Path, files: &[(&str, &str)], config_text: &str) -> Project {
    crate::test_util::make_project_with_lib(dir, PRELUDE, files, config_text)
}

// === check_advance_preconditions ============================================
// These functions are pure (no file I/O) so the tests use minimal in-memory
// Project setup without task files.

/// A project with only `max-advance` configured, plus a `RecurArgs` for
/// `task: "chore"` requesting `--advance advance`. Shared setup for the
/// `check_advance_preconditions` regression tests below.
fn advance_precondition_fixture(
    max_advance: u32,
    advance: u32,
) -> (tempfile::TempDir, Project, RecurArgs) {
    let dir = tempfile::tempdir().unwrap();
    let config_text =
        format!("[[tasks.status]]\nvalue = \"wip\"\n[tasks.recur]\nmax-advance = {max_advance}\n");
    let project = setup(dir.path(), &[], &config_text);
    let args = RecurArgs {
        task: "chore".into(),
        dry_run: false,
        advance: Some(advance),
        skip: false,
        force: false,
    };
    (dir, project, args)
}

#[test]
fn check_advance_preconditions_rejects_non_spawn() {
    // Kills:
    //   check_advance_preconditions -> Result<()> with Ok(())
    //   replace != with == in check_advance_preconditions
    // Real behaviour: materialize != "spawn" → bail.
    // Ok(()) mutant: returns Ok → test catches (expects Err).
    // == mutant: materialize == "spawn" is false for "mutate" → no bail → Ok.
    let (_dir, project, args) = advance_precondition_fixture(52, 1);
    let recur = Recur {
        materialize: Some("mutate".into()),
        ..weekly(None, None)
    };
    let task = bare_task("chore");
    let err = check_advance_preconditions(&project, &task, &recur, &args, 1).unwrap_err();
    assert!(
        err.to_string().contains("spawn"),
        "expected spawn-materialize error, got: {err}"
    );
}

#[test]
fn check_advance_preconditions_rejects_excessive_count() {
    // Kills replace > with == and replace > with <
    // Real: 10 > 5 → true → bail.
    // == mutant: 10 == 5 → false → no bail → Ok → test catches.
    // < mutant:  10 < 5  → false → no bail → Ok → test catches.
    // >= mutant: 10 >= 5 → true  → bail → not killed here (separate test).
    let (_dir, project, args) = advance_precondition_fixture(5, 10);
    let task = bare_task("chore");
    let recur = weekly(None, None);
    let err = check_advance_preconditions(&project, &task, &recur, &args, 10).unwrap_err();
    assert!(
        err.to_string().contains("max-advance"),
        "expected max-advance error, got: {err}"
    );
}

#[test]
fn check_advance_preconditions_passes_at_max_advance_boundary() {
    // Kills replace > with >=
    // Real: 5 > 5 → false → Ok.
    // >= mutant: 5 >= 5 → true → bail → test catches (expects Ok).
    let (_dir, project, args) = advance_precondition_fixture(5, 5);
    let task = bare_task("chore");
    let recur = weekly(None, None);
    assert!(check_advance_preconditions(&project, &task, &recur, &args, 5).is_ok());
}

/// A project with one loaded recurring task (`chore.typ`, from
/// `RECUR_TASK_SRC`), plus that task's absolute file path and parsed `Recur`.
/// Shared setup for the `do_skip`/`do_advance`/`build_advance_chain`/
/// `write_advance_chain` regression tests below.
fn advance_fixture() -> (tempfile::TempDir, Project, Task, PathBuf, Recur) {
    let dir = tempfile::tempdir().unwrap();
    let config_text = "[[tasks.status]]\nvalue = \"wip\"\n\
                       [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
                       [tasks.recur]\nmax-advance = 52\n";
    let project = setup(dir.path(), &[("chore.typ", RECUR_TASK_SRC)], config_text);
    assert_eq!(project.outcome.tasks.len(), 1, "fixture must load");

    let task = project.outcome.tasks[0].clone();
    let abs = project.root.join(&task.file);
    let recur = task.recur.clone().unwrap();
    (dir, project, task, abs, recur)
}

// === do_skip (survivor 1) ===================================================
// Pins replace do_skip -> Result<()> with Ok(()).

#[test]
fn do_skip_advances_due_and_decrements_count() {
    // Real do_skip reads the file, edits due/start/count, and writes back.
    // The Ok(()) mutant does none of that. Verify side effects.
    let (_dir, project, task, abs, recur) = advance_fixture();

    let original = std::fs::read_to_string(&abs).unwrap();

    do_skip(&project, &task, &abs, &recur, false).unwrap();

    let content = std::fs::read_to_string(&abs).unwrap();
    // File must be modified by do_skip (kills the Ok(()) mutant)
    assert_ne!(original, content, "file must change after do_skip");
    // Original count 3 → must be decremented
    assert!(
        content.contains("count: 2"),
        "count not decremented: {content}"
    );
}

// === do_advance (survivor 2) ================================================
// Pins replace do_advance -> Result<()> with Ok(()).

#[test]
fn do_advance_spawns_files() {
    // Real do_advance pre-materializes future spawns as new files.
    // The Ok(()) mutant produces no files.
    // Use advance=1 so the single spawn passes verify_written (no double
    // recurred-from edge).
    let (_dir, project, task, abs, recur) = advance_fixture();
    let args = RecurArgs {
        task: "chore".into(),
        dry_run: false,
        advance: Some(1),
        skip: false,
        force: false,
    };

    do_advance(&project, &task, &abs, &recur, &args).unwrap();

    // Should have created 1 new .typ file in the tasks directory
    let tasks_dir = project.root.join("tasks");
    let typ_files: Vec<_> = std::fs::read_dir(&tasks_dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "typ"))
        .collect();
    assert_eq!(
        typ_files.len(),
        2,
        "expected 2 total .typ files (original + 1 spawn)"
    );
}

// === build_advance_chain ====================================================
// These set up a minimal project manually (file on disk, in-memory outcome)
// to avoid the full Typst-eval path — build_advance_chain only reads the raw
// source text and works on the parse plane.

const SPAWN_SRC: &str = "#show: task.with(\n  title: \"Chore\",\n  status: wip(2026, 6, 1),\n  \
     due: on(2026, 6, 15),\n  start: on(2026, 6, 10),\n  \
     recur: every(weeks: 1, count: 10),\n)\n\n== Summary\n\nBody.\n";

fn chain_project(dir: &Path) -> (Project, Task, PathBuf, Recur) {
    let task_dir = dir.join("tasks");
    std::fs::create_dir_all(&task_dir).unwrap();
    let file_path = task_dir.join("chore.typ");
    std::fs::write(&file_path, SPAWN_SRC).unwrap();

    let config_text = "[[tasks.status]]\nvalue = \"wip\"\n\
                       [tasks.naming]\nstem = \"{n}\"\n";
    let config = parse_config(config_text).unwrap();
    let mut task = bare_task("chore");
    task.due = Some("2026-06-15".to_owned());
    task.start = Some("2026-06-10".to_owned());
    task.recur = Some(weekly(Some(10), None));

    let project = Project {
        root: dir.to_path_buf(),
        config,
        outcome: LoadOutcome {
            tasks: vec![task.clone()],
            errors: vec![],
        },
    };
    let abs = dir.join("tasks/chore.typ");
    let recur = task.recur.clone().unwrap();
    (project, task, abs, recur)
}

#[test]
fn build_advance_chain_returns_correct_count() {
    // Kills all three Ok(…) replacements.
    // Real: returns `count` entries.  Mutants return 0 or 1 entry.
    let dir = tempfile::tempdir().unwrap();
    let (project, task, abs, recur) = chain_project(dir.path());
    let result = build_advance_chain(&project, &task, &abs, &recur, 2).unwrap();
    assert_eq!(result.len(), 2, "expected 2 entries, got {}", result.len());
    // Each entry must have distinct content (different due dates)
    assert_ne!(result[0].2, result[1].2, "spawn content must differ");
}

#[test]
fn build_advance_chain_rejects_exhausted_with_empty_spawned() {
    // Kills !spawned.is_empty() → true  and  delete !
    // Real: plan_next errors on first iteration (exhausted, spawned empty)
    //       → !spawned.is_empty() is false → return Err(err).
    // →true mutant:  true → early stop → Ok(vec![]).
    // delete !:      spawned.is_empty() is true → early stop → Ok(vec![]).
    let dir = tempfile::tempdir().unwrap();
    let (project, task, abs, mut recur) = chain_project(dir.path());
    recur.count = Some(0); // first plan_next call sees exhausted

    let result = build_advance_chain(&project, &task, &abs, &recur, 1);
    assert!(
        result.is_err(),
        "expected Err for exhausted recur with empty spawned"
    );
}

#[test]
fn build_advance_chain_stops_early_with_partial() {
    // Kills !spawned.is_empty() → false
    // Real: spawn 1 entry, then plan_next errors (exhausted).
    //       spawned is NOT empty → early stop → Ok(vec![1 entry]).
    // →false mutant: → return Err.
    let dir = tempfile::tempdir().unwrap();
    let (project, task, abs, mut recur) = chain_project(dir.path());
    recur.count = Some(1); // one successful iteration, then exhausted

    let result = build_advance_chain(&project, &task, &abs, &recur, 2).unwrap();
    assert_eq!(result.len(), 1, "expected 1 partial entry");
}

#[test]
fn build_advance_chain_increments_occurrence() {
    // Kills occurrence += 1 replaced with -= 1  and  *= 1
    // Uses {n} stem template so occurrence appears in the filename stem.
    // Real: occurrence 1 → 2 (stems "1", "2").
    // -= mutant: 1 → 0 (stems "1", "0").
    // *= mutant: 1 → 1 (duplicate stems "1", "1").
    let dir = tempfile::tempdir().unwrap();
    let (project, task, abs, recur) = chain_project(dir.path());
    let result = build_advance_chain(&project, &task, &abs, &recur, 2).unwrap();
    assert_eq!(result.len(), 2);

    // Unique paths kills the *= mutant (which produces duplicate stems).
    let mut paths: Vec<_> = result.iter().map(|(p, _, _)| p.as_path()).collect();
    paths.sort();
    paths.dedup();
    assert_eq!(paths.len(), 2, "stems must be unique");

    // Real occurrence progression: 1 then 2.
    let stem1 = result[0]
        .0
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let stem2 = result[1]
        .0
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert_eq!(stem1, "1", "first spawn occurrence");
    assert_eq!(stem2, "2", "second spawn occurrence");
}

// === write_advance_chain ====================================================

#[test]
fn write_advance_chain_writes_files() {
    // Kills write_advance_chain -> Result<()> with Ok(()).
    // Real: writes each entry to disk.  Ok(()) mutant writes nothing.
    // Single entry so verification passes (no double-recurred-from edge).
    let (_dir, project, task, abs, recur) = advance_fixture();
    let entries = build_advance_chain(&project, &task, &abs, &recur, 1).unwrap();
    assert_eq!(entries.len(), 1);

    write_advance_chain(&project, &entries, false).unwrap();

    for (path, _, _) in &entries {
        assert!(path.exists(), "spawned file missing: {}", path.display());
    }
}

#[test]
fn write_advance_chain_skips_verification_on_dry_run() {
    // Kills delete ! in !dry_run.
    // Real: dry_run=true skips the verification loop entirely → Ok.
    // Mutant (if dry_run instead of if !dry_run): tries to verify every
    // entry's non-existent file → load_project finds no task → bail → Err.
    let (_dir, project, task, abs, recur) = advance_fixture();
    let entries = build_advance_chain(&project, &task, &abs, &recur, 1).unwrap();
    assert_eq!(entries.len(), 1);

    // Real: dry_run=true → prints, skips verify, returns Ok.
    // Mutant: dry_run=true → tries verify on non-existent file → Err.
    write_advance_chain(&project, &entries, true).unwrap();
}
