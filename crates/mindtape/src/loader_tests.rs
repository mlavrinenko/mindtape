#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::{Path, PathBuf};

use super::{body_after_marker, find_root, load_project};
use crate::config::Config;
use crate::numeric::NumValue;
use crate::parse_config;

const TASK_BODY: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Build\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".mindtape")).unwrap();
    std::fs::create_dir(dir.path().join("tasks")).unwrap();
    dir
}

fn write(root: &Path, rel: &str, body: &str) {
    std::fs::write(root.join(rel), body).unwrap();
}

#[test]
fn loads_tasks_and_skips_non_tasks() {
    let dir = setup();
    write(dir.path(), "tasks/build.typ", TASK_BODY);
    write(
        dir.path(),
        "tasks/notes.typ",
        "= Just notes\n\nNo marker here.\n",
    );

    let outcome = load_project(dir.path(), &Config::default());
    assert_eq!(outcome.errors, Vec::new());
    assert_eq!(outcome.tasks.len(), 1);
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.id, "build");
    assert_eq!(task.title, "Build");
    assert_eq!(task.status.as_ref().unwrap().name, "wip");
    assert_eq!(task.priority, Some(NumValue::Num(8.0)));
}

#[test]
fn loads_multi_unit_estimate_verbatim() {
    let dir = setup();
    write(
        dir.path(),
        "tasks/est.typ",
        "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"E\", estimate: (days: 1, hours: 2)),
  extras: (:),
))
",
    );

    // Units are opaque labels: stored as-is, never converted.
    let outcome = load_project(dir.path(), &Config::default());
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.estimate.get("days"), Some(&1.0));
    assert_eq!(task.estimate.get("hours"), Some(&2.0));
}

#[test]
fn records_eval_errors() {
    let dir = setup();
    write(
        dir.path(),
        "tasks/broken.typ",
        "#import \"@local/nope:9.9.9\": *\n",
    );

    let outcome = load_project(dir.path(), &Config::default());
    assert!(outcome.tasks.is_empty());
    assert_eq!(outcome.errors.len(), 1);
    assert_eq!(
        outcome.errors.first().unwrap().file,
        Path::new("tasks/broken.typ")
    );
}

#[test]
fn injects_folder_derived_tags() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tasks/backend/auth")).unwrap();
    write(dir.path(), "tasks/backend/auth/login.typ", TASK_BODY);
    write(dir.path(), "tasks/top.typ", TASK_BODY);

    let config = parse_config(
        r#"
[[tasks.roots]]
path = "tasks"
name = "mt"
derive-tags = true

[tasks.derive-tags]
collect = ["segments", "breadcrumbs"]
"#,
    )
    .unwrap();
    let outcome = load_project(dir.path(), &config);

    let nested = outcome.tasks.iter().find(|t| t.id == "login").unwrap();
    assert_eq!(
        nested.derived_tags,
        vec![
            "mt".to_owned(),
            "backend".to_owned(),
            "auth".to_owned(),
            "backend:auth".to_owned(),
        ]
    );
    // Authored tags stay untouched; derived live separately.
    assert!(nested.tags.is_empty());

    let top = outcome.tasks.iter().find(|t| t.id == "top").unwrap();
    assert_eq!(top.derived_tags, vec!["mt".to_owned()]);
}

#[test]
fn glob_gates_which_files_derive() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tasks/backend")).unwrap();
    std::fs::create_dir_all(dir.path().join("tasks/frontend")).unwrap();
    write(dir.path(), "tasks/backend/a.typ", TASK_BODY);
    write(dir.path(), "tasks/frontend/b.typ", TASK_BODY);

    let config = parse_config(
        r#"
[[tasks.roots]]
path = "tasks"
name = "mt"
derive-tags = "backend/**"
"#,
    )
    .unwrap();
    let outcome = load_project(dir.path(), &config);

    let inside = outcome.tasks.iter().find(|t| t.id == "a").unwrap();
    assert_eq!(
        inside.derived_tags,
        vec!["mt".to_owned(), "backend".to_owned()]
    );
    let outside = outcome.tasks.iter().find(|t| t.id == "b").unwrap();
    assert!(outside.derived_tags.is_empty());
}

