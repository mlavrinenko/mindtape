//! Regression test for the read/write separation (CQRS) principle: read
//! commands must never mutate the working tree.
//!
//! See `truth/tasks/2026-07/command-semantics-cqrs.typ`,
//! `truth/tasks/2026-08/gate-the-cqrs-split-against-the-command-tree.typ`, and the
//! "Command Semantics (read/write separation)" section of `CONTRIBUTING.md`.
//! Each test builds a link-free fixture task tree, hashes it, runs a single
//! read command, hashes it again, and asserts the two digests match. The match
//! is byte-identical, not just "still parses".
//!
//! [`every_classified_read_command_leaves_tree_untouched`] generates one such
//! check per command the classification in
//! `crates/mindtape-cli/src/cli/command_class.rs` declares `Read`, read back
//! through `mt _cli --json`. `cli/tree_json.rs` rides the `class` field along on
//! the same tree that `www/generated/cli.json` emits. Each check is a bare
//! invocation of one read command, so a newly classified command gets coverage
//! without a hand-written test here. The hand-written tests below it stay. Many
//! of them exercise a read command with flag combinations (`ls --format json`,
//! `config show --origin`) that the generated loop's bare invocation does not
//! reach. A generated test that dropped them would assert less than what is
//! here today. This crate has no `[lib]` target, so an integration test cannot
//! `use crate::cli::command_class` directly. Reading the classification back
//! out of the CLI's own JSON dump is how a `tests/` binary sees it at all.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::support;

/// File two link-free tasks into a scaffolded project via `mt add`.
///
/// The fixtures skip the
/// `depends-on`/`relates-to`/`supersedes`/`duplicates`/`links` field. A
/// parallel task is reworking the link DSL on the same base commit. These
/// fixtures stay decoupled from it.
fn build_fixture() -> tempfile::TempDir {
    let dir = support::scaffold();

    for (name, title) in [("alpha", "Alpha task"), ("beta", "Beta task")] {
        support::run_ok(
            dir.path(),
            &["add", "--name", name, "--set", &format!("title={title}")],
        );
    }

    dir
}

/// A fixture holding one recurring task (`recur: every(weeks: 1)`). It proves a
/// read command spawns or advances nothing even when a recurrence is present.
/// That is the CQRS invariant `truth/tasks/2026-07/recurring-tasks.typ` depends
/// on. `mt recur` is the only recurrence writer, so `mt ls`/`mt check` must
/// leave a recurring task's tree exactly as untouched as any other.
fn build_recurring_fixture() -> tempfile::TempDir {
    let dir = support::scaffold();
    write_file(
        dir.path(),
        "tasks/water-plants.typ",
        mindtape::typst_task!(
            "(\n  \
         title: \"Water plants\",\n  \
         status: wip(2026, 6, 1),\n  \
         recur: every(weeks: 1),\n\
         )\n"
        ),
    );
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

/// Hash every file under `dir` by relative path and byte content into a single
/// order-independent digest. Any mutation then changes the result: a content
/// change, an added file, or a removed file.
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
/// The walk skips the git-ignored `.mindtape/cache/` scratch, because the
/// harvest derives that state rather than authoring it. A read command
/// populates it by design
/// (the persistent content-hash harvest cache), the way a build populates
/// `target/`. The CQRS invariant this test guards is that reads leave the
/// *authored* tree byte-identical, not that they avoid touching the disk.
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
/// and after. The read command must succeed, and it must not change anything.
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
/// same loaded project to emit DOT. Both are reads, and neither had coverage
/// here. The gap surfaced when CONTRIBUTING.md's read list turned out to
/// reference two commands that do not exist while omitting these.
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
    assert_read_untouched(dir.path(), &["explain", "views"]);
}

#[test]
fn dicts_name_json_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(
        dir.path(),
        &["explain", "dicts", "mindtape-verbs", "--json"],
    );
}

#[test]
fn at_leaves_tree_untouched() {
    let dir = build_fixture();
    assert_read_untouched(dir.path(), &["at"]);
}

/// Every `Read`-classified leaf's flattened path (`"ls"`, `"config schema"`,
/// `"preset ls"`, …), read from a fresh `mt _cli --json` rather than
/// hand-copied. The exhaustiveness this file leans on lives in
/// `cli/tree_json.rs`, where `cli::command_class::CLASSIFICATION` must equal the
/// visible command tree. This function only has to walk the tree the same way
/// that gate does.
fn read_command_paths() -> Vec<String> {
    read_leaves().into_iter().map(|(path, _)| path).collect()
}

/// Every `Read`-classified leaf of a fresh `mt _cli --json`, as its flattened
/// path and its JSON node. A test can then read any mark the registry exports.
pub(crate) fn read_leaves() -> Vec<(String, serde_json::Value)> {
    let dir = tempdir();
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

/// Recurse into a node with subcommands (`config`, `preset`, `explain`), and
/// collect a node whose `class.kind` is `"read"`: a leaf, or a runnable parent
/// like `explain`. The function flattens its path the same way
/// `cli/tree_json.rs`'s `flattened_name` does.
fn collect_read_leaves(
    commands: &[serde_json::Value],
    prefix: &str,
    out: &mut Vec<(String, serde_json::Value)>,
) {
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
        let kind = cmd
            .pointer("/class/kind")
            .and_then(serde_json::Value::as_str);
        if kind == Some("read") {
            out.push((path.clone(), cmd.clone()));
        }
        if !subs.is_empty() {
            collect_read_leaves(subs, &path, out);
        }
    }
}

/// Extra args a read command needs beyond its bare name split on spaces, for
/// a required positional the classification has no way to supply generically.
pub(crate) fn extra_args(path: &str) -> &'static [&'static str] {
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
