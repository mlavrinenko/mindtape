#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Generic artifact-gate rule engine",
  status: wip(2026, 6, 20)[
    Core structural engine extracted as the `artifact-gate` crate; `check` rides
    it. The fact/`when` seam is externalized to consumers and the open questions
    are deferred — see Landed (v1).
  ],
  priority: ice(impact: 8, confidence: 0.8, ease: 9),
  estimate: (hours: 10),
  depends-on: (link("swh:1:rev:8069f3182b74520070fcfeb584a5626df6db1203;path=truth/tasks/typst-cst-structure-extraction.typ"),),
  relates-to: (
    link("extract-typst-harvest.typ"),
    link("converge-speconaut.typ"),
  ),
  tags: ("substrate", "gating"),
)

== Summary

A generic, plane-agnostic structural rule engine, reusable by any consumer
(speconaut included), not bound to MindTape's task model. The engine is a pure
function over two abstract inputs and emits diagnostics; it knows nothing about
Typst or task concepts.

- Structure view: a tree of nodes (kind, text, span, children), produced by
  `typst-cst` for Typst, but pluggable for anything else.
- Fact provider: named values, produced by `typst-harvest` for computed fields,
  or derived statically when possible.
- Diagnostics: rule id, severity, file:line, message.

== Scope

- Static, eval, or both. The engine is pure over (structure, facts); the plane
  is an input concern. Facts may come from a cheap static derivation or from
  eval via `typst-harvest`. Each rule declares which inputs it needs, so the
  engine reports a ruleset's requirements and a consumer can run static-only
  (editor, pre-commit) when no rule touches eval facts.
- Rule model: adopt ast-grep's vocabulary, not its engine. A rule is
  kind + relation (direct child / descendant) + match (exact | regex) +
  action (forbid | require) + optional `when` (filter expression over facts) +
  message/severity. Headings are one node kind; the model generalises to slots
  and body shape later.
- `when` reuses the existing evalexpr filter language (`has_tag`, `status`,
  field comparisons) unchanged.
- Two entry points: config-driven (YAML/TOML rule files) and a library API
  (`run_rules(structure, facts, rules) -> diagnostics`).
- No MindTape/speconaut domain types in the core crate.

== Landed (v1)

The reusable core shipped as `crates/artifact-gate`, consumer-blind (no Typst,
task model, or domain concepts — deps are `regex` + `thiserror` only):

- `Node { kind, text, line, children }` is the abstract structure tree; `Matcher`
  (exact | regex, direct-child | recursive), `Action` (require | forbid),
  `Severity`, `Rule`, `Diagnostic`, and `run_rules(roots, rules) -> diagnostics`
  are the public surface. The path-matching that lived in MindTape's `gate.rs` now
  lives here; `gate.rs` adapts the `typst-cst` heading tree into `Node`s and maps
  `Diagnostic` back to its `Violation`. `mindtape check` behaviour and public API
  are unchanged.

Decisions baked in (flag on review):

- Fact/`when` seam externalized, not engine-owned. The engine is pure over
  (structure, rules) and knows nothing about facts. Each consumer keeps its own
  fact model and filter language and pre-selects which rules apply: MindTape
  filters by `when` through its existing `Filter` over `Task` (unchanged, so no
  loss of `priority` / `stale` / framework predicates), then calls `run_rules`.
  This is cleaner than baking a generic fact-filter into the engine and is the
  convergence boundary — shared structure-matching, per-consumer facts. A
  fact-provider entry point for a standalone config-driven engine is the planned
  extension, not a v1 need.
- Open-world require/forbid only for v1. `kind` is carried on `Node` but not yet a
  match dimension; cardinality (`unique`), ordering, and the closed-world
  allowlist are deferred as follow-ups — land them when a consumer (speconaut's
  Decision artifact) actually needs them, rather than guess the shape now.

== Open questions

Decide before v1 — retrofitting these is painful:

- Closed-world allowlist (only these headers permitted, everything else
  rejected) vs the open-world forbid+require model. speconaut's Decision
  artifact (Context / Choice / Consequences only) smells closed-world.
- Cardinality (`action = unique`, exactly-one).
- Ordering (Summary must precede Scope).

Follow-up to `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.
