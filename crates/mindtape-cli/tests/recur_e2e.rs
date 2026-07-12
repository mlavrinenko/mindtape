//! End-to-end tests for `mt recur` that shell out to the `mt` binary.
//!
//! Lives here (not an inline `#[path]` unit test) so the fixtures exercise the
//! real prelude (`every()`, `recurred-from`) and the compiled `mt` binary
//! end-to-end, mirroring `add_e2e.rs`/`read_no_write_e2e.rs`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// A weekly, spawn-materialized recurring task (`materialize` defaults to
/// `spawn`), due today plus a week from whenever the fixture is written —
/// callers overwrite `due:`/`recur:` inline when a test needs a specific
/// stop-condition shape.
const WEEKLY_SPAWN: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Water plants\",\n  \
status: wip(2026, 6, 1),\n  \
priority: 3,\n  \
recur: every(weeks: 1),\n\
)\n\n== Body\n";

const WEEKLY_MUTATE: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Standup\",\n  \
status: wip(2026, 6, 1),\n  \
priority: 3,\n  \
recur: every(weeks: 1, materialize: \"mutate\"),\n\
)\n\n== Body\n";

fn scaffold(dir: &Path, files: &[(&str, &str)]) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), CONFIG).unwrap();
    support::install_lib(dir);
    for (name, body) in files {
        std::fs::write(dir.join("tasks").join(name), body).unwrap();
    }
}

fn mt(dir: &Path) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("mt").unwrap();
    cmd.current_dir(dir)
        // These recurrence-math tests assume the real wall clock: clear any
        // ambient `$MINDTAPE_NOW` pin so a caller that exports one doesn't leak
        // into "today" here — see `date_pin_e2e.rs` for the pin's own tests.
        .env_remove("MINDTAPE_NOW");
    cmd
}

fn tasks_dir_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir.join("tasks"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn spawn_writes_a_new_file_linked_back_to_the_predecessor() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), &[("water-plants.typ", WEEKLY_SPAWN)]);

    mt(dir.path())
        .args(["recur", "water-plants"])
        .assert()
        .success();

    let entries = tasks_dir_entries(dir.path());
    assert_eq!(entries.len(), 2, "{entries:?}");
    let spawned_name = entries
        .iter()
        .find(|n| n.as_str() != "water-plants.typ")
        .unwrap();
    let spawned = std::fs::read_to_string(dir.path().join("tasks").join(spawned_name)).unwrap();
    assert!(
        spawned.contains(r#"recurred-from("water-plants.typ")"#),
        "{spawned}"
    );
    assert!(spawned.contains("status: proposed("), "{spawned}");
    assert!(spawned.contains("recur: every(weeks: 1)"), "{spawned}");

    // The predecessor's own status is untouched by a spawn.
    let original = std::fs::read_to_string(dir.path().join("tasks/water-plants.typ")).unwrap();
    assert!(original.contains("status: wip("), "{original}");

    mt(dir.path()).arg("check").assert().success();
}

#[test]
fn dry_run_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), &[("water-plants.typ", WEEKLY_SPAWN)]);

    mt(dir.path())
        .args(["recur", "water-plants", "--dry-run"])
        .assert()
        .success();

    assert_eq!(
        tasks_dir_entries(dir.path()),
        vec!["water-plants.typ".to_owned()]
    );
}

#[test]
fn double_spawn_guard_refuses_a_second_spawn_without_force() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), &[("water-plants.typ", WEEKLY_SPAWN)]);

    mt(dir.path())
        .args(["recur", "water-plants"])
        .assert()
        .success();
    mt(dir.path())
        .args(["recur", "water-plants"])
        .assert()
        .failure();
    assert_eq!(tasks_dir_entries(dir.path()).len(), 2);

    // --force bypasses the guard and spawns a third file.
    mt(dir.path())
        .args(["recur", "water-plants", "--force"])
        .assert()
        .success();
    assert_eq!(tasks_dir_entries(dir.path()).len(), 3);
}

#[test]
fn mutate_refreshes_the_same_file_in_place() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), &[("standup.typ", WEEKLY_MUTATE)]);

    mt(dir.path()).args(["recur", "standup"]).assert().success();

    assert_eq!(
        tasks_dir_entries(dir.path()),
        vec!["standup.typ".to_owned()]
    );
    let refreshed = std::fs::read_to_string(dir.path().join("tasks/standup.typ")).unwrap();
    assert!(refreshed.contains("status: proposed("), "{refreshed}");
    assert!(!refreshed.contains("recurred-from"), "{refreshed}");

    mt(dir.path()).arg("check").assert().success();
}

/// Scaffold a weekly-spawn task with its `recur:` replaced by
/// `recur_override`, run `mt recur`, and assert it refuses with a stderr
/// message containing `expect` — writing nothing.
fn assert_recur_refused(recur_override: &str, expect: &str) {
    let dir = tempfile::tempdir().unwrap();
    let body = WEEKLY_SPAWN.replace("recur: every(weeks: 1),", recur_override);
    scaffold(dir.path(), &[("water-plants.typ", &body)]);

    let output = mt(dir.path())
        .args(["recur", "water-plants"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr).into_owned();
    assert!(stderr.contains(expect), "{stderr}");
    assert_eq!(
        tasks_dir_entries(dir.path()),
        vec!["water-plants.typ".to_owned()]
    );
}

#[test]
fn count_zero_refuses_to_advance() {
    assert_recur_refused("recur: every(weeks: 1, count: 0),", "exhausted");
}

#[test]
fn until_in_the_past_ends_the_series() {
    // 2020-01-01 is behind any date this test could plausibly run on.
    assert_recur_refused(
        "recur: every(weeks: 1, until: on(2020, 1, 1)),",
        "series ended",
    );
}

#[test]
fn done_notice_points_at_mt_recur_for_a_spawn_series() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), &[("water-plants.typ", WEEKLY_SPAWN)]);

    let output = mt(dir.path())
        .args(["done", "water-plants"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout).into_owned();
    assert!(stdout.contains("mt recur"), "{stdout}");

    let after = std::fs::read_to_string(dir.path().join("tasks/water-plants.typ")).unwrap();
    assert!(
        after.contains("recur: every(weeks: 1)"),
        "recur must survive a pure close: {after}"
    );
}

#[test]
fn done_notice_says_mutate_series_ends() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), &[("standup.typ", WEEKLY_MUTATE)]);

    let output = mt(dir.path()).args(["done", "standup"]).assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout).into_owned();
    assert!(stdout.contains("ends the series"), "{stdout}");
}

#[test]
fn no_recur_field_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    let plain = "#import \"@local/mindtape:0.2.0\": *\n\n#show: task.with(\n  title: \"Plain\",\n  status: wip(2026, 6, 1),\n)\n";
    scaffold(dir.path(), &[("plain.typ", plain)]);

    mt(dir.path()).args(["recur", "plain"]).assert().failure();
}
