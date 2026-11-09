//! End-to-end tests for the `add` destination guard: a refused destination
//! creates nothing, and only an outside-root destination warns.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;

/// A project whose only task root is `.`, in a Git work tree that ignores
/// `target/`: the destination the write guard must refuse.
fn ignored_target_project() -> tempfile::TempDir {
    let dir = support::scaffold_project("[tasks]\nroots = [\".\"]\n");
    write_file(dir.path(), ".gitignore", "target/\n");
    support::git(dir.path(), &["init", "-q"]);
    dir
}

/// The path `mt` shows for `rel` under `dir`, in the native separator.
fn shown(dir: &std::path::Path, rel: &str) -> String {
    mindtape::display_path(dir.join(rel))
}

#[test]
fn an_ignored_destination_is_refused_and_creates_nothing() {
    let dir = ignored_target_project();

    let output = support::mt(dir.path())
        .args([
            "add",
            "--to",
            "target",
            "--name",
            "hidden",
            "--set",
            "title=Hidden task",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        !dir.path().join("target").exists(),
        "a refused destination must not be created"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&shown(dir.path(), "target/hidden.typ")),
        "the refusal must name the final path: {stderr}"
    );
    assert!(stderr.contains("would be invisible"), "{stderr}");
    assert!(
        stderr.contains("--into"),
        "the fix must offer another destination: {stderr}"
    );
}

#[test]
fn a_reserved_destination_is_refused_and_creates_nothing() {
    let dir = support::scaffold_project("[tasks]\nroots = [\".\"]\n");

    let output = support::mt(dir.path())
        .args([
            "add",
            "--to",
            ".mindtape/templates",
            "--name",
            "hidden",
            "--set",
            "title=Hidden task",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        !dir.path().join(".mindtape/templates/hidden.typ").exists(),
        "a refused destination must not be created"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&shown(dir.path(), ".mindtape/templates/hidden.typ")),
        "the refusal must name the final path: {stderr}"
    );
    assert!(stderr.contains("reserved"), "{stderr}");
}

/// `link/..` is `target` on disk, which is ignored, while its lexical form is
/// the project root. Resolving the symlink before the policy asks makes the
/// escape visible and refused.
#[cfg(unix)]
#[test]
fn a_symlinked_dot_dot_escape_into_an_ignored_directory_is_refused() {
    let dir = ignored_target_project();
    write_file(dir.path(), "target/sub/keep", "");
    std::os::unix::fs::symlink(dir.path().join("target/sub"), dir.path().join("link")).unwrap();

    let output = support::mt(dir.path())
        .args([
            "add",
            "--to",
            "link/..",
            "--name",
            "hidden",
            "--set",
            "title=Hidden task",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    assert!(
        !dir.path().join("target/hidden.typ").exists(),
        "a refused destination must not be created"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&shown(dir.path(), "target/hidden.typ")),
        "the refusal must name the resolved path: {stderr}"
    );
    assert!(stderr.contains("would be invisible"), "{stderr}");
}

/// `--no-verify` opens content validation, and nothing else: the destination
/// guard is not content validation.
#[test]
fn no_verify_does_not_bypass_the_destination_guard() {
    let dir = ignored_target_project();

    support::mt(dir.path())
        .args([
            "add",
            "--no-verify",
            "--to",
            "target",
            "--name",
            "hidden",
            "--set",
            "title=Hidden task",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("would be invisible"));

    assert!(!dir.path().join("target/hidden.typ").exists());
}

/// An outside-root destination stays what it always was: a write with a
/// warning, not a refusal.
#[test]
fn an_outside_root_destination_still_only_warns() {
    let dir = support::scaffold();
    let outside = tempdir();

    let output = support::mt(dir.path())
        .args([
            "add",
            "--no-verify",
            "--into",
            outside.path().to_str().unwrap(),
            "--name",
            "outside-task",
            "--set",
            "title=Outside task",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    assert!(outside.path().join("outside-task.typ").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("outside the configured roots"), "{stderr}");
}
