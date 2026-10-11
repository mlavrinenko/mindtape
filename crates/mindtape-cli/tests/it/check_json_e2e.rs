//! `mt check --json` prints every problem as one object on stdout and leaves
//! the exit code to the problems, as the text report does.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use std::path::Path;

use crate::support;

use support::{mt, scaffold_project};

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.rules]]\nname = \"imperative-title\"\n\
    assert = 'title != \"Loader refactoring\"'\n\
    message = \"start the title with an imperative verb\"\nseverity = \"error\"\n\n\
    [[tasks.rules]]\nname = \"no-wip-note\"\nassert = 'title != \"Loader polishing\"'\n\
    message = \"pick a better title\"\nseverity = \"warn\"\n";

fn task(title: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"{title}\",\n  status: wip(2026, 6, 1),\n)\n"
    )
}

fn run(dir: &Path, args: &[&str]) -> (Option<i32>, serde_json::Value, String) {
    let out = mt(dir).args(args).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|err| panic!("{err}: {stdout:?}"));
    (
        out.status.code(),
        json,
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn an_error_and_a_warning_are_two_objects_and_the_exit_code_is_one() {
    let dir = scaffold_project(CONFIG);
    std::fs::write(
        dir.path().join("tasks/subject.typ"),
        task("Loader refactoring"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("tasks/polish.typ"),
        task("Fix Loader polishing"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("tasks/warned.typ"),
        task("Loader polishing"),
    )
    .unwrap();

    let (code, json, stderr) = run(dir.path(), &["check", "--json"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(json["schema_version"], mindtape::SCHEMA_VERSION);
    let problems = json["problems"].as_array().unwrap();
    let error = problems
        .iter()
        .find(|p| p["severity"] == "error" && p["file"] == "tasks/subject.typ")
        .unwrap_or_else(|| panic!("{json}"));
    assert_eq!(error["line"], 3);
    assert_eq!(error["rule"], "rules.imperative-title");
    assert_eq!(error["message"], "start the title with an imperative verb");
    assert_eq!(error["hints"], serde_json::json!([]));
    let warning = problems
        .iter()
        .find(|p| p["severity"] == "warning" && p["file"] == "tasks/warned.typ")
        .unwrap_or_else(|| panic!("{json}"));
    assert_eq!(warning["rule"], "rules.no-wip-note");
    assert!(
        !stderr.contains("rules.imperative-title"),
        "problems go to stdout: {stderr}"
    );
}

#[test]
fn a_clean_project_prints_an_empty_list_and_exits_zero() {
    let dir = scaffold_project(CONFIG);
    std::fs::write(dir.path().join("tasks/fine.typ"), task("Fix Loader")).unwrap();

    let (code, json, stderr) = run(dir.path(), &["check", "--json"]);

    assert_eq!(code, Some(0), "{stderr} {json}");
    assert_eq!(json["problems"], serde_json::json!([]));
}

#[test]
fn a_warning_alone_exits_zero() {
    let dir = scaffold_project(CONFIG);
    std::fs::write(
        dir.path().join("tasks/warned.typ"),
        task("Loader polishing"),
    )
    .unwrap();

    let (code, json, _) = run(dir.path(), &["check", "--json"]);

    assert_eq!(code, Some(0));
    assert_eq!(json["problems"][0]["severity"], "warning");
}
