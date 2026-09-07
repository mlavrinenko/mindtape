#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "CLI writer for task links",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("cli", "tests", "docs"),
  estimate: (files: 8),
  assignee: "agent",
  status: done(
    2026,
    9,
    7,
  )[mt link ships: single-edge verb, --apply adjacency map, merge/replace, pre-write checks],
)

== Summary

links: is the one field with no CLI writer, so cross-linking many tasks means hand-editing files - which is how the datarp.wiki link-tasks.py regex one-shot happened. Add `mt link`: a bulk --apply reading a YAML adjacency map merged idempotently into each task's links:, plus the single-edge verb it makes nearly free. Reuses the existing edge-writing plumbing (task_edit upsert, recur's edge append, rm's batch plan/report).


== Scope

- `mt link <stem> <kind> <target...>`: upsert one edge into links:, merging with existing edges and deduping on (kind, target) so re-runs are no-ops. Kind must be a configured link kind; related without a note is refused at parse, naming the edge.
- `mt link --apply <file>` (also `-` for stdin): read a YAML/JSON adjacency map, stem to kind to targets. Merge by default, --replace to make the file the complete edge set. Per-file failures reported, the rest still land (rm-style plan/report); --dry-run prints the plan without writing.
- Bulk shape: seed-content-from-telegram-export: { depends-on: \[article-content-type, error-record-content-type\] }, with related carrying a note field per target.
- Serialize edges through the prelude's typed helpers with the same markup escaping text slots get, so written files parse and mt check stays green.
- Docs: SKILL.md gains the command, the www site documents the format. Tests cover merge/dedupe/replace/notes/unknown-kind/missing-target per command path.

