#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Generic artifact-gate rule engine",
  status: proposed(2026, 6, 14),
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

== Open questions

Decide before v1 — retrofitting these is painful:

- Closed-world allowlist (only these headers permitted, everything else
  rejected) vs the open-world forbid+require model. speconaut's Decision
  artifact (Context / Choice / Consequences only) smells closed-world.
- Cardinality (`action = unique`, exactly-one).
- Ordering (Summary must precede Scope).

Follow-up to `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.
