//! `mt schema` holds the recorded output schemas to the binary, and live
//! `--json` output validates against its recorded schema.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::disallowed_methods
)]

use std::path::{Path, PathBuf};

use crate::support;

/// The checkout root, derived from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every golden this suite and `mt schema --check` compare byte for byte is
/// checked out with LF. Git's `core.autocrlf` would otherwise turn it into
/// CRLF on Windows, and the comparison would fail only there. New schema files
/// and prelude snapshots are covered by the walk, so a new golden cannot ship
/// without the attribute.
#[test]
fn byte_compared_goldens_are_pinned_to_lf() {
    let root = repo_root();
    let listed = std::process::Command::new("git")
        .current_dir(&root)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "schema",
            "crates/mindtape/src/loader/preludes",
        ])
        .output()
        .unwrap();
    assert!(listed.status.success(), "{listed:?}");
    let files = String::from_utf8(listed.stdout).unwrap();
    assert!(!files.is_empty(), "the golden walk found no files");
    for file in files.split('\0').filter(|file| !file.is_empty()) {
        let attr = std::process::Command::new("git")
            .current_dir(&root)
            .args(["check-attr", "eol", "--", file])
            .output()
            .unwrap();
        assert!(attr.status.success(), "{attr:?}");
        let attr = String::from_utf8(attr.stdout).unwrap();
        assert!(
            attr.trim_end().ends_with(": eol: lf"),
            "{file} is not pinned to LF by .gitattributes: {attr}"
        );
    }
}

/// Every registered output, in `mt schema` order.
const NAMES: [&str; 24] = [
    "ls",
    "task",
    "add",
    "check",
    "set-check",
    "graph",
    "preset-ls",
    "explain-add",
    "explain",
    "rules",
    "tags",
    "eval",
    "board",
    "at",
    "at-project",
    "at-set",
    "set-ls",
    "packs",
    "describe",
    "views",
    "view",
    "dicts",
    "dict",
    "config-show",
];

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
    assert_eq!(lines.len(), NAMES.len(), "{text}");
    for (line, name) in lines.iter().zip(NAMES) {
        assert!(line.starts_with(name), "{text}");
        assert!(line.contains(&format!("/{name}/")), "{text}");
    }
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
    for name in NAMES {
        let path = support::native(&format!("schema/output/{name}/"));
        assert!(text.contains(&path), "{text}");
    }
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

/// Validate `value` against the recorded schema `name` and check that its
/// `$schema` names that same file.
fn validate(name: &str, value: &serde_json::Value) {
    let root = repo_root();
    let schema_url = value["$schema"].as_str().unwrap();
    let tail = schema_url.split_once("/schema/output/").map_or_else(
        || panic!("unexpected $schema {schema_url}"),
        |(_, tail)| tail,
    );
    assert!(tail.starts_with(&format!("{name}/")), "{schema_url}");
    let schema_text = std::fs::read_to_string(root.join("schema/output").join(tail)).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_text).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    if let Some(error) = validator.iter_errors(value).next() {
        panic!("{name}: {error}");
    }
}

/// Validate each `(schema name, argv)` case's JSON output in `dir`.
fn validate_cases(dir: &Path, cases: &[(&str, &[&str])]) {
    for (name, args) in cases {
        validate(name, &support::json(dir, args));
    }
}

#[test]
fn live_object_outputs_validate_against_their_recorded_schemas() {
    let dir = support::project_with_tasks(&[("a.typ", "Alpha")]);
    validate_cases(
        dir.path(),
        &[
            ("ls", &["ls", "", "--json"][..]),
            ("check", &["check", "--json"][..]),
            ("preset-ls", &["preset", "ls", "--json"][..]),
            ("explain", &["explain", "--json"][..]),
            ("rules", &["explain", "rules", "--json"][..]),
            ("tags", &["explain", "tags", "--json"][..]),
            ("eval", &["eval", "a", "--json"][..]),
        ],
    );
}

#[test]
fn live_set_check_output_validates_against_the_recorded_schema() {
    let (_alpha, _beta, cfg) = crate::at_e2e::setup();
    let out = crate::at_e2e::run(cfg.path(), &["@both", "check", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    validate("set-check", &value);
}

#[test]
fn live_graph_output_validates_against_the_recorded_schema() {
    let dir = support::project_with_tasks(&[("a.typ", "Alpha")]);
    validate("graph", &support::json(dir.path(), &["graph", "--json"]));
}

#[test]
fn live_add_output_validates_against_the_recorded_schema() {
    let dir = support::scaffold();
    let text = support::add_wired_json(dir.path());
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    validate("add", &value);
}

#[test]
fn live_collection_outputs_validate_against_their_schemas() {
    let dir = support::project_with_tasks(&[("a.typ", "Alpha")]);
    let config = dir.path().join(".mindtape/config.toml");
    let mut text = std::fs::read_to_string(&config).unwrap();
    text.push_str(
        "\n[views.hot]\nfilter = 'priority > 2'\n\n\
         [dicts.labels]\nvalues = [\"bug\", \"feat\"]\n",
    );
    std::fs::write(&config, text).unwrap();

    validate_cases(
        dir.path(),
        &[
            ("board", &["board", "--json"][..]),
            ("at", &["at", "--json"][..]),
            ("describe", &["add", "--describe=json"][..]),
            ("views", &["explain", "views", "--json"][..]),
            ("view", &["explain", "views", "hot", "--json"][..]),
            ("dicts", &["explain", "dicts", "--json"][..]),
            ("dict", &["explain", "dicts", "labels", "--json"][..]),
            ("config-show", &["config", "show", "--json"][..]),
        ],
    );
}
