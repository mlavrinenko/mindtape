#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Export the dependency graph",
  priority: ice(
    impact: 4,
    confidence: 0.85,
    ease: 6,
  ),
  tags: (),
  status: done(2026, 7, 12),
)

== Summary

`mt graph` emits the task dependency graph as DOT or Mermaid. The tool already
models typed edges (`links:` holds `depends-on`, `parent`, `recurred-from`,
`supersedes`, `duplicates`, `related`, plus any project-declared custom kind)
and computes ready/blocked from the `depends-on` ones
(`crates/mindtape/src/graph.rs`). This exposes that structure as a view. On
brand with "one source, many views": a read-only projection of the same typed
source, with no new data and no new store. That projection is the thing a
central-DB tracker cannot hand you as a plain artifact you can diff.

== Scope

- `mt graph [filter]` emits DOT by default and `--format mermaid`. Nodes are task
  stems labelled by title and status. Edges are the link relations, styled per
  relation kind.
- Honour the filter language to scope the graph, with an option to include or
  leave out terminal tasks. Highlight blocked vs ready.
- Reuse the existing graph model: do not re-implement link resolution.
- Tests: an edge per relation kind, cycle rendering, a filtered subset.