/// A minimal `task` prelude stub: emits the `mindtape.task` marker so a harvested
/// file projects to a task without pulling the full in-tree prelude.
const PRELUDE_STUB: &str = "#let task(body, ..meta) = {\n  [#metadata((marker: \"mindtape.task\", fields: (title: meta.named().at(\"title\", default: \"\")), extras: (:))) <mindtape>]\n  body\n}\n";

fn write_pkg(root: &Path, dir_rel: &str, name: &str, version: &str, entry: &str) {
    let pkg = root.join(dir_rel);
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(
        pkg.join("typst.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"{version}\"\nentrypoint = \"lib.typ\"\n"
        ),
    )
    .unwrap();
    std::fs::write(pkg.join("lib.typ"), entry).unwrap();
}

// A bogus version no install would ship, so resolution depends only on the
// in-tree override — these stay hermetic regardless of any installed package.
#[test]
fn in_tree_lib_registered_when_named_mindtape() {
    let dir = setup();
    write_pkg(dir.path(), "lib", "mindtape", "99.99.99", PRELUDE_STUB);
    write(
        dir.path(),
        "tasks/t.typ",
        "#import \"@local/mindtape:99.99.99\": *\n#show: task.with(title: \"Hi\")\n= Body\n",
    );
    let outcome = load_project(dir.path(), &Config::default());
    assert_eq!(outcome.errors, Vec::new());
    assert_eq!(outcome.tasks.len(), 1);
    assert_eq!(outcome.tasks.first().unwrap().title, "Hi");
}

#[test]
fn foreign_lib_not_hijacked_as_mindtape() {
    // `<root>/lib` is some other package: the `mindtape` override must NOT fire,
    // so `@local/mindtape` is unresolved (no install here) and the file errors
    // rather than silently loading against the foreign lib.
    let dir = setup();
    write_pkg(dir.path(), "lib", "other", "99.99.99", PRELUDE_STUB);
    write(
        dir.path(),
        "tasks/t.typ",
        "#import \"@local/mindtape:99.99.99\": *\n#show: task.with(title: \"Hi\")\n= Body\n",
    );
    let outcome = load_project(dir.path(), &Config::default());
    assert!(
        outcome.tasks.is_empty(),
        "foreign lib must not resolve as the mindtape package"
    );
    assert_eq!(outcome.errors.len(), 1);
}

/// The real, in-tree prelude (not `PRELUDE_STUB`): proves the migrated
/// `links:` DSL — variadic helpers, the trailing-note idiom, and combining
/// several kinds with `+` — round-trips through the actual harvester, not just
/// hand-built `HVal` fixtures.
const REAL_PRELUDE: &str = include_str!("../../../lib/prelude.typ");

#[test]
fn real_prelude_parses_typed_links_end_to_end() {
    let dir = setup();
    write_pkg(dir.path(), "lib", "mindtape", "0.2.0", REAL_PRELUDE);
    write(
        dir.path(),
        "tasks/t.typ",
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(\n  \
           title: \"T\",\n  \
           links: depends-on(\"a.typ\", \"b.typ\") + related(\"c.typ\")[why],\n\
         )\n",
    );

    let outcome = load_project(dir.path(), &Config::default());
    assert_eq!(outcome.errors, Vec::new());
    let task = outcome.tasks.first().unwrap();

    let deps: Vec<&str> = task.link_targets("depends-on").collect();
    assert_eq!(deps, vec!["a.typ", "b.typ"]);

    let related: Vec<_> = task.links.iter().filter(|l| l.kind == "related").collect();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].task, "c.typ");
    assert_eq!(related[0].note.as_deref(), Some("why"));
}

#[test]
fn config_registers_extra_local_package() {
    // A configured [[typst.local-packages]] entry resolves a foreign @local
    // package from an in-tree path without a global install.
    let dir = setup();
    write_pkg(
        dir.path(),
        "vendor/helper",
        "helper",
        "0.1.0",
        "#let mk(t) = [#metadata((marker: \"mindtape.task\", fields: (title: t), extras: (:))) <mindtape>]\n",
    );
    write(
        dir.path(),
        "tasks/t.typ",
        "#import \"@local/helper:0.1.0\": *\n#mk(\"Hi\")\n",
    );
    let config =
        parse_config("[[typst.local-packages]]\nname = \"helper\"\npath = \"vendor/helper\"\n")
            .unwrap();
    let outcome = load_project(dir.path(), &config);
    assert_eq!(outcome.errors, Vec::new());
    assert_eq!(outcome.tasks.len(), 1);
    assert_eq!(outcome.tasks.first().unwrap().title, "Hi");
}

