//! JSON rendering for `mt graph --json`: the nodes and typed edges the DOT and
//! Mermaid renderers would draw.

use mindtape::{Edge, Readiness, Task};

use crate::cmd::graph::render::{ExternalTarget, ready_name};
use crate::cmd::schema::GRAPH;
use crate::render::envelope::Envelope;
use crate::render::objects::{EdgeJson, ExternalJson, GraphPayload, NodeJson};

/// The graph inside its `$schema` envelope: `nodes` (id, title, status and
/// readiness `state`), `edges` between them (from, to, kind, note) and
/// `external` leaves, each a link whose target is outside the backlog.
pub(super) fn render_json(
    tasks: &[Task],
    edges: &[Edge],
    external: &[ExternalTarget],
    readiness: &Readiness,
) -> Envelope<GraphPayload> {
    let nodes = tasks
        .iter()
        .map(|task| NodeJson {
            id: task.id.clone(),
            title: task.title.clone(),
            status: task.status.as_ref().map(|status| status.name.clone()),
            state: ready_name(readiness.state(task)).to_owned(),
        })
        .collect();
    let edges = edges
        .iter()
        .map(|edge| EdgeJson {
            from: edge.from.id.clone(),
            to: edge.to.id.clone(),
            kind: edge.kind.to_owned(),
            note: edge.note.map(str::to_owned),
        })
        .collect();
    let external = external
        .iter()
        .map(|entry| ExternalJson {
            from: entry.from.id.clone(),
            kind: entry.kind.to_owned(),
            target: entry.target.to_owned(),
        })
        .collect();
    Envelope::new(
        GRAPH,
        GraphPayload {
            nodes,
            edges,
            external,
        },
    )
}
