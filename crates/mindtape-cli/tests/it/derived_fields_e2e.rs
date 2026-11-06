//! End-to-end tests for `[[tasks.collectors]]` and the derived fields they
//! produce: `mt ls --json`, `mt eval --json`, `mt explain rules` and `mt import`.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::disallowed_methods
)]

use mindtape::test_support::write_file;

use crate::support;

use support::{mt, scaffold_project, stdout};

const CONFIG: &str = r#"[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "changes"
query = { section = "Changes", pattern = "(?m)^\\s*-\\s+`([^`]+)`" }
returns = ["path"]

[[tasks.rules]]
name = "has-changes"
assert = 'len(derived.changes) > 0'
message = "name the files in Changes"
severity = "error"
"#;

const TASK: &str = "#import \"@local/mindtape:0.4.0\": *\n\n\
#show: task.with(\n  title: \"Collect me\",\n  status: wip(2026, 11, 4),\n)\n\n\
== Changes\n\n- `src/a.rs`\n- `src/b.rs`\n";

fn project() -> tempfile::TempDir {
    let dir = scaffold_project(CONFIG);
    write_file(dir.path(), "tasks/subject.typ", TASK);
    dir
}

fn json(text: &str) -> serde_json::Value {
    serde_json::from_str(text).unwrap()
}

#[test]
fn a_listing_carries_the_derived_field() {
    let dir = project();
    let listing = json(&stdout(
        dir.path(),
        &["ls", "has(derived.changes)", "--json"],
    ));
    let changes = &listing["tasks"][0]["derived"]["changes"];
    assert_eq!(changes[0], "src/a.rs", "{listing}");
    assert_eq!(changes[1], "src/b.rs", "{listing}");
}

#[test]
fn a_filter_may_count_a_derived_list() {
    let dir = project();
    let listing = json(&stdout(
        dir.path(),
        &["ls", "len(derived.changes) == 2", "--json"],
    ));
    assert_eq!(listing["tasks"][0]["id"], "subject", "{listing}");
    let none = json(&stdout(
        dir.path(),
        &["ls", "len(derived.changes) == 3", "--json"],
    ));
    assert!(none["tasks"].as_array().unwrap().is_empty(), "{none}");
}

#[test]
fn rules_lists_the_derived_field_and_its_reader() {
    let dir = project();
    let text = stdout(dir.path(), &["explain", "rules"]);
    assert!(text.contains("derived fields"), "{text}");
    assert!(text.contains("derived.changes"), "{text}");
    assert!(text.contains("regex"), "{text}");
    assert!(text.contains("tasks.rules.has-changes"), "{text}");
}

#[test]
fn rules_for_commit_hides_the_derived_block() {
    let dir = project();
    let text = stdout(dir.path(), &["explain", "rules", "--for", "commit"]);
    assert!(!text.contains("derived fields"), "{text}");
    assert_eq!(text, "no rules apply\n");
}

#[test]
fn eval_fills_the_body_and_runs_collectors() {
    let dir = project();
    let task = json(&stdout(dir.path(), &["eval", "subject", "--json"]));
    assert!(
        task["body"].as_str().unwrap().contains("== Changes"),
        "{task}"
    );
    assert_eq!(task["derived"]["changes"][0], "src/a.rs", "{task}");
}

#[test]
fn import_runs_collectors_but_writes_no_derived_field() {
    let dir = project();
    let record = serde_json::json!({
        "title": "Imported",
        "derived": {"changes": ["src/ignored.rs"]},
        "body": "== Changes\n\n- `src/a.rs`\n",
    });
    let input = serde_json::json!([record]);
    mt(dir.path())
        .args(["import"])
        .write_stdin(input.to_string())
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/imported.typ")).unwrap();
    assert!(!written.contains("derived"), "{written}");
}

#[test]
fn eval_a_file_outside_the_roots_harvests_it_and_runs_collectors() {
    let dir = project();
    write_file(
        dir.path(),
        "loose/loose.typ",
        TASK.replace("Collect me", "Loose"),
    );
    let task = json(&stdout(dir.path(), &["eval", "loose/loose.typ", "--json"]));
    assert_eq!(task["derived"]["changes"][0], "src/a.rs", "{task}");
}

#[test]
fn eval_a_file_under_no_project_reads_the_global_config() {
    let dir = mindtape::test_support::tempdir();
    support::install_lib(dir.path());
    write_file(dir.path(), "solo.typ", TASK);
    let config_home = dir.path().join("no-global-config");
    write_file(
        &config_home,
        "mindtape/defaults.toml",
        "[[tasks.collectors]]\nname = \"changes\"\n\
         query = { section = \"Changes\", pattern = \"(?m)^\\\\s*-\\\\s+`([^`]+)`\" }\n\
         returns = [\"path\"]\n",
    );
    let task = json(&stdout(dir.path(), &["eval", "solo.typ", "--json"]));
    assert!(
        task["body"].as_str().unwrap().contains("== Changes"),
        "{task}"
    );
    assert_eq!(task["id"], "solo", "{task}");
    assert_eq!(task["derived"]["changes"][0], "src/a.rs", "{task}");
}

#[test]
fn rules_lists_an_ast_grep_collector_and_its_reader() {
    let dir = scaffold_project(
        r#"[tasks]
roots = ["tasks"]

[[tasks.collectors]]
name = "callers"
query = { ast_grep = { language = "rust", pattern = "parse_task($$$ARGS)" } }
returns = ["path"]

[[tasks.rules]]
name = "small-fanout"
assert = 'len(derived.callers) <= 2'
message = "keep the fan-out small"
severity = "error"
"#,
    );
    let text = stdout(dir.path(), &["explain", "rules"]);
    for part in ["ast-grep", "derived.callers", "read by rules.small-fanout"] {
        assert!(text.contains(part), "{part}: {text}");
    }
}
