#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Ready MindTape to host speconaut's task layer",
  status: proposed(2026, 6, 20),
  priority: ice(impact: 8, confidence: 0.8, ease: 8),
  relates-to: (
    link("foreign-worktree-package-resolution.typ"),
    link("consumer-blind-root-markers.typ"),
  ),
  tags: ("speconaut", "substrate"),
)

== Summary

This is the MindTape-side half of the convergence: make `crates/mindtape` a
correct task library for a foreign tool to depend on. speconaut deletes its own
task model, DSL, and graph code and depends on `crates/mindtape`; the
speconaut-side migration is tracked in the speconaut repo (its
`converge-task-tracking-with-mindtape` umbrella and three sub-tasks). This task
holds MindTape parts only and never references speconaut internals.

The boundary contract is settled in
`docs/decisions/005-task-layer-convergence-boundary.md`: MindTape owns the task
model, the `@local/mindtape` DSL, config-driven validation, `list` / `add` /
`cleanup`, the dependency graph and ranking; speconaut owns specs↔code and the
spec-first session FSM and depends on the library. The core-vs-extras field split
is fixed there too: `supersedes`, `duplicates`, `reviewed` are MindTape core (all
three already on `model::Task`); DDD contexts, decisions, and spec↔code links
stay `x-speconaut`.

Most of the MindTape contract has already landed: the three core-field
promotions, `graph::toposort` with cycle detection, `ready::Readiness`, the
`x-<ns>` extension seam, and config-driven `validate`. What remains before
speconaut hardens the dependency are two correctness/hygiene fixes — harder to
make once a second tool pins the crate, so they go first.

== Scope

- Land the two fixes this task relates to:
  - `foreign-worktree-package-resolution.typ` (blocking): `mindtape ls`/`check`
    must resolve `@local/mindtape` and harvest tasks correctly when run inside a
    foreign worktree (one whose `<root>/lib` is some other Typst package). Today
    the in-tree lib override hijacks the `mindtape` package name, so a speconaut
    checkout would surface zero tasks — defeating the shared-checkout property
    ADR 005 promises.
  - `consumer-blind-root-markers.typ` (hygiene): drop the hardcoded `.mindtape/`
    marker from `typst-harvest`, which speconaut already path-depends on. A
    consumer-blind substrate must not name a consumer.
- Shared-checkout contract: an `x-<ns>` slot may hold any expression — a plain
  dict or a call like `speconaut.task-extras(..)` — as long as it evaluates to a
  value; MindTape harvests it opaquely as an `HVal`. So a task file is free to
  `#import "@local/speconaut"` for an extras helper. The only requirement is that
  MindTape's World can resolve whatever `@local/*` packages the file imports.
  MindTape already resolves `@local/*` from the standard local-package store;
  `foreign-worktree-package-resolution.typ` adds config-based whitelisting so a
  worktree can resolve a foreign package from an in-tree path without a global
  install. No restriction on the extras payload shape.

== Notes

- No marker-shape or `lib/typst.toml` version bump: the core promotions
  (`supersedes` / `duplicates` / `reviewed`) already shipped in `0.2.0`.
- `extract-typst-harvest` (lift the harvester to its own repo) stays postponed;
  it is not a convergence blocker. speconaut consumes it as a path dep today.
- Replaces the stale dependencies on the now-done `convergence-boundary-adr`
  (→ ADR 005) and `absorb-task-graph-into-lib` (→ `graph::toposort`).
