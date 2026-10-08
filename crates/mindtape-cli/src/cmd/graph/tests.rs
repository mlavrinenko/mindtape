#![allow(clippy::unwrap_used)]

use mindtape::test_support::{TASK_SRC, bare_task, link_edge, noted_edge};
use mindtape::{LoadOutcome, Status, Task};

use super::{GraphArgs, build};
use crate::cmd::graph::render::GraphFormat;
use crate::util::Project;

fn with_status(mut task: Task, name: &str) -> Task {
    task.status = Some(Status {
        name: name.to_owned(),
        date: None,
        note: None,
    });
    task
}

fn stub_project(tasks: Vec<Task>) -> Project {
    // `parse_config("")` supplies the embedded legend floor, so `done` counts as
    // terminal (a bare `Config::default()` has an empty legend).
    project_with(tasks, "")
}

/// A stub project whose config is `config_toml` layered over the floor.
fn project_with(tasks: Vec<Task>, config_toml: &str) -> Project {
    Project {
        root: ".".into(),
        config: mindtape::parse_config(config_toml).unwrap(),
        outcome: LoadOutcome {
            tasks,
            errors: vec![],
            ..LoadOutcome::default()
        },
    }
}

/// A project declaring one kind that points outside the backlog, validated by
/// `validate` (`url`, `any`, …).
fn external_project(tasks: Vec<Task>, validate: &str) -> Project {
    project_with(
        tasks,
        &format!("[[tasks.links.kinds]]\nname = \"discharged-by\"\nvalidate = \"{validate}\"\n"),
    )
}

fn args(format: GraphFormat, filter: Option<&str>, no_terminal: bool) -> GraphArgs {
    GraphArgs {
        filter: filter.map(ToOwned::to_owned),
        format,
        json: false,
        no_terminal,
    }
}

#[test]
fn dot_renders_an_edge_per_relation_kind() {
    let mut a = bare_task("a");
    a.title = "Alpha".to_owned();
    a.links = vec![
        link_edge("b.typ", "depends-on"),
        link_edge("c.typ", "parent"),
        noted_edge("d.typ", "related", "see also"),
    ];
    let tasks = vec![
        with_status(a, "wip"),
        with_status(bare_task("b"), "done"),
        bare_task("c"),
        bare_task("d"),
    ];
    let out = build(&args(GraphFormat::Dot, None, false), stub_project(tasks)).unwrap();
    assert!(out.contains("digraph mindtape"));
    assert!(out.contains("\"a\" -> \"b\" [label=\"depends-on\", style=solid"));
    assert!(
        out.contains("#c0392b"),
        "depends-on edge must use red colour"
    );
    assert!(out.contains("\"a\" -> \"c\" [label=\"parent\", style=bold"));
    assert!(out.contains("\"a\" -> \"d\" [label=\"related\", style=dashed"));
    assert!(out.contains("label=\"Alpha [wip]\""));
}

#[test]
fn dot_renders_a_cycle() {
    // a -> b -> a: rendering is a plain projection, so a dependency cycle draws
    // both edges (no toposort, nothing to reject).
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "depends-on")];
    let mut b = bare_task("b");
    b.links = vec![link_edge("a.typ", "depends-on")];
    let out = build(
        &args(GraphFormat::Dot, None, false),
        stub_project(vec![a, b]),
    )
    .unwrap();
    assert!(out.contains("\"a\" -> \"b\""));
    assert!(out.contains("\"b\" -> \"a\""));
}

#[test]
fn filter_scopes_to_the_induced_subgraph() {
    // `!terminal` drops the done task b; a's edge to it is not induced.
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "depends-on")];
    let tasks = vec![with_status(a, "wip"), with_status(bare_task("b"), "done")];
    let out = build(
        &args(GraphFormat::Dot, Some("!terminal"), false),
        stub_project(tasks),
    )
    .unwrap();
    assert!(out.contains("\"a\""));
    assert!(!out.contains("\"b\""));
    assert!(!out.contains("->"));
}

