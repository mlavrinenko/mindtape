//! End-to-end tests for the pinnable clock (`$MINDTAPE_NOW`, see
//! `truth/tasks/pinnable-clock-for-reproducible-output.typ`).
//!
//! These shell out to the compiled `mt` binary (like `recur_e2e.rs`) rather
//! than calling `mindtape::date` directly: the override is resolved once into
//! a process-wide cache in `main` (`crates/mindtape/src/date_pin.rs`), so the
//! only race-free way to prove two different pins actually take effect is two
//! separate processes — each `assert_cmd` invocation is its own OS process
//! with its own env, so this sidesteps the shared-global-state hazard a
//! same-process test (mutating `std::env` or re-calling `init_pinned_today`)
//! would hit.
#![allow(clippy::unwrap_used)]

use std::path::Path;

mod support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// A non-terminal task, status-dated 2026-06-01, due 2026-07-10 — something
/// for `now()`/`due`/`stale()` filters and the today-stamping commands to bite
/// on once "today" is pinned around those dates.
const WIRE_OAUTH: &str = "#import \"@local/mindtape:0.2.0\": *\n\n\
#show: task.with(\n  \
title: \"Wire OAuth\",\n  \
status: wip(2026, 6, 1),\n  \
priority: 5,\n  \
due: on(2026, 7, 10),\n\
)\n\n== Body\n";

fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), CONFIG).unwrap();
    support::install_lib(dir);
    std::fs::write(dir.join("tasks/wire-oauth.typ"), WIRE_OAUTH).unwrap();
}

fn mt(dir: &Path) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("mt").unwrap();
    cmd.current_dir(dir);
    cmd
}

/// `mt ls '<filter>' --json` task ids, with `env` set on the child process
/// only (never mutates the parent test process's real environment).
fn ls_ids(dir: &Path, filter: &str, env: &[(&str, &str)]) -> Vec<String> {
    let mut cmd = mt(dir);
    cmd.args(["ls", filter, "--json"]);
    for (key, value) in env {
        cmd.env(key, value);
    }
    let output = cmd.assert().success().get_output().stdout.clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    parsed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|task| task.get("id").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn mindtape_now_drives_the_now_filter() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    // Pinned before the due date: not yet overdue.
    let before = ls_ids(dir.path(), "due < now()", &[("MINDTAPE_NOW", "2026-07-01")]);
    assert!(before.is_empty(), "{before:?}");

    // Pinned after the due date: overdue.
    let after = ls_ids(dir.path(), "due < now()", &[("MINDTAPE_NOW", "2026-07-11")]);
    assert_eq!(after, vec!["wire-oauth"]);
}

#[test]
fn mindtape_now_rfc3339_timestamp_truncates_to_its_date() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    let ids = ls_ids(
        dir.path(),
        "due < now()",
        &[("MINDTAPE_NOW", "2026-07-11T23:59:59Z")],
    );
    assert_eq!(ids, vec!["wire-oauth"]);
}

#[test]
fn mindtape_now_drives_the_stale_filter() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    // Status dated 2026-06-01, no `reviewed:`: `stale(30)` trips once "today"
    // is more than 30 days past that.
    let fresh = ls_ids(dir.path(), "stale(30)", &[("MINDTAPE_NOW", "2026-06-15")]);
    assert!(fresh.is_empty(), "{fresh:?}");

    let stale = ls_ids(dir.path(), "stale(30)", &[("MINDTAPE_NOW", "2026-08-01")]);
    assert_eq!(stale, vec!["wire-oauth"]);
}

#[test]
fn malformed_mindtape_now_is_a_hard_error_naming_the_variable() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    let output = mt(dir.path())
        .env("MINDTAPE_NOW", "not-a-date")
        .arg("ls")
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr).into_owned();
    assert!(stderr.contains("MINDTAPE_NOW"), "{stderr}");
    assert!(stderr.contains("not-a-date"), "{stderr}");
}

#[test]
fn unset_falls_back_to_the_system_clock() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    mt(dir.path())
        .env_remove("MINDTAPE_NOW")
        .arg("ls")
        .assert()
        .success();
}

#[test]
fn mindtape_now_stamps_the_pinned_date_on_done() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    mt(dir.path())
        .env("MINDTAPE_NOW", "2026-09-01")
        .args(["done", "wire-oauth"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-oauth.typ")).unwrap();
    assert!(written.contains("status: done(2026, 9, 1)"), "{written}");
}

#[test]
fn mindtape_now_stamps_the_pinned_date_on_review() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    mt(dir.path())
        .env("MINDTAPE_NOW", "2026-09-02")
        .args(["review", "wire-oauth"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-oauth.typ")).unwrap();
    assert!(written.contains("reviewed: on(2026, 9, 2)"), "{written}");
}

#[test]
fn mindtape_now_stamps_the_pinned_date_on_add() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    mt(dir.path())
        .env("MINDTAPE_NOW", "2026-09-03")
        .args(["add", "--name", "fresh-task", "--set", "title=Fresh task"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/fresh-task.typ")).unwrap();
    assert!(
        written.contains("status: proposed(2026, 9, 3)"),
        "{written}"
    );
}

#[test]
fn mindtape_now_stamps_the_pinned_date_on_recur_spawn() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    std::fs::write(dir.path().join(".mindtape/config.toml"), CONFIG).unwrap();
    support::install_lib(dir.path());
    std::fs::write(
        dir.path().join("tasks/water-plants.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  \
         title: \"Water plants\",\n  \
         status: wip(2026, 6, 1),\n  \
         priority: 3,\n  \
         recur: every(weeks: 1),\n\
         )\n\n== Body\n",
    )
    .unwrap();

    mt(dir.path())
        .env("MINDTAPE_NOW", "2026-09-04")
        .args(["recur", "water-plants"])
        .assert()
        .success();

    let entries: Vec<String> = std::fs::read_dir(dir.path().join("tasks"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    let spawned_name = entries
        .iter()
        .find(|n| n.as_str() != "water-plants.typ")
        .unwrap();
    let spawned = std::fs::read_to_string(dir.path().join("tasks").join(spawned_name)).unwrap();
    assert!(
        spawned.contains("status: proposed(2026, 9, 4)"),
        "{spawned}"
    );
}
