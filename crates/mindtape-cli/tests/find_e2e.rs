//! End-to-end tests for `mt find` (`truth/tasks/mt-find-as-sugar-over-contains.typ`):
//! sugar over `contains()` that shares `mt ls`'s code path — every flag
//! forwards, and the desugared filter surfaces in the `applied` facet like
//! any manual `mt ls '...'` filter would.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

fn scaffold() -> tempfile::TempDir {
    support::project_with_tasks(&[
        ("login.typ", "fix login flow"),
        ("bug.typ", "squash a bug"),
        ("both.typ", "login bug triage"),
        ("neither.typ", "unrelated cleanup"),
    ])
}

fn find_paths(dir: &Path, terms: &[&str]) -> Vec<String> {
    let mut args: Vec<&str> = vec!["find"];
    args.extend_from_slice(terms);
    args.extend(["--format", "path"]);
    let out = support::mt(dir).args(&args).output().unwrap();
    let mut lines: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    lines.sort_unstable();
    lines
}

#[test]
fn find_one_term_matches_every_task_naming_it() {
    let dir = scaffold();
    assert_eq!(
        find_paths(dir.path(), &["login"]),
        vec!["tasks/both.typ", "tasks/login.typ"]
    );
}

#[test]
fn find_several_terms_conjoin() {
    let dir = scaffold();
    assert_eq!(
        find_paths(dir.path(), &["login", "bug"]),
        vec!["tasks/both.typ"]
    );
}

#[test]
fn find_json_reports_the_desugared_filter_in_applied() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .args(["find", "bug", "--json"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(support::at(&json, "/applied/facets/filter/from"), "manual");
    assert_eq!(
        support::at(&json, "/applied/facets/filter/value"),
        "contains(\"bug\")"
    );
}

#[test]
fn find_table_footer_names_the_applied_filter() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .args(["find", "bug"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("applied: manual (filter)"));
}

#[test]
fn find_forwards_limit() {
    let dir = scaffold();
    // An empty needle matches every task (`contains("")` is trivially true),
    // isolating `-n` as the only thing narrowing the result.
    let out = support::mt(dir.path())
        .args(["find", "", "--format", "path", "-n", "2"])
        .output()
        .unwrap();
    let count = String::from_utf8_lossy(&out.stdout).lines().count();
    assert_eq!(count, 2);
}

#[test]
fn find_forwards_sort() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .args(["find", "", "--format", "path", "--sort", "title:asc"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let lines: Vec<&str> = std::str::from_utf8(&out.stdout).unwrap().lines().collect();
    assert_eq!(lines.len(), 4);
}

#[test]
fn find_has_no_in_flag() {
    let dir = scaffold();
    let out = support::mt(dir.path())
        .args(["find", "bug", "--in", "title"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "no --in flag exists on find");
}
