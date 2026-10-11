//! Regression test for the read/write separation (CQRS) principle: read
//! commands must never mutate the working tree.
//!
//! See `truth/tasks/command-semantics-cqrs.typ`,
//! `truth/tasks/gate-the-cqrs-split-against-the-command-tree.typ`, and the
//! "Command Semantics (read/write separation)" section of `CONTRIBUTING.md`.
//! Each test builds a link-free fixture task tree, hashes it, runs a single
//! read command, hashes it again, and asserts the two digests match —
//! byte-identical, not just "still parses".
//!
//! [`every_classified_read_command_leaves_tree_untouched`] generates one such
//! check per command the classification in
//! `crates/mindtape-cli/src/cli/command_class.rs` declares `Read`, read back via
//! `mt _cli --json` (`cli/tree_json.rs` rides the `class` field along on the same
//! tree `www/generated/cli.json` ships) — a bare invocation of every read
//! command, so a newly classified one is covered with no test hand-written
//! here. The hand-written tests below it stay: several exercise a read
//! command with flag combinations (`ls --format json`, `config show
//! --origin`) the generated loop's bare invocation does not reach, and a
//! generated test that dropped them would assert less than what is here
//! today. This crate has no `[lib]` target, so an integration test cannot
//! `use crate::cli::command_class` directly — reading the classification back out
//! of the CLI's own JSON dump is how a `tests/` binary sees it at all.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::support;

/// Scaffold a temp project's `.mindtape/config.toml` + the in-tree lib
/// override — the shared prologue every fixture below starts from.
fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    support::install_lib(dir);
}

/// File two link-free tasks into a scaffolded project via `mt add`.
///
/// No task holds a `depends-on`/`relates-to`/`supersedes`/`duplicates`/`links`
/// field: a parallel task is reworking the link DSL on the same base commit,
/// so these fixtures stay decoupled from it.
fn build_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    for (name, title) in [("alpha", "Alpha task"), ("beta", "Beta task")] {
        support::mt(dir.path())
            .args(["add", "--name", name, "--set", &format!("title={title}")])
            .assert()
            .success();
    }

    dir
}

/// A fixture holding one recurring task (`recur: every(weeks: 1)`), proving a
/// read command spawns/advances nothing even when a recurrence is present —
/// the CQRS invariant `truth/tasks/recurring-tasks.typ` depends on: `mt recur`
/// is the only recurrence writer, so `mt ls`/`mt check` must leave a
/// recurring task's tree exactly as untouched as any other.
fn build_recurring_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::write(
        dir.path().join("tasks/water-plants.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  \
         title: \"Water plants\",\n  \
         status: wip(2026, 6, 1),\n  \
         recur: every(weeks: 1),\n\
         )\n",
    )
    .unwrap();
    dir
}

#[test]
fn ls_over_a_recurring_task_spawns_nothing() {
    let dir = build_recurring_fixture();
    assert_read_untouched(dir.path(), &["ls"]);
}

#[test]
fn check_over_a_recurring_task_spawns_nothing() {
    let dir = build_recurring_fixture();
    assert_read_untouched(dir.path(), &["check"]);
}

/// Hash every file under `dir` by relative path and byte content into a
/// single order-independent digest, so any mutation — content change, added
/// or removed file — changes the result.
fn hash_tree(dir: &Path) -> u64 {
    let mut files = Vec::new();
    collect_files(dir, dir, &mut files);
    files.sort();

    let mut hasher = DefaultHasher::new();
    for (rel, bytes) in &files {
        rel.hash(&mut hasher);
        bytes.hash(&mut hasher);
    }
    hasher.finish()
}

/// Recursively collect `(path relative to root, file bytes)` pairs under `dir`.
///
/// The git-ignored `.mindtape/cache/` scratch is skipped: it is derived, not
/// authored, state — a read command populates it by design (the persistent
/// content-hash harvest cache), the way a build populates `target/`. The CQRS
/// invariant this test guards is that reads leave the *authored* tree
/// byte-identical, not that they touch no disk at all.
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            if path.ends_with(".mindtape/cache") {
                continue;
            }
            collect_files(root, &path, out);
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            out.push((rel, std::fs::read(&path).unwrap()));
        }
    }
}

