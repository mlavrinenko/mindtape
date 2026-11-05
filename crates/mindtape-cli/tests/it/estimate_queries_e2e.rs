//! End-to-end proof that an estimate unit takes a manual count plus file
//! queries (`truth/tasks/resolve-an-estimate-from-a-manual-count-plus-file-queries.typ`).
//!
//! Drives a real git working copy and the real prelude, so `count(...)`,
//! `grep(...)` and `glob(...)` go through the constructors a task file calls,
//! and the queries match the files git lists.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

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
        "#import \"@local/mindtape:0.4.0\": *\n\n\
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
    let estimate =
        "count(1, grep(`parse_task`, \"src/*.rs\"), glob(\"src/b.rs\"), glob(\"src/c.rs\"))";
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
        "estimate.src-files: grep `parse(` isn't a regex: unclosed group\n",
    );
}

#[test]
fn a_queries_entry_naming_an_unknown_path_set_fails_the_config() {
    let dir = fixture(&config("src-files = \"@sources\"", ""), "1");
    let (ok, text) = support::run_text(dir.path(), &["check"]);
    assert!(!ok, "{text}");
    assert!(text.contains("tasks.estimate.queries.src-files"), "{text}");
}

#[test]
fn mt_eval_shows_where_a_count_comes_from() {
    let dir = fixture(
        &config("src-files = \"@source\"", ""),
        "count(2, grep(`parse_task`, \"src/*.rs\"), glob(\"docs/**\"))",
    );
    let (ok, text) = support::run_text(dir.path(), &["eval", "rename"]);
    assert!(ok, "{text}");
    assert!(
        text.contains("Rename the parser\n\nestimate.src-files"),
        "one blank line above the breakdown: {text:?}"
    );
    assert!(
        text.contains(
            "estimate.src-files = 4\n  2        manual\n  2 files  grep `parse_task` in `src/*.rs`\n  0 files  glob `docs/**`\n"
        ),
        "{text}"
    );
    let (ok, json) = support::run_text(dir.path(), &["eval", "rename", "--json"]);
    assert!(ok, "{json}");
    let json: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(json["estimate"]["src-files"], serde_json::json!(4.0));
    assert_eq!(
        json["estimate_counts"]["src-files"]["queries"][0]["files"],
        serde_json::json!(2)
    );
}

/// `mt eval <task>` output, asserted to succeed.
fn eval_text(root: &Path, task: &str) -> String {
    let (ok, text) = support::run_text(root, &["eval", task]);
    assert!(ok, "{text}");
    text
}

/// The `ls --json` export of `root`, as its text and parsed value.
fn export(root: &Path) -> (String, serde_json::Value) {
    let (ok, text) = support::run_text(root, &["ls", "", "--json"]);
    assert!(ok, "{text}");
    let json = serde_json::from_str(&text).unwrap();
    (text, json)
}

/// Re-import `export` after removing the task file, then return what
/// `mt import` wrote back.
fn reimport(root: &Path, export: &str) -> String {
    write_file(root, "export.json", export);
    std::fs::remove_file(root.join("tasks/rename.typ")).unwrap();
    let (ok, text) = support::run_text(root, &["import", "export.json"]);
    assert!(ok, "{text}");
    std::fs::read_to_string(root.join("tasks/rename.typ")).unwrap()
}

#[test]
fn mt_ls_json_and_mt_import_give_back_the_count() {
    let dir = fixture(
        &config("src-files = \"@source\"", ""),
        "count(2, grep(`parse_task`), glob(\"src/c.rs\"))",
    );
    let root = dir.path();
    let (export, json) = export(root);
    let task = &json["tasks"][0];
    assert_eq!(task["estimate"]["src-files"], serde_json::json!(5.0));
    assert_eq!(
        task["estimate_counts"]["src-files"]["count"],
        serde_json::json!(2.0)
    );
    let written = reimport(root, &export);
    assert!(
        written.contains("src-files: count(2, grep(`parse_task`), glob(\"src/c.rs\"))"),
        "{written}"
    );
    assert_eq!(resolved(root), serde_json::json!(5.0));
}

