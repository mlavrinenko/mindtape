#![allow(clippy::unwrap_used, clippy::indexing_slicing, unsafe_code)]
//! `retire` strategy tests: the status rewrite, the archive rename and the
//! inbound retarget.

use mindtape::parse_config;
use std::path::PathBuf;

use mindtape::test_support::{tempdir, tempdir_with_files, write_file};

use super::super::{retire_apply, retire_plan};
use super::gitless_tests::legend;
use super::{depends_survivor, gone_and_live, ids, survivor, task};

/// A real task artifact for the file being retired — the status rewrite needs
/// a `task.with(…)` call to target.
fn artifact() -> String {
    mindtape::typst_task!(
        "(\n  title: \"Gone\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n"
    )
    .to_owned()
}

/// The legend plus `[retire] rename = ".archive"`.
fn archive_config() -> mindtape::Config {
    parse_config(&format!("{}\n[retire]\nrename = \".archive\"\n", legend())).unwrap()
}

/// The refusal `retire_plan` gives for `files` on disk, as the user reads it.
fn retire_error(files: &[(&str, &str)]) -> String {
    let dir = tempdir_with_files(files);
    let config = parse_config(&legend()).unwrap();
    let err = retire_plan(&gone_and_live("wip"), &ids(&["gone"]), dir.path(), &config).unwrap_err();
    crate::diag::plain(&err)
}

/// `files` on disk and the rename plan that retires `gone` over them.
fn archive_plan(files: &[(&str, &str)]) -> (tempfile::TempDir, super::super::RetirePlan) {
    let dir = tempdir_with_files(files);
    let plan = retire_plan(
        &gone_and_live("wip"),
        &ids(&["gone"]),
        dir.path(),
        &archive_config(),
    )
    .unwrap();
    (dir, plan)
}

// --- retire ---

#[test]
fn retire_plan_defaults_to_first_terminal_status() {
    let dir = tempdir_with_files(&[
        ("gone.typ", &artifact()),
        ("live.typ", &survivor("depends-on(\"gone.typ\")")),
    ]);
    let root = dir.path();
    let config = parse_config(&legend()).unwrap();
    let tasks = gone_and_live("wip");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert_eq!(plan.status, "done");
    assert!(plan.rename.is_none());
    assert_eq!(plan.deletions.len(), 0);
    assert_eq!(plan.rewrites.len(), 1);
    assert_eq!(plan.rewrites[0].abs, root.join("gone.typ"));
    assert!(
        plan.rewrites[0].contents.contains("done("),
        "no status flip"
    );
}

#[test]
fn retire_plan_uses_configured_status() {
    let dir = tempdir_with_files(&[("gone.typ", &artifact())]);
    let root = dir.path();
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n[retire]\nstatus = \"cancelled\"\n",
    )
    .unwrap();
    let tasks = gone_and_live("wip");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert_eq!(plan.status, "cancelled");
    assert!(plan.rewrites[0].contents.contains("cancelled("));
}

#[test]
fn retire_plan_errors_without_terminal_status() {
    let dir = tempdir_with_files(&[("gone.typ", "gone\n")]);
    let root = dir.path();
    let config = parse_config("[[tasks.status]]\nvalue = \"wip\"\n").unwrap();
    let tasks = gone_and_live("wip");

    let err = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap_err();

    crate::diag::assert_plain(
        &err,
        "error: the status legend declares no terminal status\n\
         hint: set [retire] status, or mark a status terminal\n",
    );
}

#[test]
fn retire_plan_names_a_task_file_it_cannot_read() {
    assert_eq!(
        retire_error(&[]),
        "error: can't read: 'gone.typ': no such file or directory\n"
    );
}

#[test]
fn retire_plan_rename_moves_file_into_archive_folder() {
    let (dir, plan) = archive_plan(&[("gone.typ", &artifact())]);
    let root = dir.path();
    assert_eq!(plan.deletions.len(), 1);
    assert_eq!(plan.deletions[0].rel, PathBuf::from("gone.typ"));
    assert_eq!(plan.rewrites.len(), 1);
    assert_eq!(plan.rewrites[0].rel, PathBuf::from(".archive/gone.typ"));

    retire_apply(&plan).unwrap();

    assert!(!root.join("gone.typ").exists());
    assert!(root.join(".archive/gone.typ").exists());
}

