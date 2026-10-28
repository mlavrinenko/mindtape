//! End-to-end tests for `mt add --format`, reusing `mt ls`'s `text`/`path`/
//! `path0`/`json` vocabulary. Each format is checked against every dedup
//! outcome (`wrote`, `skipped`, `suffixed`) so a script can tell them apart
//! without parsing prose — the pairing `add-dedup-modes-to-mt-add.typ` and
//! `give-mt-add-machine-readable-output.typ` both call out.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::write_file;

/// `mt add`'s reported path is the exact `dest.join(...)` it wrote to — under
/// the default `--into .`, that is `<project root>/tasks/<rel>` in full,
/// mirroring today's `wrote <path>` prose (never shortened to a cwd-relative
/// form the way `mt ls --format path` shows a task's `file`).
fn expected_path(dir: &std::path::Path, rel: &str) -> String {
    mindtape::slash_path(&dir.join("tasks").join(rel))
}

/// A scaffolded project with a pre-existing `tasks/dup.typ`, the shared
/// fixture every skipped/suffixed outcome test collides against.
fn scaffold_with_existing_dup() -> tempfile::TempDir {
    let dir = support::scaffold();
    write_file(dir.path(), "tasks/dup.typ", "original");
    dir
}

/// `mt <args>` in `dir`, which must succeed, with its stdout parsed as JSON.
fn add_json(dir: &std::path::Path, args: &[&str]) -> serde_json::Value {
    let output = support::mt(dir).args(args).assert().success();
    serde_json::from_slice(&output.get_output().stdout).unwrap()
}

#[test]
fn text_format_is_the_default_wrote_prose() {
    let dir = support::scaffold();

    let output = support::mt(dir.path())
        .args(["add", "--name", "wired", "--set", "title=T"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.trim().starts_with("wrote "), "{stdout}");
    assert!(stdout.contains("wired.typ"), "{stdout}");
}

#[test]
fn path_format_prints_only_the_written_path() {
    let dir = support::scaffold();

    let output = support::mt(dir.path())
        .args([
            "add", "--name", "wired", "--set", "title=T", "--format", "path",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    assert_eq!(
        stdout.trim_end_matches('\n'),
        expected_path(dir.path(), "wired.typ")
    );
}

#[test]
fn path0_format_is_nul_terminated() {
    let dir = support::scaffold();

    let output = support::mt(dir.path())
        .args([
            "add", "--name", "wired", "--set", "title=T", "--format", "path0",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let expected = format!("{}\0", expected_path(dir.path(), "wired.typ"));
    assert_eq!(output, expected.into_bytes());
}

#[test]
fn json_format_reports_the_wrote_outcome() {
    let dir = support::scaffold();

    let value = add_json(
        dir.path(),
        &[
            "add", "--name", "wired", "--set", "title=T", "--format", "json",
        ],
    );
    assert_eq!(support::at(&value, "/outcome").as_str(), Some("wrote"));
    assert_eq!(
        support::at(&value, "/path").as_str(),
        Some(expected_path(dir.path(), "wired.typ").as_str())
    );
}

#[test]
fn json_format_reports_the_skipped_outcome() {
    let dir = scaffold_with_existing_dup();

    let value = add_json(
        dir.path(),
        &[
            "add", "--name", "dup", "--set", "title=T", "--dedup", "skip", "--format", "json",
        ],
    );
    assert_eq!(support::at(&value, "/outcome").as_str(), Some("skipped"));
    assert_eq!(
        support::at(&value, "/path").as_str(),
        Some(expected_path(dir.path(), "dup.typ").as_str())
    );
}

#[test]
fn json_format_reports_the_suffixed_outcome_with_the_minted_path() {
    let dir = scaffold_with_existing_dup();

    let value = add_json(
        dir.path(),
        &[
            "add",
            "--name",
            "dup",
            "--set",
            "title=T",
            "--dedup",
            "add-suffix",
            "--format",
            "json",
        ],
    );
    assert_eq!(support::at(&value, "/outcome").as_str(), Some("suffixed"));
    assert_eq!(
        support::at(&value, "/path").as_str(),
        Some(expected_path(dir.path(), "dup-2.typ").as_str())
    );
}

#[test]
fn text_format_names_the_skipped_and_suffixed_outcomes_too() {
    let dir = scaffold_with_existing_dup();

    let skipped = support::mt(dir.path())
        .args([
            "add", "--name", "dup", "--set", "title=T", "--dedup", "skip",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let skipped = String::from_utf8_lossy(&skipped);
    assert!(skipped.contains("skipped"), "{skipped}");
    assert!(skipped.contains("dup.typ"), "{skipped}");

    let suffixed = support::mt(dir.path())
        .args([
            "add",
            "--name",
            "dup",
            "--set",
            "title=T",
            "--dedup",
            "add-suffix",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let suffixed = String::from_utf8_lossy(&suffixed);
    assert!(suffixed.contains("suffixed"), "{suffixed}");
    assert!(suffixed.contains("dup-2.typ"), "{suffixed}");
}

/// `--print` stays orthogonal to `--format`: it previews the task source and
/// returns before any dedup decision or write, regardless of `--format`.
#[test]
fn print_ignores_format_and_previews_source_instead() {
    let dir = support::scaffold();

    let output = support::mt(dir.path())
        .args([
            "add", "--name", "preview", "--set", "title=T", "--print", "--format", "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    assert!(stdout.contains("title: \"T\""), "{stdout}");
    assert!(!dir.path().join("tasks/preview.typ").exists());
}

/// The slug hash the text report ends with, so the task can be named at once.
fn reported_hash(stdout: &str) -> String {
    let inner = stdout.trim().rsplit_once(" (").map(|(_, tail)| tail);
    inner
        .and_then(|tail| tail.strip_suffix(')'))
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn text_report_names_the_slug_hash_that_mt_eval_resolves() {
    let dir = support::scaffold();

    let stdout = support::stdout(
        dir.path(),
        &["add", "--name", "wired", "--set", "title=Wired up"],
    );
    let hash = reported_hash(&stdout);
    assert!(!hash.is_empty(), "no hash in {stdout}");
    assert!(
        stdout.starts_with(&shown("wrote tasks/wired.typ (")),
        "path is cwd-relative: {stdout}"
    );
    let evaluated = support::stdout(dir.path(), &["eval", &hash]);
    assert!(evaluated.contains("Wired up"), "{hash}: {evaluated}");
}

#[test]
fn json_report_carries_the_slug_hash() {
    let dir = support::scaffold();

    let stdout = support::stdout(
        dir.path(),
        &[
            "add", "--name", "wired", "--set", "title=T", "--format", "json",
        ],
    );
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let hash = value.get("slug").and_then(|slug| slug.as_str()).unwrap();
    let evaluated = support::stdout(dir.path(), &["eval", hash]);
    assert!(
        evaluated.contains("wired") || evaluated.contains('T'),
        "{evaluated}"
    );
}

/// An expected message, each task path in it in the separator `mt` shows.
fn shown(text: &str) -> String {
    support::native_in(text, &["tasks/wired.typ"])
}
