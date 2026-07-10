#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Export the dependency graph",
  priority: ice(
    impact: 4,
    confidence: 0.85,
    ease: 6,
  ),
  tags: (),
  status: proposed(2026, 7, 8),
)

== Summary

`mt graph` emits the task dependency graph as DOT or Mermaid. MindTape already
models typed edges (`links:` — `depends-on`, `parent`, `recurred-from`,
`supersedes`, `duplicates`, `related`, plus any project-declared custom kind)
and computes ready/blocked from the `depends-on` ones
(`crates/mindtape/src/graph.rs`); this exposes
that structure as a view. On brand with "one source, many views": a read-only
projection of the same typed source, no new data and no new store — the thing a
central-DB tracker cannot hand you as a plain diffable artifact.

== Scope

- `mt graph [filter]` emits DOT by default and `--format mermaid`. Nodes are task
  stems labelled by title and status; edges are the link relations, styled per
  relation kind.
- Honour the filter language to scope the subgraph; option to include or exclude
  terminal tasks. Highlight blocked vs ready.
- Reuse the existing graph model — do not re-implement link resolution.
- Tests: an edge per relation kind, cycle rendering, a filtered subgraph.
