//! End-to-end tests for `mt recur` that shell out to the `mt` binary.
//!
//! Lives here (not an inline `#[path]` unit test) so the fixtures exercise the
//! real prelude (`every()`, `recurred-from`) and the compiled `mt` binary
//! end-to-end, mirroring `add_e2e.rs`/`read_no_write_e2e.rs`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::tempdir;
use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n";

/// A weekly, spawn-materialized recurring task (`materialize` defaults to
/// `spawn`), due today plus a week from whenever the fixture is written —
/// callers overwrite `due:`/`recur:` inline when a test needs a specific
/// stop-condition shape.
const WEEKLY_SPAWN: &str = mindtape::typst_task!(
    "(\n  \
title: \"Water plants\",\n  \
status: wip(2026, 6, 1),\n  \
priority: 3,\n  \
recur: every(weeks: 1),\n\
)\n\n== Body\n"
);

const WEEKLY_MUTATE: &str = mindtape::typst_task!(
    "(\n  \
title: \"Standup\",\n  \
status: wip(2026, 6, 1),\n  \
priority: 3,\n  \
recur: every(weeks: 1, materialize: \"mutate\"),\n\
)\n\n== Body\n"
);

fn scaffold(files: &[(&str, &str)]) -> tempfile::TempDir {
    support::scaffold_config(CONFIG, files)
}

fn mt(dir: &Path) -> assert_cmd::Command {
    // These recurrence-math tests assume the real wall clock, which is what
    // `support::mt` hands them: it clears any ambient `$MINDTAPE_NOW` pin, so a
    // caller that exports one cannot leak into "today" here — see
    // `date_pin_e2e.rs` for the pin's own tests.
    support::mt(dir)
}