#[test]
fn find_root_locates_mindtape_dir() {
    let dir = setup();
    let child = dir.path().join("tasks");
    assert_eq!(find_root(&child).unwrap(), dir.path());
}

#[test]
fn find_root_returns_none_without_marker() {
    let dir = tempfile::tempdir().unwrap();
    assert!(find_root(dir.path()).is_none());
}

#[test]
fn explicit_id_wins_over_stem() {
    let dir = setup();
    write(
        dir.path(),
        "tasks/build.typ",
        "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"Build\", id: \"custom\"),
  extras: (:),
))
",
    );

    let outcome = load_project(dir.path(), &Config::default());
    assert_eq!(outcome.errors, Vec::new());
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.id, "custom", "explicit id: should win over stem");
}

#[test]
fn stem_used_when_id_absent() {
    let dir = setup();
    write(dir.path(), "tasks/my-task.typ", TASK_BODY);

    let outcome = load_project(dir.path(), &Config::default());
    let task = outcome.tasks.first().unwrap();
    assert_eq!(task.id, "my-task");
}

#[test]
fn duplicate_ids_produce_uniqueness_violation() {
    let dir = setup();
    // Two files, one with explicit id matching the other's stem.
    write(
        dir.path(),
        "tasks/alpha.typ",
        "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"Alpha\", id: \"shared\"),
  extras: (:),
))
",
    );
    write(
        dir.path(),
        "tasks/shared.typ",
        "#metadata((
  marker: \"mindtape.task\",
  fields: (title: \"Shared\"),
  extras: (:),
))
",
    );

    let outcome = load_project(dir.path(), &Config::default());
    let violations = crate::validate::validate(&outcome.tasks, &Config::default());
    assert!(
        violations.iter().any(|v| v.field == "id"),
        "expected uniqueness violation on duplicate id"
    );
}

#[test]
fn body_after_marker_captures_trimmed_prose_tail() {
    let src = "#import \"@local/mindtape:0.2.0\": *\n\n\
        #show: task.with(\n  title: \"T\",\n)\n\n\
        == Summary\n\nFirst line.\n\nSecond line.\n";
    assert_eq!(
        body_after_marker(src),
        "== Summary\n\nFirst line.\n\nSecond line."
    );
}

#[test]
fn body_after_marker_empty_without_body_or_marker() {
    assert_eq!(body_after_marker("#show: task.with(title: \"T\")\n"), "");
    assert_eq!(body_after_marker("no marker here\n"), "");
}

// --- Persistent content-hash cache ---------------------------------------

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
    write(dir.path(), "tasks/build.typ", TASK_BODY);
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
    write(dir.path(), "tasks/build.typ", TASK_BODY);
    let _ = load_project(dir.path(), &Config::default());

    // New content -> new hash -> cache miss -> fresh eval.
    write(
        dir.path(),
        "tasks/build.typ",
        &TASK_BODY.replace("Build", "Rebuilt"),
    );
    let out = load_project(dir.path(), &Config::default());
    assert_eq!(out.tasks.first().unwrap().title, "Rebuilt");
}

#[test]
fn config_change_busts_the_cache() {
    let dir = setup();
    write(dir.path(), "tasks/build.typ", TASK_BODY);
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
    write(
        dir.path(),
        "tasks/t.typ",
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(title: \"Hi\")\n",
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
    write(dir.path(), "tasks/build.typ", TASK_BODY);
    let _ = load_project(dir.path(), &Config::default());

    std::fs::write(sole_cache_entry(dir.path()), "{ not json").unwrap();
    let out = load_project(dir.path(), &Config::default());
    assert_eq!(out.errors, Vec::new());
    assert_eq!(out.tasks.first().unwrap().title, "Build");
}