#[test]
fn retire_plan_rename_retargets_inbound_edges() {
    // A move breaks path-based resolution, so the survivor's edge to `gone`
    // must be rewritten onto the new relative path — the tombstone leaves no
    // dangling edge.
    let (dir, plan) = archive_plan(&[("gone.typ", &artifact()), ("live.typ", &depends_survivor())]);
    let root = dir.path();
    assert_eq!(plan.link_rewrites.len(), 1);
    let rewritten = &plan.link_rewrites[0];
    assert_eq!(rewritten.rel, PathBuf::from("live.typ"));
    assert_eq!(rewritten.links, 1);
    assert!(
        rewritten
            .contents
            .contains("depends-on(\".archive/gone.typ\")"),
        "got: {}",
        rewritten.contents
    );

    retire_apply(&plan).unwrap();

    let live = std::fs::read_to_string(root.join("live.typ")).unwrap();
    assert!(
        live.contains("depends-on(\".archive/gone.typ\")"),
        "got: {live}"
    );
    assert!(root.join(".archive/gone.typ").exists());
}

#[test]
fn retire_plan_leaves_already_terminal_task_untouched() {
    // A tombstone already: the default status matches the current one, and
    // without a rename there is nothing to write at all.
    let dir = tempdir_with_files(&[("gone.typ", "gone\n")]);
    let root = dir.path();
    let config = parse_config(&legend()).unwrap();
    let tasks = gone_and_live("done");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert!(plan.rewrites.is_empty());
    assert!(plan.deletions.is_empty());
}

#[test]
fn retire_plan_applies_without_git() {
    let dir = tempdir_with_files(&[("gone.typ", &artifact())]);
    let root = dir.path();
    let config = parse_config(&legend()).unwrap();
    let tasks = gone_and_live("wip");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();
    retire_apply(&plan).unwrap();

    let contents = std::fs::read_to_string(root.join("gone.typ")).unwrap();
    assert!(contents.contains("done("), "got: {contents}");
}

// --- relative_target, via retire's inbound retarget ---
//
// A survivor in one folder pointing at a retired task in another: the common
// path prefix between `live` and `gone/.archive` is empty, so the rewrite must
// climb with `..`. A `==` -> `!=` mutant in the prefix scan counts the first
// mismatching component as common and drops the climb, yielding a path that no
// longer reaches the moved file.

#[test]
fn retire_rename_retargets_across_folders_with_a_climb() {
    let dir = tempdir();
    let root = dir.path();
    std::fs::create_dir_all(root.join("gone")).unwrap();
    std::fs::create_dir_all(root.join("live")).unwrap();
    write_file(root, "gone/gone.typ", artifact());
    write_file(root, "live/live.typ", depends_survivor());
    let config = archive_config();
    let tasks = vec![
        task("gone", "gone/gone.typ", Some("wip")),
        task("live", "live/live.typ", None),
    ];

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert_eq!(plan.link_rewrites.len(), 1);
    let rewritten = &plan.link_rewrites[0];
    assert!(
        rewritten
            .contents
            .contains("depends-on(\"../gone/.archive/gone.typ\")"),
        "the climb out of `live/` is required, got: {}",
        rewritten.contents
    );
}

#[test]
fn retire_plan_names_a_task_file_whose_status_cannot_be_rewritten() {
    let text = retire_error(&[("gone.typ", "not a task\n")]);
    assert!(
        text.starts_with("error: can't rewrite the status: 'gone.typ'"),
        "got: {text}"
    );
}

#[test]
fn retire_rename_retargets_only_the_edges_to_the_moved_task() {
    let live = survivor("depends-on(\"gone.typ\") + related(\"other.typ\")");
    let (_dir, plan) = archive_plan(&[("gone.typ", &artifact()), ("live.typ", &live)]);

    let rewritten = &plan.link_rewrites[0];
    assert_eq!(rewritten.links, 1);
    let got = &rewritten.contents;
    let moved = got.contains("depends-on(\".archive/gone.typ\")");
    assert!(
        moved && got.contains("related(\"other.typ\")"),
        "got: {got}"
    );
}

#[test]
fn retire_apply_names_a_folder_it_cannot_create() {
    // The would-be folder's parent is a regular file, so create_dir_all fails.
    let dir = tempdir_with_files(&[("blocker", "a file, not a folder\n")]);
    let abs = dir.path().join("blocker/.archive/gone.typ");
    let plan = super::super::RetirePlan {
        status: "done".to_owned(),
        rename: None,
        rewrites: vec![typst_cleanup::Rewrite {
            abs,
            rel: PathBuf::from("gone.typ"),
            contents: String::new(),
            links: 0,
        }],
        link_rewrites: vec![],
        deletions: vec![],
    };

    let err = retire_apply(&plan).unwrap_err();

    let text = crate::diag::plain(&err);
    assert!(
        text.starts_with("error: can't create folder: '") && text.contains(".archive'"),
        "got: {text}"
    );
}
