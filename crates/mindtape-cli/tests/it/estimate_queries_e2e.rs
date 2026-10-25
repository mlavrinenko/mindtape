//! End-to-end proof that an estimate unit takes a manual count plus file
//! queries (`truth/tasks/resolve-an-estimate-from-a-manual-count-plus-file-queries.typ`).
//!
//! Drives a real git working copy and the real prelude, so `count(...)`,
//! `grep(...)` and `glob(...)` go through the constructors a task file calls,
//! and the queries match the files git lists.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::test_support::{git_init, write_file};

use crate::support;

/// A project declaring a `src-files` unit, with `queries` as its
/// `[tasks.estimate.queries]` body and `rules` appended verbatim.
fn config(queries: &str, rules: &str) -> String {
    format!(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[tasks.status]]\nvalue = \"wip\"\n\
         [[tasks.status]]\nvalue = \"done\"\nterminal = true\n\n\
         [tasks.estimate]\nunits = [\"src-files\"]\n\n\
         [tasks.estimate.queries]\n{queries}\n\n\
         [dicts.source]\ncase = \"exact\"\nvalues = [\"src/**\", \"!src/gen/**\"]\n\n{rules}"
    )
}

/// The cap a resolved count is held to.
const CAP: &str = "[[tasks.rules]]\nname = \"cap\"\n\
     assert = 'estimate.src-files <= 4'\nseverity = \"error\"\n";

/// A `wip` task whose estimate is `estimate`, through the real prelude.
fn task(estimate: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Rename the parser\",\n  status: wip(2026, 10, 25),\n  \
         estimate: (src-files: {estimate}),\n)\n"
    )
}

/// A git working copy under `config` whose one task estimates `estimate`.
/// `src/a.rs` and `src/b.rs` name `parse_task`, `src/c.rs` doesn't, and
/// `src/gen/d.rs` names it outside the `source` path set.
fn fixture(config: &str, estimate: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(config);
    let root = dir.path();
    git_init(root);
    write_file(root, "tasks/rename.typ", task(estimate));
    write_file(root, "src/a.rs", "fn parse_task() {}\n");
    write_file(root, "src/b.rs", "use a::parse_task;\nfn main() {}\n");
    write_file(root, "src/c.rs", "fn other() {}\n");
    write_file(root, "src/gen/d.rs", "parse_task();\n");
    dir
}

/// The resolved `src-files` count `mt ls --json` reports for the task.
fn resolved(root: &Path) -> serde_json::Value {
    support::at(&support::task_json(root, "rename"), "/estimate/src-files").clone()
}

/// What `estimate` resolves to when `src-files` counts inside `@source`.
fn resolves(estimate: &str) -> serde_json::Value {
    let dir = fixture(&config("src-files = \"@source\"", ""), estimate);
    resolved(dir.path())
}

#[test]
fn a_count_resolves_to_the_manual_part_plus_the_files_its_queries_match() {
    assert_eq!(
        resolves("count(2, grep(`parse_task`))"),
        serde_json::json!(4.0)
    );
}

#[test]
fn a_file_two_queries_match_counts_once() {
    let estimate = "count(1, grep(`parse_task`, within: \"src/*.rs\"), glob(\"src/b.rs\"), glob(\"src/c.rs\"))";
    assert_eq!(resolves(estimate), serde_json::json!(4.0));
}

#[test]
fn a_rule_reads_the_resolved_count() {
    let dir = fixture(
        &config("src-files = \"@source\"", CAP),
        "count(3, grep(`parse_task`))",
    );
    support::assert_check_fails(
        dir.path(),
        "rules.cap: must satisfy `estimate.src-files <= 4`",
    );
}

#[test]
fn a_plain_number_still_works() {
    let dir = fixture(&config("src-files = \"@source\"", CAP), "3");
    let (ok, text) = support::run_text(dir.path(), &["check"]);
    assert!(ok, "{text}");
    assert_eq!(resolved(dir.path()), serde_json::json!(3.0));
}

#[test]
fn a_unit_with_no_queries_entry_refuses_a_count() {
    let dir = fixture(&config("", ""), "count(2, grep(`parse_task`))");
    support::assert_check_fails(
        dir.path(),
        "estimate.src-files: `src-files` takes a plain number; name its path set under [tasks.estimate.queries] to count files",
    );
}

#[test]
fn a_regex_that_does_not_compile_is_a_violation() {
    let dir = fixture(
        &config("src-files = \"@source\"", ""),
        "count(2, grep(`parse(`))",
    );
    support::assert_check_fails(
        dir.path(),
        "estimate.src-files: grep `parse(` isn't a regex",
    );
}

#[test]
fn a_queries_entry_naming_an_unknown_path_set_fails_the_config() {
    let dir = fixture(&config("src-files = \"@sources\"", ""), "1");
    let (ok, text) = support::run_text(dir.path(), &["check"]);
    assert!(!ok, "{text}");
    assert!(text.contains("tasks.estimate.queries.src-files"), "{text}");
}
