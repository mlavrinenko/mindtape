//! End-to-end tests for `mt add --dedup` and its `[tasks.naming] dedup`
//! config default: `fail` (today's hard error, still the default), `skip`
//! (exit 0, write nothing, report the existing path), and `add-suffix`
//! (write under a disambiguated stem). The `add-suffix` exhaustion path is
//! covered as a fast fs-free unit test (`add_dedup_tests.rs`) rather than
//! here: reaching `naming`'s 10,000-attempt ceiling with real files would
//! make this suite slow for no extra coverage — `add.rs::run()` calls the
//! exact same `add_dedup::resolve` the unit test exercises.
//!
//! The section below the bare-stem tests repeats the same three modes with a
//! `[tasks.naming] stem` template configured (`title` minted, not `--name` —
//! that is what routes through `output_stem`'s template branch). This is the
//! regression matrix for
//! `make-dedup-modes-reachable-under-a-templated-stem`: before that fix,
//! `output_stem` disambiguated a templated stem itself, so every `--dedup`
//! mode silently behaved as `add-suffix` under a template. Each templated
//! test below is the direct counterpart of a bare-stem test above it, same
//! assertions, only the fixture's `[tasks.naming] stem` differs — the matrix
//! this bug needed and the existing suites (one per path) didn't have.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

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

/// The templated-stem counterpart of [`scaffold_with_existing_dup`]: a
/// `[tasks.naming] stem = "{slug}"` template (mints exactly the bare slug, so
/// `dup.typ` collides identically) plus a pre-existing `dup.typ`. Every test
/// built on this fixture files with a bare positional title (`Dup`), never
/// `--name`, so the mint goes through `output_stem`'s template branch rather
/// than the literal short-circuit.
fn scaffold_with_templated_stem_and_existing_dup(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n[tasks.naming]\nstem = \"{slug}\"\n",
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

/// Shared assertion for a `--dedup skip` run, bare or templated stem alike:
/// the reported output names the existing file, and that file is untouched.
fn assert_skip_reported_and_left_dup_untouched(dir: &Path, stdout: &[u8]) {
    let stdout = String::from_utf8_lossy(stdout);
    assert!(
        stdout.contains("tasks/dup.typ") || stdout.contains("dup.typ"),
        "{stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("tasks/dup.typ")).unwrap(),
        "original content",
        "skip must never overwrite the existing file"
    );
}

/// Shared assertion for an `add-suffix` run, bare or templated stem alike:
/// the original is untouched and the disambiguated `dup-2.typ` carries
/// `title`.
fn assert_original_untouched_and_suffixed_title(dir: &Path, title: &str) {
    assert_eq!(
        std::fs::read_to_string(dir.join("tasks/dup.typ")).unwrap(),
        "original content",
        "add-suffix must never overwrite the original"
    );
    let suffixed = std::fs::read_to_string(dir.join("tasks/dup-2.typ")).unwrap();
    assert!(
        suffixed.contains(&format!("title: {title:?}")),
        "{suffixed}"
    );
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
    assert_skip_reported_and_left_dup_untouched(dir.path(), &output);
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

    assert_original_untouched_and_suffixed_title(dir.path(), "Dup Two");
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

// -- templated-stem counterparts: the regression matrix ----------------------

/// Counterpart of `dedup_fail_is_the_default_and_errors_on_collision` /
/// `dedup_fail_flag_is_explicit_too`: before this fix, `output_stem` minted
/// `dup-2` for a colliding templated stem before `--dedup` ever saw a
/// collision, so `fail` could never fire under a template.
#[test]
fn dedup_fail_errors_on_collision_under_a_templated_stem() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_templated_stem_and_existing_dup(dir.path());

    support::mt(dir.path())
        .args(["add", "Dup", "--dedup", "fail"])
        .assert()
        .failure();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("tasks/dup.typ")).unwrap(),
        "original content",
        "a failed add must never touch the existing file under a templated stem either"
    );
    assert!(
        !dir.path().join("tasks/dup-2.typ").exists(),
        "fail must not fall back to minting a suffixed stem"
    );
}

/// Counterpart of `dedup_skip_exits_zero_writes_nothing_and_reports_existing_path`:
/// the exact scenario the task calls out — `contrib/ask-refactoring.sh`'s
/// re-run guard depends on `--dedup skip` meaning skip even when the project
/// templates `stem`.
#[test]
fn dedup_skip_exits_zero_writes_nothing_and_reports_existing_path_under_a_templated_stem() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_templated_stem_and_existing_dup(dir.path());

    let output = support::mt(dir.path())
        .args(["add", "Dup", "--dedup", "skip"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_skip_reported_and_left_dup_untouched(dir.path(), &output);
    assert!(
        !dir.path().join("tasks/dup-2.typ").exists(),
        "skip must not fall back to minting a suffixed stem"
    );
}

/// Counterpart of `dedup_add_suffix_writes_under_a_disambiguated_stem`: an
/// explicit `add-suffix` must keep working under a templated stem exactly as
/// it did before this fix.
#[test]
fn dedup_add_suffix_writes_under_a_disambiguated_stem_with_a_templated_stem() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_templated_stem_and_existing_dup(dir.path());

    support::mt(dir.path())
        .args(["add", "Dup", "--dedup", "add-suffix"])
        .assert()
        .success();

    assert_original_untouched_and_suffixed_title(dir.path(), "Dup");
}

/// The config-default half of the fix: a project that configures
/// `[tasks.naming] stem` and never touches `dedup` must still see `add-suffix`
/// with no `--dedup` flag at all — the behaviour a templated stem always had,
/// now sourced from `NamingConfig::effective_dedup` instead of a
/// pre-disambiguation special case in `output_stem`.
#[test]
fn templated_stem_dedup_defaults_to_add_suffix_without_the_flag() {
    let dir = tempfile::tempdir().unwrap();
    scaffold_with_templated_stem_and_existing_dup(dir.path());

    support::mt(dir.path())
        .args(["add", "Dup"])
        .assert()
        .success();

    assert_original_untouched_and_suffixed_title(dir.path(), "Dup");
}
