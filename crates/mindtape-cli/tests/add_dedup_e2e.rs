//! End-to-end tests for `mt add --dedup` and its `[tasks.naming] dedup`
//! config default: `fail` (today's hard error, still the default), `skip`
//! (exit 0, write nothing, report the existing path), and `add-suffix`
//! (write under a disambiguated stem). The `add-suffix` exhaustion path is
//! covered as a fast fs-free unit test (`add_dedup_tests.rs`) rather than
//! here: reaching `naming`'s 10,000-attempt ceiling with real files would
//! make this suite slow for no extra coverage — `add.rs::run()` calls the
//! exact same `add_dedup::resolve` the unit test exercises.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

/// A pre-existing `dup.typ` with recognisable content, so a test can prove
/// `--dedup` left it untouched (`skip`) or wrote alongside it (`add-suffix`).
fn scaffold_with_existing_dup(dir: &Path) {
    support::scaffold(dir);
    std::fs::write(dir.join("tasks/dup.typ"), "original content").unwrap();
}

/// A project whose `[tasks.naming] dedup` default is `skip`, plus a
/// pre-existing `dup.typ` — the shared fixture for the config-default tests
/// below, which differ only in whether `--dedup` overrides it.
fn scaffold_with_dedup_skip_default(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n[tasks.naming]\ndedup = \"skip\"\n",
    )
    .unwrap();
    support::install_lib(dir);
    std::fs::write(dir.join("tasks/dup.typ"), "original content").unwrap();
}

#[test]
fn dedup_fail_is_the_default_and_errors_on_collision() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_existing_dup(dir.path());

    support::mt(dir.path())
        .args(["add", "--name", "dup", "--set", "title=Dup"])
        .assert()
        .failure();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/dup.typ")).unwrap(),
        "original content",
        "a failed add must never touch the existing file"
    );
}

#[test]
fn dedup_fail_flag_is_explicit_too() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_existing_dup(dir.path());

    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "dup",
            "--set",
            "title=Dup",
            "--dedup",
            "fail",
        ])
        .assert()
        .failure();
}

#[test]
fn dedup_skip_exits_zero_writes_nothing_and_reports_existing_path() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_existing_dup(dir.path());

    let output = support::mt(dir.path())
        .args([
            "add",
            "--name",
            "dup",
            "--set",
            "title=Dup",
            "--dedup",
            "skip",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    assert!(
        stdout.contains("tasks/dup.typ") || stdout.contains("dup.typ"),
        "{stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/dup.typ")).unwrap(),
        "original content",
        "skip must never overwrite the existing file"
    );
}

#[test]
fn dedup_add_suffix_writes_under_a_disambiguated_stem() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_existing_dup(dir.path());

    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "dup",
            "--set",
            "title=Dup Two",
            "--dedup",
            "add-suffix",
        ])
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/dup.typ")).unwrap(),
        "original content",
        "add-suffix must never overwrite the original"
    );
    let suffixed = std::fs::read_to_string(dir.path().join("tasks/dup-2.typ")).unwrap();
    assert!(suffixed.contains("title: \"Dup Two\""), "{suffixed}");
}

/// `add-suffix` is reachable with no `[tasks.naming] stem` template configured
/// at all — today it only ever ran when a stem template was set; the bare
/// slug case is exactly what this task makes newly reachable.
#[test]
fn dedup_add_suffix_works_without_a_naming_template() {
    let dir = tempfile::tempdir().unwrap();
    support::scaffold(dir.path());
    std::fs::write(dir.path().join("tasks/ship-it.typ"), "original").unwrap();

    support::mt(dir.path())
        .args(["add", "Ship It", "--dedup", "add-suffix"])
        .assert()
        .success();
    assert!(dir.path().join("tasks/ship-it-2.typ").exists());
}

#[test]
fn config_dedup_default_is_honoured_without_the_flag() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_dedup_skip_default(dir.path());

    support::mt(dir.path())
        .args(["add", "--name", "dup", "--set", "title=Dup"])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/dup.typ")).unwrap(),
        "original content"
    );
}

#[test]
fn dedup_flag_overrides_the_config_default() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_dedup_skip_default(dir.path());

    // The project default is `skip`; `--dedup fail` on the CLI wins outright.
    support::mt(dir.path())
        .args([
            "add",
            "--name",
            "dup",
            "--set",
            "title=Dup",
            "--dedup",
            "fail",
        ])
        .assert()
        .failure();
}
