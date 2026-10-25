#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{EvalArgs, locate, run};
use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;

const TASK: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

/// `mt eval <file>` with the path spelled as the task reference.
fn eval(file: &std::path::Path, json: bool) -> crate::diag::Result<()> {
    run(&EvalArgs {
        task: file.to_string_lossy().into_owned().into(),
        json,
    })
}

#[test]
fn eval_valid_file_table_succeeds() {
    let dir = tempdir();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, TASK).unwrap();
    assert!(eval(&file, false).is_ok());
}

#[test]
fn eval_valid_file_json_succeeds() {
    let dir = tempdir();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, TASK).unwrap();
    assert!(eval(&file, true).is_ok());
}

#[test]
fn eval_missing_file_returns_err() {
    // A path under a fresh tempdir that was never written: nonexistent by
    // construction on every platform, unlike a hardcoded `/tmp/...`.
    let dir = tempdir();
    let file = dir.path().join("no_such_file.typ");
    assert!(eval(&file, false).is_err());
}

#[test]
fn eval_file_without_marker_returns_err() {
    let dir = tempdir();
    let file = dir.path().join("notes.typ");
    std::fs::write(&file, "= Just notes\n\nNo marker here.\n").unwrap();
    assert!(eval(&file, false).is_err());
}

#[test]
fn build_world_missing_file_returns_err() {
    let dir = tempdir();
    let file = dir.path().join("no_such_file.typ");
    assert!(super::build_world(&file).is_err());
}

/// A `@local` package on disk: manifest plus the entrypoint it names. The
/// stub emits a `mindtape.task` marker directly, so a task importing it
/// harvests without the real prelude.
fn write_pkg(root: &std::path::Path, dir_rel: &str, name: &str) {
    let pkg = root.join(dir_rel);
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(
        pkg.join("typst.toml"),
        // A version no install would ship, so these stay hermetic whatever the
        // machine has under its own local-package directory.
        format!("[package]\nname = \"{name}\"\nversion = \"99.99.99\"\nentrypoint = \"lib.typ\"\n"),
    )
    .unwrap();
    write_file(
        &pkg,
        "lib.typ",
        "#let mk(t) = [#metadata((marker: \"mindtape.task\", fields: (title: t), extras: (:))) <mindtape>]\n",
    );
}

/// Write a task file importing `@local/mindtape` under `root`, returning it.
fn write_importing_task(root: &std::path::Path) -> std::path::PathBuf {
    let file = root.join("tasks").join("t.typ");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(
        &file,
        "#import \"@local/mindtape:99.99.99\": *\n#mk(\"Hi\")\n",
    )
    .unwrap();
    file
}

#[test]
fn eval_registers_an_in_tree_lib_named_mindtape() {
    let dir = tempdir();
    write_pkg(dir.path(), "lib", "mindtape");
    let file = write_importing_task(dir.path());
    assert!(eval(&file, true).is_ok());
}

#[test]
fn eval_does_not_hijack_a_foreign_lib_package() {
    // `<root>/lib` is some other project's Typst package, and the project
    // points `@local/mindtape` elsewhere. Keying the override off the mere
    // existence of `lib/typst.toml` resolved the import to `lib/` and failed
    // on its declared name; the manifest name is what decides.
    let dir = tempdir();
    write_pkg(dir.path(), "lib", "other");
    write_pkg(dir.path(), "vendor/mindtape", "mindtape");
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape").join("config.toml"),
        "[[typst.local-packages]]\nname = \"mindtape\"\npath = \"vendor/mindtape\"\n",
    )
    .unwrap();
    let file = write_importing_task(dir.path());
    assert!(eval(&file, true).is_ok());
}

// --- locate ---
//
// `locate` treats a path-spelled reference as a path (so it bails when the file
// is absent) and only falls through to project resolution for a bare stem or
// `#` handle. The two spellings below each take one side of
// `contains(['/', '\\']) || ends_with(".typ")`, so a `||` -> `&&` mutant —
// which lets both fall through to `resolve_task` and answers with a different
// error — fails the exact-message assertion.

#[test]
fn locate_refuses_a_missing_path_with_a_separator() {
    let err = locate("sub/nope").unwrap_err();
    crate::diag::assert_plain(&err, "error: no such file: 'sub/nope'\n");
}

#[test]
fn locate_refuses_a_missing_typ_path() {
    let err = locate("nope.typ").unwrap_err();
    crate::diag::assert_plain(&err, "error: no such file: 'nope.typ'\n");
}

// --- the breakdown of a count(...) estimate ---

/// A task whose `src-files` resolved to 2 manual plus 3 files, from a grep
/// that matched 3 and a glob that matched none.
fn counted() -> mindtape::Task {
    use mindtape::estimate_queries::{CountEstimate, Query};
    let mut task = mindtape::test_support::bare_task("t");
    task.estimate.insert("src-files".to_owned(), 5.0);
    task.estimate.insert("hours".to_owned(), 1.0);
    // jscpd:ignore-start
    // The same two queries the library's parse test reads, as plain data.
    let count = CountEstimate {
        manual: 2.0,
        queries: vec![
            Query::Grep {
                regex: "PathFilter".to_owned(),
                glob: Some("crates/**".to_owned()),
            },
            Query::Glob {
                pattern: "docs/*.md".to_owned(),
            },
        ],
        matches: vec![3, 0],
        problem: None,
    };
    // jscpd:ignore-end
    task.estimate_counts.insert("src-files".to_owned(), count);
    task
}

#[test]
fn a_count_estimate_prints_its_manual_part_each_query_and_the_total() {
    assert_eq!(
        super::breakdown(&counted()),
        "estimate.src-files = 5\n\
         \x20 2        manual\n\
         \x20 3 files  grep `PathFilter` in `crates/**`\n\
         \x20 0 files  glob `docs/*.md`\n"
    );
}

#[test]
fn a_count_that_did_not_resolve_prints_why() {
    let mut task = counted();
    task.estimate.remove("src-files");
    let count = task.estimate_counts.get_mut("src-files").unwrap();
    count.matches.clear();
    count.problem = Some("no path set is named `@src`".to_owned());
    assert_eq!(
        super::breakdown(&task),
        "estimate.src-files: no path set is named `@src`\n"
    );
}

#[test]
fn a_plain_estimate_prints_no_breakdown() {
    let mut task = counted();
    task.estimate_counts.clear();
    assert_eq!(super::breakdown(&task), "");
}

#[test]
fn the_json_breakdown_sits_beside_the_resolved_number() {
    let json = crate::render::output::task_to_json(&counted(), &mindtape::Config::default());
    assert_eq!(json["estimate"]["src-files"], serde_json::json!(5.0));
    assert_eq!(
        json["estimate_counts"],
        serde_json::json!({
            "src-files": {
                "count": 2.0,
                "queries": [
                    {"query": "grep", "regex": "PathFilter", "glob": "crates/**", "files": 3},
                    {"query": "glob", "pattern": "docs/*.md", "files": 0}
                ],
                "problem": null
            }
        })
    );
}

#[test]
fn a_count_under_no_project_says_its_unit_takes_a_plain_number() {
    let dir = tempdir();
    let file = dir.path().join("t.typ");
    std::fs::write(&file, TASK).unwrap();
    let mut task = counted();
    super::resolve_counts(&mut task, &file, None);
    assert_eq!(
        super::breakdown(&task),
        "estimate.src-files: `src-files` takes a plain number; name its path set under [tasks.estimate.queries] to count files\n"
    );
}
