//! End-to-end coverage for the `role`/`assignee` filters and the actors
//! registry, driven through the compiled `mt` binary.
//!
//! `crates/mindtape/src/filter/actors_tests.rs` covers the same behaviour
//! against a hand-built `Config` only — nothing exercised it through TOML
//! parsing, project loading and `mt ls`/`mt check` until the 2026-08-07
//! manual session that surfaced the gap
//! (`truth/tasks/cover-the-actors-filters-end-to-end.typ`). This file closes
//! it.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

mod support;

/// `[[roles]]` + `[[actors]]`: `ada` (human) holds only `reviewer`, `gpt`
/// (llm) holds only `author` — a clean partition, so `role == "<x>"` matches
/// exactly the tasks assigned to one actor.
const REGISTRY: &str = "[[roles]]\nname = \"reviewer\"\n\n\
     [[roles]]\nname = \"author\"\n\n\
     [[actors]]\nname = \"ada\"\nkind = \"human\"\nroles = [\"reviewer\"]\n\n\
     [[actors]]\nname = \"gpt\"\nkind = \"llm\"\nroles = [\"author\"]\n";

/// `(stem, assignee)` for every task the fixture files. `eve` is never
/// declared in [`REGISTRY`] — the undeclared-assignee case.
const TASKS: &[(&str, &str)] = &[
    ("ada-one", "ada"),
    ("ada-two", "ada"),
    ("gpt-one", "gpt"),
    ("gpt-two", "gpt"),
    ("eve-mystery", "eve"),
];

fn task_typ(title: &str, assignee: &str) -> String {
    format!(
        "#import \"@local/mindtape:0.2.0\": *\n\n#show: task.with(\n  \
         title: \"{title}\",\n  \
         status: wip(2026, 6, 1),\n  \
         assignee: \"{assignee}\",\n)\n"
    )
}

/// A project with `[[roles]]`/`[[actors]]` ([`REGISTRY`]) plus `config_tail`,
/// and one task per [`TASKS`] entry.
fn scaffold(config_tail: &str) -> tempfile::TempDir {
    let config = format!("[tasks]\nroots = [\"tasks\"]\n\n{REGISTRY}{config_tail}");
    let dir = support::scaffold_project(&config);
    for (stem, assignee) in TASKS {
        std::fs::write(
            dir.path().join(format!("tasks/{stem}.typ")),
            task_typ(stem, assignee),
        )
        .unwrap();
    }
    dir
}

/// The sorted task ids `mt ls FILTER --json` returns, so a set comparison
/// does not depend on listing order.
fn ids_matching(dir: &Path, filter: &str) -> Vec<String> {
    let output = support::mt(dir)
        .args(["ls", filter, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let tasks = parsed
        .get("tasks")
        .and_then(serde_json::Value::as_array)
        .unwrap();
    let mut ids = Vec::new();
    for task in tasks {
        if let Some(id) = task.get("id").and_then(serde_json::Value::as_str) {
            ids.push(id.to_owned());
        }
    }
    ids.sort();
    ids
}

#[test]
fn role_filter_matches_exactly_the_assignees_holding_it() {
    let dir = scaffold("");
    assert_eq!(
        ids_matching(dir.path(), r#"role == "reviewer""#),
        ["ada-one", "ada-two"]
    );
    assert_eq!(
        ids_matching(dir.path(), r#"role == "author""#),
        ["gpt-one", "gpt-two"]
    );
}

#[test]
fn assignee_filter_matches_by_actor_name() {
    let dir = scaffold("");
    assert_eq!(
        ids_matching(dir.path(), r#"assignee == "gpt""#),
        ["gpt-one", "gpt-two"]
    );
    assert_eq!(
        ids_matching(dir.path(), r#"assignee == "ada""#),
        ["ada-one", "ada-two"]
    );
}

/// An assignee naming nobody in the registry claims no role at all — it must
/// not fall through and match either side of the partition. `role != "..."`
/// is documented (`crates/mindtape/src/filter/mod.rs`) as the exact complement of
/// `role == "..."` under the missing-field rule, so this only asserts `==`:
/// the undeclared assignee resolving `role` to anything but `Missing` would
/// be the actual regression this task exists to catch.
#[test]
fn undeclared_assignee_resolves_no_role_rather_than_falling_through() {
    let dir = scaffold("");
    assert!(ids_matching(dir.path(), "").contains(&"eve-mystery".to_owned()));
    assert!(!ids_matching(dir.path(), r#"role == "reviewer""#).contains(&"eve-mystery".to_owned()));
    assert!(!ids_matching(dir.path(), r#"role == "author""#).contains(&"eve-mystery".to_owned()));
}

#[test]
fn check_accepts_undeclared_assignee_as_free_text_when_allowed_is_unset() {
    let dir = scaffold("");
    support::mt(dir.path()).arg("check").assert().success();
}

#[test]
fn check_rejects_undeclared_assignee_once_allowed_is_set() {
    let dir = scaffold("[tasks.assignee]\nallowed = [\"ada\", \"gpt\"]\n");
    support::mt(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stdout(predicates::str::contains("not an allowed assignee"))
        .stdout(predicates::str::contains("eve-mystery"));
}

/// A registry violation fails the whole command at config load — before any
/// task is even read — rather than showing up as a per-task violation.
#[test]
fn registry_violations_surface_as_config_load_errors() {
    let cases = [
        (
            "[tasks]\nroots = [\"tasks\"]\n\n\
             [[roles]]\nname = \"reviewer\"\n\n\
             [[actors]]\nname = \"ada\"\nkind = \"human\"\nroles = [\"ghost\"]\n",
            "unknown role: 'ghost'",
        ),
        (
            "[tasks]\nroots = [\"tasks\"]\n\n\
             [[actors]]\nname = \"ada\"\nkind = \"human\"\n\n\
             [tasks.assignee]\nallowed = [\"ada\", \"eve\"]\n",
            "unknown actor: 'eve'",
        ),
    ];
    for (config, expected) in cases {
        let dir = support::scaffold_project(config);
        support::mt(dir.path())
            .arg("check")
            .assert()
            .failure()
            .stderr(predicates::str::contains(expected));
    }
}
