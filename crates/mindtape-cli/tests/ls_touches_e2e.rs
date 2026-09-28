//! End-to-end tests for `touches("<path>")` through `mt ls`
//! (`truth/tasks/add-a-touches-filter-over-the-files-a-tasks-commits-changed.typ`):
//! the tasks named by the trailers of the commits that changed a path, and
//! the retired stems met, in the `--json` envelope.
#![allow(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use std::path::Path;

mod support;

use support::{git, git_commit_all, mt, project_with_tasks};

/// Write `path` under `dir` and commit it with `message`.
fn commit_file(dir: &Path, path: &str, message: &str) {
    let file = dir.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, format!("{message}\n")).unwrap();
    git(dir, &["add", path]);
    git(
        dir,
        &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
    );
}

/// `mt ls <expr> --json`, parsed.
fn ls_json(dir: &Path, expr: &str) -> serde_json::Value {
    let out = mt(dir).args(["ls", expr, "--json"]).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn ls_lists_the_tasks_a_paths_commits_name_and_the_retired_stems_met() {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser"), ("lexer.typ", "Lex")]);
    let dir = dir.path();
    git_commit_all(dir);
    commit_file(dir, "src/parse.rs", "feat: parse\n\nRefs: tasks/parser.typ");
    commit_file(dir, "src/parse.rs", "fix: parse\n\nRefs: dropped");

    let value = ls_json(dir, r#"touches("src/parse.rs")"#);
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|task| task["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["parser"]);
    assert_eq!(value["retired"], serde_json::json!(["dropped"]));

    let plain = ls_json(dir, "");
    assert_eq!(
        plain["retired"],
        serde_json::json!([]),
        "always present, empty when no touches() met a retired stem"
    );
}

#[test]
fn outside_git_touches_fails_and_every_other_listing_works() {
    let dir = project_with_tasks(&[("parser.typ", "Write the parser")]);
    let dir = dir.path();
    let out = mt(dir)
        .args(["ls", r#"touches("src/parse.rs")"#])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("touches() cannot read git"), "{stderr}");
    assert!(mt(dir).args(["ls", ""]).output().unwrap().status.success());
}
