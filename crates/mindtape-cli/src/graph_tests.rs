#![allow(clippy::unwrap_used)]

use mindtape::test_support::{bare_task, link_edge, noted_edge};
use mindtape::{LoadOutcome, Status, Task};

use super::{GraphArgs, build};
use crate::graph_render::GraphFormat;
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
    Project {
        root: ".".into(),
        config: mindtape::parse_config("").unwrap(),
        outcome: LoadOutcome {
            tasks,
            errors: vec![],
        },
    }
}

fn args(format: GraphFormat, filter: Option<&str>, no_terminal: bool) -> GraphArgs {
    GraphArgs {
        filter: filter.map(ToOwned::to_owned),
        format,
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
fn invalid_filter_errors() {
    let project = stub_project(vec![bare_task("a")]);
    let result = build(&args(GraphFormat::Dot, Some("!!bad!!"), false), project);
    assert!(result.is_err());
}
