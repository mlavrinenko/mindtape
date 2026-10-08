//! DOT and Mermaid rendering for `mt graph`.
//!
//! Pure string builders over a resolved node set and its induced [`Edge`]s.
//! Nodes carry a readiness fill (ready/blocked/terminal); edges are styled per
//! relation kind so a rendered graph reads the typed structure at a glance.
//!
//! A link kind whose `[[tasks.links.kinds]] validate` is not `local-task-path`
//! points at an artifact outside the backlog (a commit, a pull request, a
//! build). Those arrive separately as [`ExternalTarget`]s and render as their
//! own distinctly-styled leaf nodes: never a task node, never traversed, and
//! never part of the readiness or ordering the task nodes carry.

use std::collections::HashMap;
use std::fmt::Write as _;

use mindtape::{Edge, Readiness, ReadyState, Task};

/// One `links:` edge pointing outside the backlog: the linking task, the edge
/// kind, and the raw target as authored (a URL, a git SHA, a UUID …).
#[derive(Debug, Clone, Copy)]
pub struct ExternalTarget<'a> {
    /// The linking task (edge tail).
    pub from: &'a Task,
    /// The edge kind, e.g. a project's `discharged-by`.
    pub kind: &'a str,
    /// The raw target string. Never resolved against the task set.
    pub target: &'a str,
}

/// The distinct external targets in `external`, in first-seen order — one node
/// per artifact, however many tasks point at it.
fn distinct_targets<'a>(external: &[ExternalTarget<'a>]) -> Vec<&'a str> {
    let mut seen: Vec<&str> = Vec::new();
    for entry in external {
        if !seen.contains(&entry.target) {
            seen.push(entry.target);
        }
    }
    seen
}

/// Output syntax for `mt graph`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum GraphFormat {
    /// Graphviz DOT.
    #[default]
    Dot,
    /// A Mermaid flowchart.
    Mermaid,
}

/// Node-id prefix for an external target, keeping it in a namespace no task id
/// can collide with.
const EXTERNAL_PREFIX: &str = "external:";

/// Fill colour for an external-target node — off the readiness palette
/// entirely, since an artifact outside the backlog has no readiness.
const EXTERNAL_FILL: &str = "#fdf3d0";

/// Stroke colour for an external-target node and its edge.
const EXTERNAL_STROKE: &str = "#b9770e";

/// Fill colour for a node in each readiness state.
fn fill(state: ReadyState) -> &'static str {
    match state {
        ReadyState::Ready => "#d5f5e3",
        ReadyState::Blocked => "#fadbd8",
        ReadyState::Settled => "#eaeded",
    }
}

/// Mermaid `classDef` name for each readiness state, also the `state` of a
/// node in `--json`.
pub(super) fn ready_name(state: ReadyState) -> &'static str {
    match state {
        ReadyState::Ready => "ready",
        ReadyState::Blocked => "blocked",
        ReadyState::Settled => "settled",
    }
}

/// DOT `(line-style, colour)` for an edge kind. Built-in kinds each get a
/// distinct style; an unknown/custom kind falls back to a neutral solid line.
fn dot_edge_style(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "depends-on" => ("solid", "#c0392b"),
        "parent" => ("bold", "#2471a3"),
        "recurred-from" => ("dotted", "#7d3c98"),
        "supersedes" => ("dashed", "#b9770e"),
        "duplicates" => ("dashed", "#7f8c8d"),
        "related" => ("dashed", "#95a5a6"),
        _ => ("solid", "#34495e"),
    }
}

/// Mermaid arrow syntax for an edge kind: thick for `parent`, dotted for the
/// non-blocking cross-references, plain for `depends-on` and custom kinds.
fn mermaid_arrow(kind: &str) -> &'static str {
    match kind {
        "parent" => "==>",
        "recurred-from" | "supersedes" | "duplicates" | "related" => "-.->",
        _ => "-->",
    }
}

/// Node label: `Title [status]`, falling back to the id when the title is
/// empty and to `—` when the task has no status.
fn node_label(task: &Task) -> String {
    let title = if task.title.is_empty() {
        &task.id
    } else {
        &task.title
    };
    let status = task.status.as_ref().map_or("\u{2014}", |s| s.name.as_str());
    format!("{title} [{status}]")
}

/// Escape a string for a DOT double-quoted id/label: backslash and quote get
/// escaped; embedded newlines collapse to spaces so a label stays one line.
fn dot_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}

/// Escape a string for a Mermaid quoted label: quotes become the `#quot;`
/// entity Mermaid understands; newlines collapse to spaces.
fn mermaid_escape(s: &str) -> String {
    s.replace('"', "#quot;").replace('\n', " ")
}

