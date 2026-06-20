#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Absorb generic task-graph logic into the mindtape library",
  status: done(2026, 6, 20),
  priority: ice(impact: 8, confidence: 0.8, ease: 9),
  estimate: (hours: 3),
  depends-on: (link("convergence-boundary-adr.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Host the generic task-graph capability speconaut hand-rolls (its
`commands/deps.rs` toposort and cycle detection) in `crates/mindtape` as public
API, so speconaut deletes its copy and calls the library. Consumer-blind:
operates on the typed `Task` model and knows nothing about specs or sessions.

Scope narrowed: dependency-ready and ranked-ready already landed via
`ready-state-filter-predicates.typ`. Readiness lives in
`crates/mindtape/src/ready.rs` (one-hop, status-only, keyed by `file`), and
ranked-ready is the default `list` sort (`ready` → `priority`), so no `next`
command is needed on either side. This task now owns only the transitive,
ordering-sensitive half: toposort and cycle detection.

== Scope

- Public API in `crates/mindtape`:
  - toposort over `depends-on` with cycle detection (loud diagnostic on cycle).
- Build on the shipped `ready::Readiness` primitive — toposort consumes the same
  resolved-link / terminal-set machinery, it does not re-derive readiness.
- Reuse the existing loader and config terminal set; no new config.
- Unit tests cover cycles and topological ordering.

== Notes

Follow-up to the convergence ADR. The readiness half shipped in
`ready-state-filter-predicates.typ`; what remains is the toposort speconaut's
`commands/deps.rs` still hand-rolls.
