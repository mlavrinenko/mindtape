#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Absorb generic task-graph logic into the mindtape library",
  status: proposed(2026, 6, 18),
  priority: 5,
  estimate: (hours: 6),
  depends-on: (link("convergence-boundary-adr.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Host the generic task-graph capability speconaut hand-rolls (its
`commands/deps.rs` toposort and cycle detection, `next` ranking, dependency-ready
computation) in `crates/mindtape` as public API, so speconaut deletes its copy
and calls the library. Consumer-blind: operates on the typed `Task` model and
knows nothing about specs or sessions.

== Scope

- Public API in `crates/mindtape`:
  - toposort over `depends-on` with cycle detection (loud diagnostic on cycle).
  - dependency-ready: a task whose every `depends-on` target is terminal (per the
    project's configured terminal status set).
  - ranking / `next`: ready tasks ordered by priority then a stable tiebreak.
- Reuse the existing loader and config terminal set; no new config.
- Unit tests cover cycles, partial-terminal deps, priority ordering.

== Notes

Pairs with `ready-state-filter-predicates.typ` if that lands — the filter
predicate calls the same readiness function. Enables the speconaut-side `next`
and session re-anchor. Follow-up to the convergence ADR.