/// Render the node set, its edges, and its external targets as a Graphviz DOT
/// digraph.
#[must_use]
pub fn render_dot(
    tasks: &[Task],
    edges: &[Edge],
    external: &[ExternalTarget],
    readiness: &Readiness,
) -> String {
    let mut out = String::from("digraph mindtape {\n");
    out.push_str("  rankdir=LR;\n");
    out.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"sans-serif\"];\n");
    for task in tasks {
        let _ = writeln!(
            out,
            "  \"{id}\" [label=\"{label}\", fillcolor=\"{fill}\"];",
            id = dot_escape(&task.id),
            label = dot_escape(&node_label(task)),
            fill = fill(readiness.state(task)),
        );
    }
    for target in distinct_targets(external) {
        let _ = writeln!(
            out,
            "  \"{EXTERNAL_PREFIX}{id}\" [label=\"{id}\", shape=note, \
             style=\"filled,dashed\", fillcolor=\"{EXTERNAL_FILL}\"];",
            id = dot_escape(target),
        );
    }
    for edge in edges {
        let (style, color) = dot_edge_style(edge.kind);
        let _ = writeln!(
            out,
            "  \"{from}\" -> \"{to}\" [label=\"{kind}\", style={style}, color=\"{color}\"];",
            from = dot_escape(&edge.from.id),
            to = dot_escape(&edge.to.id),
            kind = dot_escape(edge.kind),
        );
    }
    for entry in external {
        let _ = writeln!(
            out,
            "  \"{from}\" -> \"{EXTERNAL_PREFIX}{to}\" [label=\"{kind}\", style=dotted, \
             color=\"{EXTERNAL_STROKE}\"];",
            from = dot_escape(&entry.from.id),
            to = dot_escape(entry.target),
            kind = dot_escape(entry.kind),
        );
    }
    out.push_str("}\n");
    out
}

/// Render the node set, its edges, and its external targets as a Mermaid
/// flowchart. Task ids map to stable `nK` aliases so any stem is a legal
/// Mermaid node id; the real stem/title lives in the label. External targets
/// take their own `xK` aliases and a stadium shape, so they never read as tasks.
#[must_use]
pub fn render_mermaid(
    tasks: &[Task],
    edges: &[Edge],
    external: &[ExternalTarget],
    readiness: &Readiness,
) -> String {
    // Stable `nK` alias per task id: the index into the node set. Edges look
    // theirs up rather than index the map, so a stray unknown id is skipped, not
    // a panic (clippy's indexing-slicing).
    let alias: HashMap<&str, usize> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.as_str(), i))
        .collect();
    let targets = distinct_targets(external);
    let ext_alias: HashMap<&str, usize> = targets
        .iter()
        .enumerate()
        .map(|(i, target)| (*target, i))
        .collect();

    let mut out = String::from("graph LR\n");
    for (i, task) in tasks.iter().enumerate() {
        let _ = writeln!(
            out,
            "  n{i}[\"{label}\"]:::{cls}",
            label = mermaid_escape(&node_label(task)),
            cls = ready_name(readiness.state(task)),
        );
    }
    for (i, target) in targets.iter().enumerate() {
        let _ = writeln!(
            out,
            "  x{i}([\"{label}\"]):::external",
            label = mermaid_escape(target),
        );
    }
    for edge in edges {
        let (Some(&from), Some(&to)) = (
            alias.get(edge.from.id.as_str()),
            alias.get(edge.to.id.as_str()),
        ) else {
            continue;
        };
        let _ = writeln!(
            out,
            "  n{from} {arrow}|{kind}| n{to}",
            arrow = mermaid_arrow(edge.kind),
            kind = mermaid_escape(edge.kind),
        );
    }
    for entry in external {
        let (Some(&from), Some(&to)) = (
            alias.get(entry.from.id.as_str()),
            ext_alias.get(entry.target),
        ) else {
            continue;
        };
        let _ = writeln!(
            out,
            "  n{from} -.->|{kind}| x{to}",
            kind = mermaid_escape(entry.kind),
        );
    }
    out.push_str("  classDef ready fill:#d5f5e3,stroke:#27ae60;\n");
    out.push_str("  classDef blocked fill:#fadbd8,stroke:#c0392b;\n");
    out.push_str("  classDef settled fill:#eaeded,stroke:#7f8c8d;\n");
    let _ = writeln!(
        out,
        "  classDef external fill:{EXTERNAL_FILL},stroke:{EXTERNAL_STROKE},stroke-dasharray: 4 2;"
    );
    out
}
