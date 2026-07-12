//! DOT and Mermaid rendering for `mt graph`.
//!
//! Pure string builders over a resolved node set and its induced [`Edge`]s.
//! Nodes carry a readiness fill (ready/blocked/terminal); edges are styled per
//! relation kind so a rendered graph reads the typed structure at a glance.

use std::collections::HashMap;
use std::fmt::Write as _;

use mindtape::{Edge, Readiness, ReadyState, Task};

/// Output syntax for `mt graph`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum GraphFormat {
    /// Graphviz DOT (the default).
    #[default]
    Dot,
    /// Mermaid flowchart.
    Mermaid,
}

/// Fill colour for a node in each readiness state.
fn fill(state: ReadyState) -> &'static str {
    match state {
        ReadyState::Ready => "#d5f5e3",
        ReadyState::Blocked => "#fadbd8",
        ReadyState::Settled => "#eaeded",
    }
}

/// Mermaid `classDef` name for each readiness state.
fn class_name(state: ReadyState) -> &'static str {
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

/// Render the node set and its edges as a Graphviz DOT digraph.
#[must_use]
pub fn render_dot(tasks: &[Task], edges: &[Edge], readiness: &Readiness) -> String {
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
    out.push_str("}\n");
    out
}

/// Render the node set and its edges as a Mermaid flowchart. Task ids map to
/// stable `nK` aliases so any stem is a legal Mermaid node id; the real
/// stem/title lives in the label.
#[must_use]
pub fn render_mermaid(tasks: &[Task], edges: &[Edge], readiness: &Readiness) -> String {
    // Stable `nK` alias per task id: the index into the node set. Edges look
    // theirs up rather than index the map, so a stray unknown id is skipped, not
    // a panic (clippy's indexing-slicing).
    let alias: HashMap<&str, usize> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.as_str(), i))
        .collect();

    let mut out = String::from("graph LR\n");
    for (i, task) in tasks.iter().enumerate() {
        let _ = writeln!(
            out,
            "  n{i}[\"{label}\"]:::{cls}",
            label = mermaid_escape(&node_label(task)),
            cls = class_name(readiness.state(task)),
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
    out.push_str("  classDef ready fill:#d5f5e3,stroke:#27ae60;\n");
    out.push_str("  classDef blocked fill:#fadbd8,stroke:#c0392b;\n");
    out.push_str("  classDef settled fill:#eaeded,stroke:#7f8c8d;\n");
    out
}
