//! End-to-end tests for `mt fix`, which rewrite an unsupported
//! `@local/mindtape` import and never touch a supported or pinned one.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use mindtape::test_support::{tempdir, write_file};

use crate::support;

/// A task importing `version`, with a body that passes the default rules.
fn task(version: &str, title: &str) -> String {
    format!(
        "#import \"@local/mindtape:{version}\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n  priority: 3,\n)\n"
    )
}

fn project() -> tempfile::TempDir {
    support::scaffold_project("[tasks]\nroots = [\"tasks\"]\n")
}

fn put(dir: &Path, rel: &str, version: &str, title: &str) -> String {
    let source = task(version, title);
    write_file(dir, rel, &source);
    source
}

/// A project holding one task at `version`, and the task's bytes.
fn project_with(version: &str) -> (tempfile::TempDir, String) {
    let dir = project();
    let before = put(dir.path(), "tasks/old.typ", version, "Old task");
    (dir, before)
}

#[cfg(unix)]
fn link(target: &Path, path: &Path) {
    std::os::unix::fs::symlink(target, path).unwrap();
}

fn read(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap()
}

/// Run `mt fix …` and assert it succeeded.
fn fixed(dir: &Path, args: &[&str]) -> String {
    let (ok, text) = support::run_text(dir, args);
    assert!(ok, "{text}");
    text
}

/// Run `mt fix …` and assert it failed, returning stderr.
fn refused(dir: &Path, args: &[&str]) -> String {
    let (code, stderr) = support::run_stderr(dir, args);
    assert_eq!(code, Some(1), "{stderr}");
    stderr
}

/// Every `needle` appears in `text`. Printed paths are compared with `/` on
/// every platform; a package spec such as `@local/mindtape` has no separator
/// and stays as written.
fn has(text: &str, needles: &[&str]) {
    let text = text.replace('\\', "/");
    for needle in needles {
        assert!(text.contains(needle), "missing {needle:?}: {text}");
    }
}

fn fixed_has(dir: &Path, args: &[&str], needles: &[&str]) -> String {
    let text = fixed(dir, args);
    has(&text, needles);
    text
}

fn refused_has(dir: &Path, args: &[&str], needles: &[&str]) {
    let stderr = refused(dir, args);
    has(&stderr, needles);
}

fn unchanged(dir: &Path, rel: &str, before: &str) {
    assert_eq!(read(dir, rel), before, "{rel} changed");
}

fn refused_unchanged(dir: &Path, args: &[&str], needles: &[&str], rel: &str, before: &str) {
    refused_has(dir, args, needles);
    unchanged(dir, rel, before);
}

#[test]
fn list_names_the_fixer() {
    let dir = tempdir();
    fixed_has(dir.path(), &["fix", "--list"], &["prelude-import"]);
}

#[test]
fn rewrites_an_unsupported_pin_and_ends_with_check() {
    let (dir, _) = project_with("0.1.0");

    fixed_has(
        dir.path(),
        &["fix"],
        &[
            "rewrite tasks/old.typ  0.1.0 -> 0.4.0",
            "rewrote 1 import",
            "ok: 1 task",
        ],
    );
    assert_eq!(read(dir.path(), "tasks/old.typ"), task("0.4.0", "Old task"));
}

#[test]
fn keeps_a_supported_old_pin() {
    let (dir, before) = project_with("0.2.0");

    fixed_has(
        dir.path(),
        &["fix"],
        &["kept", "0.2.0", "nothing to rewrite; 1 kept"],
    );
    unchanged(dir.path(), "tasks/old.typ", &before);

    fixed_has(
        dir.path(),
        &["fix", "--check"],
        &["kept    tasks/old.typ  0.2.0"],
    );
}

#[test]
fn dry_run_prints_the_edit_and_writes_nothing() {
    let (dir, before) = project_with("0.1.0");

    fixed_has(
        dir.path(),
        &["fix", "--dry-run"],
        &[
            "rewrite tasks/old.typ  0.1.0 -> 0.4.0",
            "would rewrite 1 import",
        ],
    );
    unchanged(dir.path(), "tasks/old.typ", &before);
}

#[test]
fn check_reports_one_diagnostic_per_unsupported_version() {
    let dir = project();
    let before = put(dir.path(), "tasks/a-old.typ", "0.1.0", "A");
    put(dir.path(), "tasks/b-old.typ", "0.1.0", "B");
    put(dir.path(), "tasks/c-new.typ", "99.99.99", "C");

    let stderr = refused(dir.path(), &["fix", "--check"]);
    assert_eq!(stderr.matches("error:").count(), 2, "{stderr}");
    has(
        &stderr,
        &[
            "error: tasks/a-old.typ: imports @local/mindtape:0.1.0, which this mt does not support",
            "supported: 0.2.0, 0.3.0, 0.4.0",
            "run mt fix to rewrite it to @local/mindtape:0.4.0",
            "upgrade mt to read @local/mindtape:99.99.99",
        ],
    );
    unchanged(dir.path(), "tasks/a-old.typ", &before);
}

#[test]
fn a_newer_pin_aborts_every_write() {
    let (dir, before) = project_with("0.1.0");
    put(dir.path(), "tasks/new.typ", "99.99.99", "New");

    refused_unchanged(
        dir.path(),
        &["fix"],
        &[
            "does not support",
            "upgrade mt to read @local/mindtape:99.99.99",
        ],
        "tasks/old.typ",
        &before,
    );
}

#[test]
fn a_malformed_spec_is_reported() {
    let dir = project();
    let before = put(dir.path(), "tasks/old.typ", "0.1", "Broken");

    refused_unchanged(
        dir.path(),
        &["fix"],
        &["malformed @local/mindtape import"],
        "tasks/old.typ",
        &before,
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_file_with_an_edit_is_reported() {
    let (dir, _) = project_with("0.4.0");
    let target = tempdir();
    let before = put(target.path(), "old.typ", "0.1.0", "Linked");
    link(
        &target.path().join("old.typ"),
        &dir.path().join("tasks/linked.typ"),
    );

    refused_has(dir.path(), &["fix"], &["symlinked file"]);
    assert_eq!(read(target.path(), "old.typ"), before, "target changed");
}

#[test]
fn a_project_pin_for_the_imported_version_is_unfixable() {
    let (dir, before) = project_with("0.1.0");
    write_file(
        dir.path(),
        "lib/typst.toml",
        "[package]\nname = \"mindtape\"\nversion = \"0.1.0\"\nentrypoint = \"prelude.typ\"\n",
    );

    for args in [&["fix"][..], &["fix", "--check"]] {
        refused_unchanged(dir.path(), args, &["unfixable"], "tasks/old.typ", &before);
    }
}

#[test]
fn all_scans_past_the_default_scope() {
    let dir = project();
    let before = put(dir.path(), "notes.typ", "0.1.0", "Elsewhere");

    fixed(dir.path(), &["fix"]);
    unchanged(dir.path(), "notes.typ", &before);

    fixed_has(
        dir.path(),
        &["fix", "--all"],
        &["rewrite notes.typ  0.1.0 -> 0.4.0"],
    );
    assert_eq!(read(dir.path(), "notes.typ"), task("0.4.0", "Elsewhere"));
}

#[cfg(unix)]
#[test]
fn an_unreadable_file_aborts_the_write() {
    let (dir, before) = project_with("0.1.0");
    link(
        &dir.path().join("tasks/missing-target.typ"),
        &dir.path().join("tasks/broken.typ"),
    );

    refused_unchanged(
        dir.path(),
        &["fix"],
        &["can't read"],
        "tasks/old.typ",
        &before,
    );
}
