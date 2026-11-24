//! Persistent content-hash cache: a load serves its `.mindtape/cache/` entries
//! verbatim while a file/config/prelude hash stays unchanged, and bypasses them
//! (fresh eval) the moment any of those hashes moves. Shares
//! `loader/tests.rs`'s fixtures (`setup`, `write`, `write_pkg`, `TASK_BODY`,
//! `PRELUDE_STUB`).
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::test_support::{body_marker, project_with_file, write_file};
use std::path::{Path, PathBuf};

use typst_harvest::HVal;

use super::tests::{PRELUDE_STUB, TASK_BODY, lib_config, load_ok, setup, write_pkg};
use super::{LoadOutcome, load_project};
use crate::config::Config;
use crate::model::BodyMarker;
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

/// The first task's title from a fresh default-config load of `root`.
fn load_title(root: &Path) -> String {
    load_title_with(root, &Config::default())
}

/// The first task's title from a fresh load of `root` under `config`.
fn load_title_with(root: &Path, config: &Config) -> String {
    load_project(root, config)
        .tasks
        .first()
        .unwrap()
        .title
        .clone()
}

/// Write `tasks/build.typ` and load `root` once under the default config,
/// which fills the cache.
fn warm_build(root: &Path) -> LoadOutcome {
    write_file(root, "tasks/build.typ", TASK_BODY);
    load_project(root, &Config::default())
}

/// Doctor the cached entry, reload, and assert the doctored title came back:
/// the load read the cache and did not re-evaluate the untouched file.
fn assert_served_from_cache(root: &Path) {
    doctor_cache_title(&sole_cache_entry(root), "FROM-CACHE");
    assert_eq!(load_title(root), "FROM-CACHE");
}

#[test]
fn warm_load_serves_cache_and_skips_eval() {
    let dir = setup();
    let first = warm_build(dir.path());
    assert_eq!(first.tasks.first().unwrap().title, "Build");

    // An unchanged file must be served from its cached marker, not re-evaluated
    // (a re-eval would recover the file's real "Build" title).
    assert_served_from_cache(dir.path());
}

#[test]
fn edited_file_misses_and_reevals() {
    let dir = setup();
    let _ = warm_build(dir.path());

    // New content -> new hash -> cache miss -> fresh eval.
    write_file(
        dir.path(),
        "tasks/build.typ",
        TASK_BODY.replace("Build", "Rebuilt"),
    );
    assert_eq!(load_title(dir.path()), "Rebuilt");
}

#[test]
fn config_change_busts_the_cache() {
    let dir = project_with_file(setup(), "tasks/build.typ", TASK_BODY);
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
    let config = lib_config();
    write_pkg(dir.path(), "lib", "mindtape", "0.5.0", PRELUDE_STUB);
    write_file(
        dir.path(),
        "tasks/t.typ",
        crate::typst_task!("(title: \"Hi\")\n"),
    );
    let _ = load_project(dir.path(), &config);
    doctor_cache_title(&sole_cache_entry(dir.path()), "STALE");

    // Editing the prelude changes the lib hash -> build key -> re-eval.
    let bumped = format!("{PRELUDE_STUB}\n// bump\n");
    write_pkg(dir.path(), "lib", "mindtape", "0.5.0", &bumped);
    assert_eq!(load_title_with(dir.path(), &config), "Hi");
}

#[test]
fn corrupt_entry_falls_back_to_eval() {
    let dir = setup();
    let _ = warm_build(dir.path());

    std::fs::write(sole_cache_entry(dir.path()), "{ not json").unwrap();
    let out = load_ok(dir.path(), &Config::default());
    assert_eq!(out.tasks.first().unwrap().title, "Build");
}

#[test]
fn a_cache_hit_replays_the_files_typst_warnings() {
    let dir = setup();
    write_pkg(
        dir.path(),
        "lib",
        "mindtape",
        "0.5.0",
        crate::test_support::PRELUDE,
    );
    write_file(
        dir.path(),
        "tasks/t.typ",
        "<orphan>\n\
         #import \"@local/mindtape:0.5.0\": *\n\
         #show: task.with(title: \"T\")\n",
    );
    let first = load_project(dir.path(), &lib_config());
    assert_eq!(first.warnings.len(), 1, "{:?}", first.warnings);

    doctor_cache_title(&sole_cache_entry(dir.path()), "FROM-CACHE");
    let second = load_project(dir.path(), &lib_config());
    assert_eq!(second.tasks.first().unwrap().title, "FROM-CACHE");
    assert_eq!(second.warnings, first.warnings);
}

/// A task whose title comes from an imported module.
const IMPORTING_TASK: &str = "#import \"../shared.typ\": t\n#metadata((marker: \"mindtape.task\", fields: (title: t, status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none), priority: 8), extras: (:)))\n";

/// A task body carrying one labelled marker under a heading and one bare,
/// unlabelled marker after it, the shape a goals table emits into a section.
/// Everything except the `mindtape.task` marker is a body marker.
const MARKED_TASK: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Build\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))

== Goals

#metadata((marker: \"goals\", value: (goal: \"implementable-tasks\", verdict: \"helps\"))) <goals>

#metadata(\"bare\")
";

