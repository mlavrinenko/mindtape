//! Persistent content-hash cache: a load's `.mindtape/cache/` entries are
//! served verbatim when a file/config/prelude hash is unchanged, and bypassed
//! (fresh eval) the moment any of those hashes moves. Shares
//! `loader/tests.rs`'s fixtures (`setup`, `write`, `write_pkg`, `TASK_BODY`,
//! `PRELUDE_STUB`).
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::test_support::write_file;
use std::path::{Path, PathBuf};

use super::load_project;
use super::tests::{PRELUDE_STUB, TASK_BODY, setup, write_pkg};
use crate::config::Config;
use crate::parse_config;

/// The sole build-key directory a single populated load leaves under
/// `.mindtape/cache/`.
fn build_key_dir(root: &Path) -> PathBuf {
    std::fs::read_dir(root.join(".mindtape/cache"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.is_dir())
        .unwrap()
}

/// The one cached task entry under the current build-key directory.
fn sole_cache_entry(root: &Path) -> PathBuf {
    std::fs::read_dir(build_key_dir(root))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .unwrap()
}

/// Rewrite a cached entry's marker title, so a load that reads the cache (rather
/// than re-evaluating the untouched file) is observable by the doctored value.
fn doctor_cache_title(entry: &Path, title: &str) {
    let text = std::fs::read_to_string(entry).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
    value["marker"]["fields"]["title"] = serde_json::Value::from(title);
    std::fs::write(entry, serde_json::to_string(&value).unwrap()).unwrap();
}

#[test]
fn warm_load_serves_cache_and_skips_eval() {
    let dir = setup();
    write_file(dir.path(), "tasks/build.typ", TASK_BODY);
    let first = load_project(dir.path(), &Config::default());
    assert_eq!(first.tasks.first().unwrap().title, "Build");

    // An unchanged file must be served from its cached marker, not re-evaluated
    // (a re-eval would recover the file's real "Build" title).
    doctor_cache_title(&sole_cache_entry(dir.path()), "FROM-CACHE");
    let second = load_project(dir.path(), &Config::default());
    assert_eq!(second.tasks.first().unwrap().title, "FROM-CACHE");
}

#[test]
fn edited_file_misses_and_reevals() {
    let dir = setup();
    write_file(dir.path(), "tasks/build.typ", TASK_BODY);
    let _ = load_project(dir.path(), &Config::default());

    // New content -> new hash -> cache miss -> fresh eval.
    write_file(
        dir.path(),
        "tasks/build.typ",
        TASK_BODY.replace("Build", "Rebuilt"),
    );
    let out = load_project(dir.path(), &Config::default());
    assert_eq!(out.tasks.first().unwrap().title, "Rebuilt");
}

#[test]
fn config_change_busts_the_cache() {
    let dir = setup();
    write_file(dir.path(), "tasks/build.typ", TASK_BODY);
    let base = parse_config("").unwrap();
    let _ = load_project(dir.path(), &base);
    doctor_cache_title(&sole_cache_entry(dir.path()), "STALE");

    // A different config -> different build key -> the stale entry is unreachable.
    let variant = parse_config("[project]\ncode = \"other\"\n").unwrap();
    let out = load_project(dir.path(), &variant);
    assert_eq!(out.tasks.first().unwrap().title, "Build");
}

#[test]
fn prelude_change_busts_the_cache() {
    let dir = setup();
    write_pkg(dir.path(), "lib", "mindtape", "0.2.0", PRELUDE_STUB);
    write_file(
        dir.path(),
        "tasks/t.typ",
        crate::typst_task!("(title: \"Hi\")\n"),
    );
    let _ = load_project(dir.path(), &Config::default());
    doctor_cache_title(&sole_cache_entry(dir.path()), "STALE");

    // Editing the prelude changes the lib hash -> build key -> re-eval.
    let bumped = format!("{PRELUDE_STUB}\n// bump\n");
    write_pkg(dir.path(), "lib", "mindtape", "0.2.0", &bumped);
    let out = load_project(dir.path(), &Config::default());
    assert_eq!(out.tasks.first().unwrap().title, "Hi");
}

#[test]
fn corrupt_entry_falls_back_to_eval() {
    let dir = setup();
    write_file(dir.path(), "tasks/build.typ", TASK_BODY);
    let _ = load_project(dir.path(), &Config::default());

    std::fs::write(sole_cache_entry(dir.path()), "{ not json").unwrap();
    let out = load_project(dir.path(), &Config::default());
    assert_eq!(out.errors, Vec::new());
    assert_eq!(out.tasks.first().unwrap().title, "Build");
}

#[test]
fn a_cache_hit_replays_the_files_typst_warnings() {
    let dir = setup();
    write_pkg(
        dir.path(),
        "lib",
        "mindtape",
        "0.2.0",
        crate::test_support::PRELUDE,
    );
    write_file(
        dir.path(),
        "tasks/t.typ",
        "<orphan>\n\
         #import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"T\")\n",
    );
    let first = load_project(dir.path(), &Config::default());
    assert_eq!(first.warnings.len(), 1, "{:?}", first.warnings);

    doctor_cache_title(&sole_cache_entry(dir.path()), "FROM-CACHE");
    let second = load_project(dir.path(), &Config::default());
    assert_eq!(second.tasks.first().unwrap().title, "FROM-CACHE");
    assert_eq!(second.warnings, first.warnings);
}

/// A task whose title comes from an imported module.
const IMPORTING_TASK: &str = "#import \"../shared.typ\": t\n#metadata((marker: \"mindtape.task\", fields: (title: t, status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none), priority: 8), extras: (:)))\n";

#[test]
fn edited_imported_module_busts_the_entry() {
    let dir = setup();
    write_file(dir.path(), "shared.typ", "#let t = \"Title one\"\n");
    write_file(dir.path(), "tasks/imp.typ", IMPORTING_TASK);
    let first = load_project(dir.path(), &Config::default());
    assert_eq!(first.tasks.first().unwrap().title, "Title one");

    write_file(dir.path(), "shared.typ", "#let t = \"Title TWO\"\n");
    let second = load_project(dir.path(), &Config::default());
    assert_eq!(second.tasks.first().unwrap().title, "Title TWO");
}

#[test]
fn unchanged_imported_module_still_hits_the_cache() {
    let dir = setup();
    write_file(dir.path(), "shared.typ", "#let t = \"Title one\"\n");
    write_file(dir.path(), "tasks/imp.typ", IMPORTING_TASK);
    let _ = load_project(dir.path(), &Config::default());
    doctor_cache_title(&sole_cache_entry(dir.path()), "FROM-CACHE");
    let second = load_project(dir.path(), &Config::default());
    assert_eq!(second.tasks.first().unwrap().title, "FROM-CACHE");
}

#[test]
fn deleted_imported_module_misses() {
    let dir = setup();
    write_file(dir.path(), "shared.typ", "#let t = \"Title one\"\n");
    write_file(dir.path(), "tasks/imp.typ", IMPORTING_TASK);
    let _ = load_project(dir.path(), &Config::default());
    std::fs::remove_file(dir.path().join("shared.typ")).unwrap();
    let second = load_project(dir.path(), &Config::default());
    assert!(second.tasks.is_empty());
    assert_eq!(second.errors.len(), 1);
}