/// Run `mt <args>` in `dir` and assert the tree hash is byte-identical before
/// and after — the read command must succeed, but leave nothing changed.
fn assert_read_untouched(dir: &Path, args: &[&str]) {
    let before = hash_tree(dir);

    support::mt(dir).args(args).assert().success();

    let after = hash_tree(dir);
    assert_eq!(
        before,
        after,
        "`mt {}` mutated the fixture tree",
        args.join(" ")
    );
}

#[test]
fn ls_default_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["ls"]);
}

#[test]
fn ls_json_format_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["ls", "--format", "json"]);
}

#[test]
fn check_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["check"]);
}

#[test]
fn config_show_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["config", "show"]);
}

#[test]
fn config_show_json_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["config", "show", "--json"]);
}

#[test]
fn config_show_origin_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["config", "show", "--origin"]);
}

#[test]
fn eval_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["eval", "tasks/alpha.typ"]);
}

/// `find` is sugar over `ls` with a `contains()` filter, and `graph` walks the
/// same loaded project to emit DOT. Both are reads, and neither was covered
/// here — the gap surfaced when CONTRIBUTING.md's read list turned out to name
/// two commands that do not exist while omitting these.
#[test]
fn find_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["find", "alpha"]);
}

#[test]
fn graph_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["graph"]);
}

#[test]
fn views_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["views"]);
}

#[test]
fn worktrees_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["worktrees"]);
}

/// Every `Read`-classified leaf's flattened path (`"ls"`, `"config schema"`,
/// `"preset ls"`, …), read from a fresh `mt _cli --json` rather than
/// hand-copied — the exhaustiveness this file leans on lives in `cli/tree_json.rs`
/// (`cli::command_class::CLASSIFICATION` must equal the visible command tree), so
/// this only has to walk the tree the same way that gate does.
fn read_command_paths() -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    let output = support::mt(dir.path())
        .args(["_cli", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "`mt _cli --json` failed");
    let tree: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let commands = tree
        .get("commands")
        .and_then(serde_json::Value::as_array)
        .unwrap();
    let mut out = Vec::new();
    collect_read_leaves(commands, "", &mut out);
    out
}

/// Recurse into a node with subcommands (`config`, `preset`); collect a leaf
/// whose `class.kind` is `"read"`, flattening its path the same way
/// `cli/tree_json.rs`'s `flattened_name` does.
fn collect_read_leaves(commands: &[serde_json::Value], prefix: &str, out: &mut Vec<String>) {
    for cmd in commands {
        let name = cmd.get("name").and_then(serde_json::Value::as_str).unwrap();
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix} {name}")
        };
        let subs = cmd
            .get("subcommands")
            .and_then(serde_json::Value::as_array)
            .unwrap();
        if subs.is_empty() {
            let kind = cmd
                .pointer("/class/kind")
                .and_then(serde_json::Value::as_str);
            if kind == Some("read") {
                out.push(path);
            }
        } else {
            collect_read_leaves(subs, &path, out);
        }
    }
}

/// Extra args a read command needs beyond its bare name split on spaces, for
/// a required positional the classification has no way to supply generically.
fn extra_args(path: &str) -> &'static [&'static str] {
    match path {
        // `mt eval <file>`: the file positional is required.
        "eval" => &["tasks/alpha.typ"],
        // `mt find <term>...`: at least one search term is required.
        "find" => &["alpha"],
        _ => &[],
    }
}

#[test]
fn every_classified_read_command_leaves_tree_untouched() {
    for path in read_command_paths() {
        let dir = build_fixture();
        let mut args: Vec<&str> = path.split(' ').collect();
        args.extend_from_slice(extra_args(&path));
        assert_read_untouched(dir.path(), &args);
    }
}
