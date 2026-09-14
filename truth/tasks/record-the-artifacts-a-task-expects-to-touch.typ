#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Record the artifacts a task expects to touch",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

Naming the files is the most-cited field in agent-ticket practice — Anthropics own guidance is to name the files and interfaces involved — and blustrokes Files section proves the content is data rather than prose: median 8 words, a bare path list. Its value is directing the implementer, not validating the spec, so checking that a path exists is the wrong check. A spec legitimately names a file it will create, and a closed task legitimately names one a later refactor deleted.

== Scope

Add a field taking edges in the prelude's packager idiom:

```typ
links: created("a/b.rs") + changed("z/*.rs"),
```

Globs allowed on `changed`, refused on `created`, and never validate the paths at rest. Feed it to estimate_drift.rs instead, which already diffs a declared file count against the working tree and only ever runs on a wip task, so the comparison always sees a live tree and the dead-path problem cannot arise.

Split: this task is the field itself. Touches the packagers in `lib/prelude.typ`, `BUILTIN_KINDS` and the target validator in `crates/mindtape/src/link.rs` with `link_tests.rs`, and the glob refusal in `crates/mindtape/src/validate_links.rs` with `validate_links_tests.rs`. Feeding the declared paths to the drift check follows in `compare-declared-artifacts-against-the-working-tree.typ`, the docs in `document-the-created-and-changed-edge-kinds.typ`.

== Dogfood

This backlog declares `created`/`changed` edges from `compare-declared-artifacts-against-the-working-tree.typ` on.
