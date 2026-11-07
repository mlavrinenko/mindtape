//! `mt schema` holds the recorded output schemas to the binary, and the live
//! `mt ls --json` output validates against its recorded schema.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};

use crate::support;

/// The checkout root, derived from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn recorded_schemas_match_the_binary() {
    let root = repo_root();
    let out = support::mt(&root)
        .args(["schema", "--check"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "mt schema --check failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn schema_lists_every_registered_output() {
    let root = repo_root();
    let text = support::stdout(&root, &["schema"]);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert!(lines[0].starts_with("ls  1  "), "{text}");
    assert!(lines[1].starts_with("task  1  "), "{text}");
    assert!(lines[0].ends_with("/ls/1.schema.json"), "{text}");
    assert!(lines[1].ends_with("/task/1.schema.json"), "{text}");
}

/// `mt schema` in `dir` with `MINDTAPE_SCHEMA_BASE` set to `base`.
fn schema_urls(dir: &Path, base: &str) -> String {
    let out = support::mt(dir)
        .env("MINDTAPE_SCHEMA_BASE", base)
        .arg("schema")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn schema_urls_follow_the_instance_base() {
    let dir = tempfile::tempdir().unwrap();
    for (base, expected) in [
        (
            "file:///tmp/schemas/",
            "file:///tmp/schemas/ls/1.schema.json",
        ),
        ("", "schema/output/ls/1.schema.json"),
    ] {
        let text = schema_urls(dir.path(), base);
        assert!(text.contains(expected), "{base:?}: {text}");
    }
}

#[test]
fn schema_writes_and_checks_a_project_independent_copy() {
    let dir = tempfile::tempdir().unwrap();
    let text = support::stdout(dir.path(), &["schema", "--write"]);
    assert!(text.contains("schema/output/ls/1.schema.json"), "{text}");
    assert!(text.contains("schema/output/task/1.schema.json"), "{text}");
    support::stdout(dir.path(), &["schema", "--check"]);
}

#[test]
fn schema_prints_one_named_schema() {
    let root = repo_root();
    let text = support::stdout(&root, &["schema", "ls"]);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        value["$id"],
        "https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/output/ls/1.schema.json"
    );
}

#[test]
fn schema_refuses_an_unknown_name() {
    let root = repo_root();
    let out = support::mt(&root)
        .args(["schema", "nope"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("unknown schema"), "{stderr}");
}

#[test]
fn live_ls_output_validates_against_the_recorded_schema() {
    let root = repo_root();
    let schema_text = std::fs::read_to_string(root.join("schema/output/ls/1.schema.json")).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_text).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();

    let dir = support::project_with_tasks(&[("a.typ", "Alpha")]);
    let text = support::stdout(dir.path(), &["ls", "", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();

    let schema_url = value["$schema"].as_str().unwrap();
    assert!(schema_url.ends_with("/ls/1.schema.json"), "{schema_url}");
    if let Some(error) = validator.iter_errors(&value).next() {
        panic!("{error}");
    }
}