/// A git working copy whose files call `parse_task`: `src/a.rs` twice,
/// `src/b.rs` once, `src/c.rs` once, and `src/gen/d.rs` once outside
/// `@source`.
fn ast_grep_fixture(estimate: &str) -> tempfile::TempDir {
    let dir = support::scaffold_project(&config("src-files = \"@source\"", ""));
    let root = dir.path();
    git_init(root);
    write_file(root, "tasks/rename.typ", task(estimate));
    write_file(
        root,
        "src/a.rs",
        "fn a() { parse_task(1); parse_task(2); }\n",
    );
    write_file(root, "src/b.rs", "fn b() { parse_task(3); }\n");
    write_file(root, "src/c.rs", "fn c() { parse_task(4); }\n");
    write_file(root, "src/gen/d.rs", "fn d() { parse_task(5); }\n");
    dir
}

#[test]
fn an_ast_grep_count_reads_real_call_sites_once_per_file() {
    let dir = ast_grep_fixture("count(1, ast_grep(\"rust\", \"parse_task($$$ARGS)\", \"src/**\"))");
    assert_eq!(
        support::at(
            &support::task_json(dir.path(), "rename"),
            "/estimate/src-files"
        ),
        &serde_json::json!(4.0)
    );
    let (ok, json) = support::run_text(dir.path(), &["eval", "rename", "--json"]);
    assert!(ok, "{json}");
    let json: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        json["estimate_counts"]["src-files"]["queries"][0]["files"],
        serde_json::json!(3)
    );
}

/// Assert the estimate `estimate` fails `mt check` with `message`.
fn assert_ast_grep_problem(estimate: &str, message: &str) {
    let dir = ast_grep_fixture(estimate);
    support::assert_check_fails(dir.path(), message);
}

#[test]
fn a_bad_ast_grep_pattern_is_a_violation() {
    assert_ast_grep_problem(
        "count(1, ast_grep(\"rust\", \"fn (\"))",
        "estimate.src-files: ast-grep rust `fn (` isn't a pattern: ",
    );
}

#[test]
fn an_unknown_ast_grep_language_names_the_supported_set() {
    assert_ast_grep_problem(
        "count(1, ast_grep(\"python\", \"x = 1\"))",
        "estimate.src-files: ast-grep language `python` isn't supported; supported: rust (alias rs)",
    );
}

#[test]
fn mt_eval_shows_the_ast_grep_breakdown() {
    let dir = ast_grep_fixture("count(1, ast_grep(\"rust\", \"parse_task($$$ARGS)\", \"src/**\"))");
    let text = eval_text(dir.path(), "rename");
    assert!(
        text.contains(
            "estimate.src-files = 4\n  1        manual\n  3 files  ast-grep rust `parse_task($$$ARGS)` in `src/**`\n"
        ),
        "{text}"
    );
}

#[test]
fn mt_ls_json_and_mt_import_keep_an_ast_grep_query() {
    let dir = ast_grep_fixture("count(1, ast_grep(\"rust\", \"parse_task($$$ARGS)\", \"src/**\"))");
    let root = dir.path();
    let (export, json) = export(root);
    let query = &json["tasks"][0]["estimate_counts"]["src-files"]["queries"][0];
    assert_eq!(query["query"], "ast-grep");
    assert_eq!(query["language"], "rust");
    assert_eq!(query["pattern"], "parse_task($$$ARGS)");
    assert_eq!(query["glob"], "src/**");
    let written = reimport(root, &export);
    assert!(
        written.contains("ast_grep(\"rust\", `parse_task($$$ARGS)`, \"src/**\")"),
        "{written}"
    );
    assert_eq!(resolved(root), serde_json::json!(4.0));
}
