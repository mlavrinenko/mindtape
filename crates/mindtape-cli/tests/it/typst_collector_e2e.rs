//! End-to-end tests for a `query = { typst, function }` collector: a project
//! function over the whole task or one field, its value in `mt ls --json` and
//! `mt eval`, and its failure as a collector violation.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::disallowed_methods
)]

use mindtape::test_support::write_file;

use crate::support;

use support::{mt, run_text, scaffold_project, stdout};

const LIB: &str = r#"
#let title(task) = task.title
#let body(value) = value
#let boom(task) = panic("boom")
"#;

const TASK: &str = "#import \"@local/mindtape:0.3.0\": *\n\n\
#show: task.with(\n  title: \"A typed task\",\n  status: wip(2026, 11, 4),\n)\n\n\
== Changes\n\n- `src/a.rs`\n";

fn project(config: &str) -> tempfile::TempDir {
    let dir = scaffold_project(config);
    write_file(dir.path(), "lib.typ", LIB);
    write_file(dir.path(), "tasks/subject.typ", TASK);
    dir
}

fn json(text: &str) -> serde_json::Value {
    serde_json::from_str(text).unwrap()
}

#[test]
fn a_typst_collector_publishes_its_value() {
    let dir = project(
        r#"[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "title"
query = { typst = "lib.typ", function = "title" }
returns = "string"

[[tasks.collectors]]
name = "body"
query = { typst = "lib.typ", function = "body", field = "body" }
returns = "string"
"#,
    );
    let listing = json(&stdout(dir.path(), &["ls", "has(derived.title)", "--json"]));
    assert_eq!(
        listing["tasks"][0]["derived"]["title"], "A typed task",
        "{listing}"
    );
    assert!(
        listing["tasks"][0]["derived"]["body"]
            .as_str()
            .unwrap()
            .contains("== Changes"),
        "{listing}"
    );
    let evaluated = json(&stdout(dir.path(), &["eval", "subject", "--json"]));
    assert_eq!(evaluated["derived"]["title"], "A typed task", "{evaluated}");
}

#[test]
fn a_failing_typst_collector_is_a_check_violation_and_an_eval_note() {
    let dir = project(
        r#"[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "boom"
query = { typst = "lib.typ", function = "boom" }
returns = "string"
"#,
    );
    let (ok, text) = run_text(dir.path(), &["check"]);
    assert!(!ok, "check passed: {text}");
    assert!(text.contains("derived.boom"), "{text}");
    assert!(text.contains("boom in lib.typ"), "{text}");

    let table = stdout(dir.path(), &["eval", "subject"]);
    assert!(table.contains("derived.boom: boom in lib.typ"), "{table}");
    let evaluated = json(&stdout(dir.path(), &["eval", "subject", "--json"]));
    assert!(
        evaluated["collector_problems"][0]
            .as_str()
            .unwrap()
            .contains("derived.boom"),
        "{evaluated}"
    );
}

#[test]
fn a_missing_typst_function_fails_config_load() {
    let dir = project(
        r#"[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "missing"
query = { typst = "lib.typ", function = "nope" }
returns = "string"
"#,
    );
    let out = mt(dir.path()).args(["ls"]).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("does not export `nope`"), "{err}");
}