#[test]
fn no_terminal_prunes_settled_tasks() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "depends-on")];
    let tasks = vec![with_status(a, "wip"), with_status(bare_task("b"), "done")];
    let out = build(&args(GraphFormat::Dot, None, true), stub_project(tasks)).unwrap();
    assert!(out.contains("\"a\""));
    assert!(!out.contains("\"b\" [label"));
}

#[test]
fn dot_styles_blocked_and_ready_nodes() {
    // a depends on still-open b: a is blocked, b is ready.
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "depends-on")];
    let tasks = vec![with_status(a, "wip"), with_status(bare_task("b"), "wip")];
    let out = build(&args(GraphFormat::Dot, None, false), stub_project(tasks)).unwrap();
    assert!(out.contains("\"a\" [label=\"a [wip]\", fillcolor=\"#fadbd8\"]"));
    assert!(out.contains("\"b\" [label=\"b [wip]\", fillcolor=\"#d5f5e3\"]"));
}

#[test]
fn mermaid_renders_nodes_edges_and_classes() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "depends-on")];
    let tasks = vec![with_status(a, "wip"), with_status(bare_task("b"), "done")];
    let out = build(
        &args(GraphFormat::Mermaid, None, false),
        stub_project(tasks),
    )
    .unwrap();
    assert!(out.starts_with("graph LR"));
    assert!(out.contains("-->|depends-on|"));
    assert!(out.contains(":::ready"));
    assert!(out.contains(":::settled"));
    assert!(out.contains("classDef ready"));
}

#[test]
fn dot_renders_recurred_from_edge() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "recurred-from")];
    let out = build(
        &args(GraphFormat::Dot, None, false),
        stub_project(vec![
            with_status(a, "wip"),
            with_status(bare_task("b"), "done"),
        ]),
    )
    .unwrap();
    assert!(out.contains("recurred-from"));
    assert!(out.contains("dotted"));
    assert!(out.contains("#7d3c98"));
}

#[test]
fn dot_renders_supersedes_edge() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "supersedes")];
    let out = build(
        &args(GraphFormat::Dot, None, false),
        stub_project(vec![
            with_status(a, "wip"),
            with_status(bare_task("b"), "done"),
        ]),
    )
    .unwrap();
    assert!(out.contains("supersedes"));
    assert!(out.contains("dashed"));
    assert!(out.contains("#b9770e"));
}

#[test]
fn dot_renders_duplicates_edge() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "duplicates")];
    let out = build(
        &args(GraphFormat::Dot, None, false),
        stub_project(vec![
            with_status(a, "wip"),
            with_status(bare_task("b"), "done"),
        ]),
    )
    .unwrap();
    assert!(out.contains("duplicates"));
    assert!(out.contains("dashed"));
    assert!(out.contains("#7f8c8d"));
}

#[test]
fn mermaid_renders_recurred_edge() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "recurred-from")];
    let out = build(
        &args(GraphFormat::Mermaid, None, false),
        stub_project(vec![
            with_status(a, "wip"),
            with_status(bare_task("b"), "done"),
        ]),
    )
    .unwrap();
    assert!(out.contains("-.->"));
}

#[test]
fn mermaid_renders_parent_edge() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "parent")];
    let out = build(
        &args(GraphFormat::Mermaid, None, false),
        stub_project(vec![
            with_status(a, "wip"),
            with_status(bare_task("b"), "done"),
        ]),
    )
    .unwrap();
    assert!(out.contains("==>"));
}

#[test]
fn mermaid_renders_related_edge() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "related")];
    let out = build(
        &args(GraphFormat::Mermaid, None, false),
        stub_project(vec![
            with_status(a, "wip"),
            with_status(bare_task("b"), "done"),
        ]),
    )
    .unwrap();
    assert!(out.contains("-.->"));
}

