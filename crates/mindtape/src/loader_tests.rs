#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::Path;

use super::{find_root, find_root_bounded, load_project};
use crate::config::Config;
use crate::numeric::NumValue;
use crate::parse_config;

pub(super) const TASK_BODY: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Build\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

#[test]
fn first_task_marker_finds_mindtape_task() {
    use std::collections::BTreeMap;
    use typst_harvest::{HVal, Location, Marker};
    let loc = || {
        Some(Location {
            path: "t.typ".into(),
            line: 1,
            column: 1,
        })
    };
    let mk = |v: &str| Marker {
        value: HVal::Dict(BTreeMap::from([("marker".into(), HVal::Str(v.into()))])),
        label: None,
        location: loc(),
    };
    assert_eq!(
        super::first_task_marker(&[mk("other"), mk("mindtape.task")])
            .and_then(|v| v.get("marker").and_then(HVal::as_str)),
        Some("mindtape.task")
    );
}

#[test]
fn first_task_marker_returns_none_without_mindtape_task() {
    assert!(super::first_task_marker(&[]).is_none());
    use std::collections::BTreeMap;
    use typst_harvest::{HVal, Location, Marker};
    let mk = |v: &str| Marker {
        value: HVal::Dict(BTreeMap::from([("marker".into(), HVal::Str(v.into()))])),
        label: None,
        location: Some(Location {
            path: "t.typ".into(),
            line: 1,
            column: 1,
        }),
    };
    assert!(super::first_task_marker(&[mk("other")]).is_none());
}

pub(super) fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".mindtape")).unwrap();
    std::fs::create_dir(dir.path().join("tasks")).unwrap();
    dir
}

pub(super) fn write(root: &Path, rel: &str, body: &str) {
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
fn injects_nesting_depth() {
    let dir = setup();
    std::fs::create_dir_all(dir.path().join("tasks/backend/auth")).unwrap();
    write(dir.path(), "tasks/backend/auth/login.typ", TASK_BODY);
    write(dir.path(), "tasks/backend/util.typ", TASK_BODY);
    write(dir.path(), "tasks/top.typ", TASK_BODY);

    let outcome = load_project(dir.path(), &Config::default());

    let deep = outcome.tasks.iter().find(|t| t.id == "login").unwrap();
    assert_eq!(deep.depth, 2);
    let mid = outcome.tasks.iter().find(|t| t.id == "util").unwrap();
    assert_eq!(mid.depth, 1);
    let root = outcome.tasks.iter().find(|t| t.id == "top").unwrap();
    assert_eq!(root.depth, 0);
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
pub(super) const PRELUDE_STUB: &str = "#let task(body, ..meta) = {\n  [#metadata((marker: \"mindtape.task\", fields: (title: meta.named().at(\"title\", default: \"\")), extras: (:))) <mindtape>]\n  body\n}\n";

pub(super) fn write_pkg(root: &Path, dir_rel: &str, name: &str, version: &str, entry: &str) {
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
    // Bounded at the fixture's own top, so the walk still ascends (two levels)
    // but never reaches the machine: through the unbounded `find_root` this
    // asserted that nothing above `$TMPDIR` holds a `.mindtape`, which any
    // developer with a `~/.mindtape` project and a `$HOME`-rooted `$TMPDIR`
    // would fail. Planting `/tmp/.mindtape` reproduces it.
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a/b");
    std::fs::create_dir_all(&nested).unwrap();
    assert!(find_root_bounded(&nested, Some(dir.path())).is_none());
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

/// A task marker carrying `status` and a `depends-on` edge per entry in
/// `deps` — the shape the graph-rank test needs the loader to actually read.
fn linked_body(status: &str, deps: &[&str]) -> String {
    let links: String = deps
        .iter()
        .map(|dep| format!("(task: \"{dep}\", kind: \"depends-on\", note: none),"))
        .collect();
    format!(
        "#metadata((\n  marker: \"mindtape.task\",\n  fields: (\n    title: \"T\",\n    \
         status: (name: \"{status}\", date: none, note: none),\n    links: ({links}),\n  ),\n  \
         extras: (:),\n))\n"
    )
}

#[test]
fn stamps_graph_ranks_over_the_whole_set() {
    // The loader is the single place `topo`/`blocks` are derived (see
    // `crate::dep_rank`), so this is what wires the graph sorts and filters to
    // real files: without it every task would read 0 and `--sort topo` would be
    // a no-op.
    let dir = setup();
    write(dir.path(), "tasks/root.typ", &linked_body("wip", &[]));
    write(
        dir.path(),
        "tasks/mid.typ",
        &linked_body("wip", &["root.typ"]),
    );
    write(
        dir.path(),
        "tasks/leaf.typ",
        &linked_body("wip", &["mid.typ"]),
    );

    let outcome = load_project(dir.path(), &Config::default());
    assert_eq!(outcome.errors, Vec::new());
    let rank = |id: &str| {
        let task = outcome.tasks.iter().find(|t| t.id == id).unwrap();
        (task.topo, task.blocks)
    };
    assert_eq!(rank("root"), (0, 2));
    assert_eq!(rank("mid"), (1, 1));
    assert_eq!(rank("leaf"), (2, 0));
}

#[test]
fn a_field_without_a_line_points_at_the_marker_call() {
    let mut task = crate::test_support::bare_task("a");
    task.spans = crate::TaskCallSpans::from_lines(2, &[("links", 4)]);
    assert_eq!(task.field_location("links").line(), 4);
    assert_eq!(task.field_location("tags[1]").line(), 2);
    task.spans = crate::TaskCallSpans::default();
    assert_eq!(task.field_location("id").line(), 1);
}