/// The sole task's body markers from a fresh default-config load of `root`.
fn load_markers(root: &Path) -> Vec<BodyMarker> {
    let out = load_ok(root, &Config::default());
    out.tasks.first().unwrap().markers.clone()
}

#[test]
fn a_heading_marker_reaches_the_task_in_document_order() {
    let dir = project_with_file(setup(), "tasks/build.typ", MARKED_TASK);
    let markers = load_markers(dir.path());
    assert_eq!(markers.len(), 2, "{markers:?}");
    assert_eq!(markers[0].label.as_deref(), Some("goals"));
    assert_eq!(markers[1].label, None);
    assert_eq!(markers[1].value, HVal::Str("bare".to_owned()));
}

#[test]
fn a_marker_an_imported_helper_emits_is_kept() {
    let dir = project_with_file(
        setup(),
        "shared.typ",
        "#let goals() = [#metadata((marker: \"goals\", value: \"helps\")) <goals>]\n",
    );
    write_file(
        dir.path(),
        "tasks/imp.typ",
        "#import \"../shared.typ\": goals\n#metadata((marker: \"mindtape.task\", fields: (title: \"Imp\", status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none), priority: 8), extras: (:)))\n\n== Goals\n\n#goals()\n",
    );
    let markers = load_markers(dir.path());
    assert_eq!(markers.len(), 1, "{markers:?}");
    assert_eq!(markers[0].label.as_deref(), Some("goals"));
    assert_eq!(
        markers[0].value,
        HVal::Dict(std::collections::BTreeMap::from([
            ("marker".to_owned(), HVal::Str("goals".to_owned())),
            ("value".to_owned(), HVal::Str("helps".to_owned())),
        ]))
    );
}

/// A task body whose markers carry a `datetime`, the one value JSON cannot
/// store as itself: once inside a dict and once inside a list.
const DATED_TASK: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Dated\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))

#metadata((day: datetime(year: 2027, month: 1, day: 2))) <dated>

#metadata((datetime(year: 2027, month: 1, day: 2), datetime(year: 2027, month: 1, day: 3))) <dated-list>
";

#[test]
fn a_date_valued_marker_reads_the_same_cold_and_warm() {
    let dir = project_with_file(setup(), "tasks/dated.typ", DATED_TASK);
    let cold = load_markers(dir.path());
    assert_eq!(
        cold,
        vec![
            body_marker(
                Some("dated"),
                HVal::Dict(std::collections::BTreeMap::from([(
                    "day".to_owned(),
                    HVal::Str("2027-01-02".to_owned()),
                )])),
            ),
            body_marker(
                Some("dated-list"),
                HVal::Array(vec![
                    HVal::Str("2027-01-02".to_owned()),
                    HVal::Str("2027-01-03".to_owned()),
                ]),
            ),
        ]
    );
    assert_eq!(load_markers(dir.path()), cold);
}

/// Rewrite a cached entry's first body marker label. A load that reads the
/// cache rather than re-evaluating the untouched file then reports the new
/// label in the marker list it builds.
fn doctor_cache_marker_label(entry: &Path, label: &str) {
    let text = std::fs::read_to_string(entry).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
    value["markers"][0]["label"] = serde_json::Value::from(label);
    std::fs::write(entry, serde_json::to_string(&value).unwrap()).unwrap();
}

#[test]
fn a_warm_hit_reports_the_same_markers_as_the_cold_eval() {
    let dir = project_with_file(setup(), "tasks/build.typ", MARKED_TASK);
    let cold = load_markers(dir.path());
    assert_eq!(cold.len(), 2, "{cold:?}");

    // The doctored label proves the second load reads the cached entry rather
    // than re-evaluating the file, and that the entry carries the marker list.
    doctor_cache_marker_label(&sole_cache_entry(dir.path()), "FROM-CACHE");
    let warm = load_markers(dir.path());
    let mut expected = cold;
    expected[0].label = Some("FROM-CACHE".to_owned());
    assert_eq!(warm, expected);
}

#[test]
fn edited_imported_module_busts_the_entry() {
    let dir = project_with_file(setup(), "shared.typ", "#let t = \"Title one\"\n");
    write_file(dir.path(), "tasks/imp.typ", IMPORTING_TASK);
    let first = load_project(dir.path(), &Config::default());
    assert_eq!(first.tasks.first().unwrap().title, "Title one");

    write_file(dir.path(), "shared.typ", "#let t = \"Title TWO\"\n");
    assert_eq!(load_title(dir.path()), "Title TWO");
}

#[test]
fn unchanged_imported_module_still_hits_the_cache() {
    let dir = project_with_file(setup(), "shared.typ", "#let t = \"Title one\"\n");
    write_file(dir.path(), "tasks/imp.typ", IMPORTING_TASK);
    let _ = load_project(dir.path(), &Config::default());
    assert_served_from_cache(dir.path());
}

#[test]
fn deleted_imported_module_misses() {
    let dir = project_with_file(setup(), "shared.typ", "#let t = \"Title one\"\n");
    write_file(dir.path(), "tasks/imp.typ", IMPORTING_TASK);
    let _ = load_project(dir.path(), &Config::default());
    std::fs::remove_file(dir.path().join("shared.typ")).unwrap();
    let second = load_project(dir.path(), &Config::default());
    assert!(second.tasks.is_empty());
    assert_eq!(second.errors.len(), 1);
}