/// A target outside the backlog renders as its own distinctly-styled leaf —
/// never a task node, and never on the readiness palette.
#[test]
fn dot_renders_an_external_target_as_a_distinct_node() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("https://example.com/pr/7", "discharged-by")];
    let out = build(
        &args(GraphFormat::Dot, None, false),
        external_project(vec![with_status(a, "wip")], "url"),
    )
    .unwrap();
    assert!(
        out.contains(
            "\"external:https://example.com/pr/7\" [label=\"https://example.com/pr/7\", shape=note"
        ),
        "external node must render distinctly: {out}"
    );
    assert!(out.contains("#fdf3d0"), "external fill: {out}");
    assert!(
        out.contains("\"a\" -> \"external:https://example.com/pr/7\" [label=\"discharged-by\""),
        "{out}"
    );
}

/// The exclusion follows the `validate` axis, not the target string: a target
/// that happens to spell a loaded task's stem is still never traversed.
#[test]
fn external_kind_is_never_traversed_even_when_it_spells_a_stem() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("b.typ", "discharged-by")];
    let tasks = vec![with_status(a, "wip"), with_status(bare_task("b"), "wip")];
    let out = build(
        &args(GraphFormat::Dot, None, false),
        external_project(tasks, "any"),
    )
    .unwrap();
    assert!(
        !out.contains("\"a\" -> \"b\""),
        "an external edge is not a task-graph edge: {out}"
    );
    assert!(out.contains("\"a\" -> \"external:b.typ\""), "{out}");
    // `a` has no task-graph dependency, so it reads ready, not blocked.
    assert!(
        out.contains("\"a\" [label=\"a [wip]\", fillcolor=\"#d5f5e3\"]"),
        "{out}"
    );
}

#[test]
fn mermaid_renders_external_targets_in_their_own_class() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("https://example.com/pr/7", "discharged-by")];
    let out = build(
        &args(GraphFormat::Mermaid, None, false),
        external_project(vec![with_status(a, "wip")], "url"),
    )
    .unwrap();
    assert!(
        out.contains("x0([\"https://example.com/pr/7\"]):::external"),
        "{out}"
    );
    assert!(out.contains("n0 -.->|discharged-by| x0"), "{out}");
    assert!(out.contains("classDef external"), "{out}");
}

/// Two tasks pointing at the same artifact share one node.
#[test]
fn repeated_external_target_renders_once() {
    let mut a = bare_task("a");
    a.links = vec![link_edge("https://example.com/pr/7", "discharged-by")];
    let mut b = bare_task("b");
    b.links = vec![link_edge("https://example.com/pr/7", "discharged-by")];
    let out = build(
        &args(GraphFormat::Dot, None, false),
        external_project(vec![with_status(a, "wip"), with_status(b, "wip")], "url"),
    )
    .unwrap();
    assert_eq!(
        out.matches("[label=\"https://example.com/pr/7\", shape=note")
            .count(),
        1,
        "{out}"
    );
}

#[test]
fn invalid_filter_errors() {
    let project = stub_project(vec![bare_task("a")]);
    let result = build(&args(GraphFormat::Dot, Some("!!bad!!"), false), project);
    assert!(result.is_err());
}

/// `run` must load a project from disk and print graph output. A body replaced
/// with `Ok(())` would exit 0 without output, caught by the stdout assertion.
#[test]
fn run_prints_graph_from_disk() {
    if std::env::var("_MT_GRAPH_RUN").is_err() {
        let dir = tempfile::tempdir().unwrap();
        // Scaffold project manually (must exist before run() calls open()).
        mindtape::test_support::write_bare_project(dir.path());
        std::fs::write(dir.path().join("tasks/a.typ"), TASK_SRC).unwrap();
        // Need lib override for HarvestWorld evaluation.
        std::fs::create_dir_all(dir.path().join("lib")).unwrap();
        std::fs::write(
            dir.path().join("lib/typst.toml"),
            "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("lib/prelude.typ"),
            mindtape::test_support::PRELUDE,
        )
        .unwrap();

        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .current_dir(dir.path())
            .env("_MT_GRAPH_RUN", "1")
            .args(["run_prints_graph_from_disk", "--nocapture"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("digraph mindtape"),
            "run() should print graph output"
        );
        return;
    }
    // Child: call run() from cwd (the temp project).
    let args = GraphArgs {
        filter: None,
        format: GraphFormat::Dot,
        json: false,
        no_terminal: false,
    };
    assert!(super::run(&args).is_ok());
}
