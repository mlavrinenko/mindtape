#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Generic artifact-gate rule engine",
  status: done(2026, 6, 20)[
    Core structural engine shipped as the `artifact-gate` crate; `check` rides
    it. v1 (require/forbid path rules) plus the three deferred capabilities —
    `unique` cardinality, document ordering, and closed-world allowlists — all
    landed and dogfooded. The fact/`when` seam stays externalized to consumers
    (engine pure over structure + rules). See Landed.
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

== Landed

The reusable core shipped as `crates/artifact-gate`, consumer-blind (no Typst,
task model, or domain concepts — deps are `regex` + `thiserror` only):

- `Node { kind, text, line, children }` is the abstract structure tree; `Matcher`
  (exact | regex, direct-child | recursive), `Severity`, `Diagnostic`, and
  `run_rules(roots, rules) -> diagnostics` are the public surface. A `Rule` is
  diagnostic metadata (`id` / `severity` / `message`) plus a `Check`:
  - `Check::Path { path, action }` — `Action` is `Require` | `Forbid` | `Unique`.
  - `Check::Order { first, second }` — every `first` match must precede every
    `second` match by line; order-only (a missing side is not a violation).
  - `Check::Allow { scope, allowed }` — closed-world: under `scope`'s children
    (or the roots when `None`), only `allowed` matchers are permitted.
  Constructors `Rule::path` / `Rule::order` / `Rule::allow`; builders
  `with_id` / `with_message` / `with_severity`. The path-matching that lived in
  MindTape's `gate.rs` now lives here; `gate.rs` adapts the `typst-cst` heading
  tree into `Node`s and maps `Diagnostic` back to its `Violation`. `mindtape
  check`'s public API is unchanged.

MindTape dogfoods all three shapes through `[[tasks.body.headers.rules]]`
(`action = require|forbid|unique|order|allowlist`, each action using its own
fields — `path` for cardinality, `before`+`after` for order, `scope`+`allowed`
for allowlist; mismatched fields fail loud at config load). The backlog runs a
`unique Summary` and an `order Summary before Scope` rule.

Decisions baked in:

- Fact/`when` seam externalized, not engine-owned. The engine is pure over
  (structure, rules) and knows nothing about facts. Each consumer keeps its own
  fact model and filter language and pre-selects which rules apply: MindTape
  filters by `when` through its existing `Filter` over `Task` (unchanged, so no
  loss of `priority` / `stale` / framework predicates), then calls `run_rules`.
  This is cleaner than baking a generic fact-filter into the engine and is the
  convergence boundary — shared structure-matching, per-consumer facts. A
  fact-provider entry point for a standalone config-driven engine is the planned
  extension, not a v1 need.
- Both worlds shipped, consumer picks. Closed-world (`allowlist`) and open-world
  (`require`/`forbid`) coexist; a ruleset mixes them freely. Closed-world is a
  real primitive, not sugar — Rust `regex` has no negative lookahead, so `forbid`
  cannot express "anything not in `{A, B, C}`". speconaut's Decision artifact
  (Context / Choice / Consequences only) is the closed-world consumer.
- Ordering is strict and order-only: `max(first lines) < min(second lines)`, so
  interleaving is flagged; neither side is required to exist (pair with `require`
  or `unique` for presence). Diagnostic points at the earliest offending `second`.
- `unique` = exactly one: absent emits the require diagnostic (line none); each
  duplicate past the earliest is flagged at its own line.
- `kind` is carried on `Node` but still not a match dimension — text-based
  matching only, deferred until a consumer needs cross-kind rules.
- Fact/`when` seam stays externalized (engine pure over structure + rules); a
  fact-provider entry point for a standalone config-driven engine is still the
  planned extension, not built.

Follow-up to `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.
