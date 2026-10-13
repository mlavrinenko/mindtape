//! End-to-end tests for the clock `mt` reads: the pinnable override
//! (`$MINDTAPE_NOW`, see
//! `truth/tasks/pinnable-clock-for-reproducible-output.typ`) and, unpinned,
//! the local calendar date it falls back to (see
//! `truth/tasks/decide-whether-today-is-utc-or-local.typ`).
//!
//! These shell out to the compiled `mt` binary (like `recur_e2e.rs`) rather
//! than calling `mindtape::date` directly: the override is resolved once into
//! a process-wide cache in `main` (`crates/mindtape/src/date/pin.rs`), so the
//! only race-free way to prove two different pins actually take effect is two
//! separate processes — each `assert_cmd` invocation is its own OS process
//! with its own env, so this sidesteps the shared-global-state hazard a
//! same-process test (mutating `std::env` or re-calling `init_pinned_today`)
//! would hit.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// A non-terminal task, status-dated 2026-06-01, due 2026-07-10 — something
/// for `now()`/`due`/`stale()` filters and the today-stamping commands to bite
/// on once "today" is pinned around those dates.
const WIRE_OAUTH: &str = mindtape::typst_task!(
    "(\n  \
title: \"Wire OAuth\",\n  \
status: wip(2026, 6, 1),\n  \
priority: 5,\n  \
due: on(2026, 7, 10),\n\
)\n\n== Body\n"
);

fn scaffold() -> tempfile::TempDir {
    support::scaffold_config(CONFIG, &[("wire-oauth.typ", WIRE_OAUTH)])
}

fn mt(dir: &Path) -> assert_cmd::Command {
    support::mt(dir)
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
    // `mt ls --json` wraps the task array in an `{"tasks": [...], "applied":
    // {...}}` envelope (`truth/tasks/conditional-ls-rules.typ`).
    parsed
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .unwrap()
        .iter()
        .filter_map(|task| task.get("id").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

/// File `name` with `env` set on the child only, and read back the date `mt`
/// stamped into its `proposed(...)` status, normalised to `YYYY-MM-DD` so two
/// stamps compare as dates rather than as raw Typst arguments.
///
/// `#[cfg(unix)]` with its callers: on Windows the whole timezone pair below
/// compiles to nothing, and an ungated helper would only be dead code there.
#[cfg(unix)]
fn stamped_add_date(dir: &Path, name: &str, env: &[(&str, &str)]) -> String {
    let mut cmd = mt(dir);
    cmd.args(["add", "--name", name, "--set", &format!("title={name}")]);
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd.assert().success();

    let written = std::fs::read_to_string(dir.join("tasks").join(format!("{name}.typ"))).unwrap();
    let (stamp, _) = written
        .split_once("proposed(")
        .and_then(|(_, rest)| rest.split_once(')'))
        .unwrap_or_else(|| panic!("no proposed(...) status in {written}"));
    let (year, rest) = stamp.split_once(',').unwrap();
    let (month, day) = rest.split_once(',').unwrap();
    format!("{:04}-{:02}-{:02}", num(year), num(month), num(day))
}

#[cfg(unix)]
fn num(part: &str) -> u32 {
    part.trim().parse().unwrap()
}

/// Unpinned, "today" is the machine's local calendar date, not UTC.
///
/// POSIX `TZ` strings, so no tzdb file has to exist: the offset is what gets
/// added to local time to reach UTC, making `MTE-14` UTC+14 and `MTW+12`
/// UTC-12 — the two extremes of the inhabited calendar, 26 hours apart. At
/// every instant their dates differ, and east is always the later one; a
/// UTC-stamping `mt` would write the same date on both sides.
///
/// `#[cfg(unix)]`: chrono reads the Windows zone through the platform API,
/// which no environment variable overrides, so there is nothing to set there.
#[cfg(unix)]
#[test]
fn today_is_the_local_calendar_date_not_utc() {
    let dir = scaffold();

    let east = stamped_add_date(dir.path(), "east", &[("TZ", "MTE-14")]);
    let west = stamped_add_date(dir.path(), "west", &[("TZ", "MTW+12")]);
    assert!(
        east > west,
        "UTC+14 stamped {east}, UTC-12 stamped {west}: 26 hours apart must be different dates"
    );
}

/// The pin outranks the local zone: with `$MINDTAPE_NOW` set, the offset that
/// would otherwise decide the date changes nothing.
#[cfg(unix)]
#[test]
fn mindtape_now_outranks_the_local_timezone() {
    let dir = scaffold();

    let pin = [("MINDTAPE_NOW", "2026-09-05")];
    let east = stamped_add_date(dir.path(), "east", &[("TZ", "MTE-14"), pin[0]]);
    let west = stamped_add_date(dir.path(), "west", &[("TZ", "MTW+12"), pin[0]]);
    assert_eq!(east, "2026-09-05");
    assert_eq!(west, "2026-09-05");
}

#[test]
fn mindtape_now_drives_the_now_filter() {
    let dir = scaffold();

    // Pinned before the due date: not yet overdue.
    let before = ls_ids(dir.path(), "due < now()", &[("MINDTAPE_NOW", "2026-07-01")]);
    assert!(before.is_empty(), "{before:?}");

    // Pinned after the due date: overdue.
    let after = ls_ids(dir.path(), "due < now()", &[("MINDTAPE_NOW", "2026-07-11")]);
    assert_eq!(after, vec!["wire-oauth"]);
}

#[test]
fn mindtape_now_rfc3339_timestamp_truncates_to_its_date() {
    let dir = scaffold();
    let ids = ls_ids(
        dir.path(),
        "due < now()",
        &[("MINDTAPE_NOW", "2026-07-11T23:59:59Z")],
    );
    assert_eq!(ids, vec!["wire-oauth"]);
}

#[test]
fn mindtape_now_drives_the_stale_filter() {
    let dir = scaffold();

    // Status dated 2026-06-01, no `reviewed:`: `stale(30)` trips once "today"
    // is more than 30 days past that.
    let fresh = ls_ids(dir.path(), "stale(30)", &[("MINDTAPE_NOW", "2026-06-15")]);
    assert!(fresh.is_empty(), "{fresh:?}");

    let stale = ls_ids(dir.path(), "stale(30)", &[("MINDTAPE_NOW", "2026-08-01")]);
    assert_eq!(stale, vec!["wire-oauth"]);
}

#[test]
fn malformed_mindtape_now_is_a_hard_error_naming_the_variable() {
    let dir = scaffold();

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
    let dir = scaffold();

    mt(dir.path())
        .env_remove("MINDTAPE_NOW")
        .arg("ls")
        .assert()
        .success();
}

#[test]
fn mindtape_now_stamps_the_pinned_date_on_done() {
    let dir = scaffold();

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
    let dir = scaffold();

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
    let dir = scaffold();

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
    let dir = support::scaffold_project(CONFIG);
    write_file(
        dir.path(),
        "tasks/water-plants.typ",
        mindtape::typst_task!(
            "(\n  \
         title: \"Water plants\",\n  \
         status: wip(2026, 6, 1),\n  \
         priority: 3,\n  \
         recur: every(weeks: 1),\n\
         )\n\n== Body\n"
        ),
    );

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
