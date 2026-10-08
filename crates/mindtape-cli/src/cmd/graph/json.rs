//! JSON rendering for `mt graph --json`: the nodes and typed edges the DOT and
//! Mermaid renderers would draw.

use mindtape::{Edge, Readiness, Task};
use serde_json::{Value, json};

use crate::cmd::graph::render::{ExternalTarget, ready_name};

/// The graph as a `schema_version` envelope: `nodes` (id, title, status and
/// readiness `state`), `edges` between them (from, to, kind, note) and
/// `external` leaves, each a link whose target is outside the backlog.
pub(super) fn render_json(
    tasks: &[Task],
    edges: &[Edge],
    external: &[ExternalTarget],
    readiness: &Readiness,
) -> Value {
    let nodes: Vec<Value> = tasks
        .iter()
        .map(|task| {
            json!({
                "id": task.id,
                "title": task.title,
                "status": task.status.as_ref().map(|status| status.name.as_str()),
                "state": ready_name(readiness.state(task)),
            })
        })
        .collect();
    let edges: Vec<Value> = edges
        .iter()
        .map(|edge| {
            json!({
                "from": edge.from.id,
                "to": edge.to.id,
                "kind": edge.kind,
                "note": edge.note,
            })
        })
        .collect();
    let external: Vec<Value> = external
        .iter()
        .map(|entry| json!({"from": entry.from.id, "kind": entry.kind, "target": entry.target}))
        .collect();
    json!({
        "schema_version": mindtape::SCHEMA_VERSION,
        "nodes": nodes,
        "edges": edges,
        "external": external,
    })
}