fn tasks_dir_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir.join("tasks"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// The `(stem, source)` of the task whose `recurred-from` edge names `stem` —
/// one hop forward along a spawn chain.
fn successor_of(dir: &Path, stem: &str) -> Option<(String, String)> {
    let edge = format!(r#"recurred-from("{stem}.typ")"#);
    tasks_dir_entries(dir).into_iter().find_map(|name| {
        let source = std::fs::read_to_string(dir.join("tasks").join(&name)).ok()?;
        source
            .contains(&edge)
            .then(|| (name.trim_end_matches(".typ").to_owned(), source))
    })
}

/// The argument text of a task source's `due: on(…)`.
fn due_of(source: &str) -> String {
    let rest = source.split_once("due: on(").unwrap().1;
    rest.split_once(')').unwrap().0.to_owned()
}

#[test]
fn spawn_writes_a_new_file_linked_back_to_the_predecessor() {
    let dir = scaffold(&[("water-plants.typ", WEEKLY_SPAWN)]);

    support::run_ok(dir.path(), &["recur", "water-plants"]);

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

    support::check_ok(dir.path());
}

#[test]
fn dry_run_writes_nothing() {
    let dir = scaffold(&[("water-plants.typ", WEEKLY_SPAWN)]);

    support::run_ok(dir.path(), &["recur", "water-plants", "--dry-run"]);

    assert_eq!(
        tasks_dir_entries(dir.path()),
        vec!["water-plants.typ".to_owned()]
    );
}

#[test]
fn double_spawn_guard_refuses_a_second_spawn_without_force() {
    let dir = scaffold(&[("water-plants.typ", WEEKLY_SPAWN)]);

    support::run_ok(dir.path(), &["recur", "water-plants"]);
    mt(dir.path())
        .args(["recur", "water-plants"])
        .assert()
        .failure();
    assert_eq!(tasks_dir_entries(dir.path()).len(), 2);

    // --force bypasses the guard and spawns a third file.
    support::run_ok(dir.path(), &["recur", "water-plants", "--force"]);
    assert_eq!(tasks_dir_entries(dir.path()).len(), 3);
}

#[test]
fn mutate_refreshes_the_same_file_in_place() {
    let dir = scaffold(&[("standup.typ", WEEKLY_MUTATE)]);

    support::run_ok(dir.path(), &["recur", "standup"]);

    assert_eq!(
        tasks_dir_entries(dir.path()),
        vec!["standup.typ".to_owned()]
    );
    let refreshed = std::fs::read_to_string(dir.path().join("tasks/standup.typ")).unwrap();
    assert!(refreshed.contains("status: proposed("), "{refreshed}");
    assert!(!refreshed.contains("recurred-from"), "{refreshed}");

    support::check_ok(dir.path());
}

/// Scaffold a weekly-spawn task with its `recur:` replaced by
/// `recur_override`, run `mt recur`, and assert it refuses with a stderr
/// message containing `expect` — writing nothing.
fn assert_recur_refused(recur_override: &str, expect: &str) {
    let body = WEEKLY_SPAWN.replace("recur: every(weeks: 1),", recur_override);
    let dir = scaffold(&[("water-plants.typ", &body)]);

    let output = mt(dir.path())
        .args(["recur", "water-plants"])
        .assert()
        .failure();
    let stderr = support::stderr_of(&output);
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
    let dir = scaffold(&[("water-plants.typ", WEEKLY_SPAWN)]);

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
    let dir = scaffold(&[("standup.typ", WEEKLY_MUTATE)]);

    let output = mt(dir.path()).args(["done", "standup"]).assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout).into_owned();
    assert!(stdout.contains("ends the series"), "{stdout}");
}

/// Spawning from an occurrence that is itself a spawn — every hop of the
/// ordinary loop after the first. The new occurrence's `recurred-from` names
/// its immediate predecessor and nothing else: that kind allows one edge, so
/// carrying the older hop forward as well wrote a file `mt check` refuses.
#[test]
fn a_second_hop_names_only_its_immediate_predecessor() {
    let dir = scaffold(&[("water-plants.typ", WEEKLY_SPAWN)]);

    support::run_ok(dir.path(), &["recur", "water-plants"]);
    let first = tasks_dir_entries(dir.path())
        .into_iter()
        .find(|n| n != "water-plants.typ")
        .unwrap();
    let first_stem = first.trim_end_matches(".typ").to_owned();

    support::run_ok(dir.path(), &["recur", &first_stem]);

    let second = tasks_dir_entries(dir.path())
        .into_iter()
        .find(|n| *n != first && n != "water-plants.typ")
        .unwrap();
    let source = std::fs::read_to_string(dir.path().join("tasks").join(&second)).unwrap();
    assert!(
        source.contains(&format!(r#"recurred-from("{first_stem}.typ")"#)),
        "{source}"
    );
    assert!(
        !source.contains(r#"recurred-from("water-plants.typ")"#),
        "the older hop must not be carried forward: {source}"
    );

    support::check_ok(dir.path());
}

/// The defect this seam closes for `recur`: a spawn whose occurrence would not
/// validate is refused before the write, so the tasks directory is exactly as
/// it was. The fixture's `estimate` unit is outside the configured allow-list,
/// which the predecessor is exempt from (terminal) and the fresh occurrence is
/// not.
#[test]
fn a_rejected_spawn_creates_no_file() {
    let dir = tempdir();
    let config = "[tasks]\nroots = [\"tasks\"]\n\n[tasks.tags]\nallowed = [\"ok\"]\n";
    let body = mindtape::typst_task!(
        "(\n  \
title: \"Water plants\",\n  \
status: wip(2026, 6, 1),\n  \
tags: (\"nope\",),\n  \
recur: every(weeks: 1),\n\
)\n\n== Body\n"
    );
    std::fs::create_dir_all(dir.path().join(".mindtape")).unwrap();
    write_file(dir.path(), ".mindtape/config.toml", config);
    support::install_lib(dir.path());
    write_file(dir.path(), "tasks/water-plants.typ", body);

    let output = mt(dir.path())
        .args(["recur", "water-plants"])
        .assert()
        .failure();

    let stderr = support::stderr_of(&output);
    assert!(
        stderr.contains("error: <new task>:"),
        "a blocking pre-write problem reports as an error: {stderr}"
    );
    assert_eq!(
        tasks_dir_entries(dir.path()),
        vec!["water-plants.typ".to_owned()],
        "a rejected occurrence must never reach the disk"
    );
}

/// `--advance N` for `N > 1` over a rule anchored on completion — the default
/// anchor, and the one every hop but the first has to stop honouring. Both
/// halves of the chain used to collapse onto the first occurrence: the anchor
/// re-resolved from today each hop, so all N planned tomorrow, and the stem
/// minter disambiguated against disk alone, so all N claimed one filename.
#[test]
fn advance_lays_out_a_chain_of_distinct_occurrences() {
    let body = WEEKLY_SPAWN.replace("recur: every(weeks: 1),", "recur: every(days: 1),");
    let dir = scaffold(&[("water-plants.typ", &body)]);

    support::run_ok(dir.path(), &["recur", "water-plants", "--advance", "3"]);

    let entries = tasks_dir_entries(dir.path());
    assert_eq!(entries.len(), 4, "{entries:?}");

    // Walk the chain from the original: each hop must exist and be dated a day
    // past the one before it.
    let mut stem = "water-plants".to_owned();
    let mut dues = Vec::new();
    for hop in 1..=3 {
        let (next_stem, source) = successor_of(dir.path(), &stem)
            .unwrap_or_else(|| panic!("hop {hop}: nothing recurred-from {stem}: {entries:?}"));
        dues.push(due_of(&source));
        stem = next_stem;
    }
    let mut distinct = dues.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), 3, "occurrence dates must differ: {dues:?}");

    support::check_ok(dir.path());
}

#[test]
fn no_recur_field_is_a_hard_error() {
    let plain = mindtape::typst_task!("(\n  title: \"Plain\",\n  status: wip(2026, 6, 1),\n)\n");
    let dir = scaffold(&[("plain.typ", plain)]);

    mt(dir.path()).args(["recur", "plain"]).assert().failure();
}

#[test]
fn a_task_without_recur_says_so_and_what_to_do() {
    let body = WEEKLY_SPAWN.replace("  recur: every(weeks: 1),\n", "");
    let dir = scaffold(&[("water-plants.typ", &body)]);

    mt(dir.path())
        .args(["recur", "water-plants"])
        .assert()
        .failure()
        .stderr(predicates::str::diff(
            "error: task has no recur field: 'water-plants'\n\
                 hint: add a recur: field to the task, or pick one that has it\n",
        ));
}

#[test]
fn advance_that_runs_out_of_series_warns_and_keeps_the_spawns_it_planned() {
    let body = WEEKLY_SPAWN.replace(
        "recur: every(weeks: 1),",
        "recur: every(weeks: 1, until: on(2026, 6, 30)),",
    );
    let dir = scaffold(&[("water-plants.typ", &body)]);

    mt(dir.path())
        .env("MINDTAPE_NOW", "2026-06-18")
        .args(["recur", "water-plants", "--advance", "3"])
        .assert()
        .success()
        .stderr(predicates::str::diff(
            "warning: stopping early: series ended: the next occurrence is past until: '2026-07-02'\n",
        ));
    assert_eq!(tasks_dir_entries(dir.path()).len(), 2);
}
