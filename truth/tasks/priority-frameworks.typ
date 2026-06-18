#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Priority frameworks: scalar and categorical, config-defined",
  status: proposed(2026, 6, 19),
  priority: "normal",
  estimate: (days: 2),
  depends-on: (link("numeric-priority-scale.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Compute the numeric priority from a configured framework instead of hand-set
integers. Every framework yields a scalar score (for sort and banding) and
optionally a category (a named display cell); scalar and categorical are one
mechanism, with or without categories. Frameworks live in config data, not Rust —
Rust supplies only the eval engine (evalexpr, the filter language's) and
validation. Built-ins ship as a baked-in default config layer; a user
`[tasks.priority.frameworks.<name>]` deep-merges over it.

== Scope

- Framework config: typed and bounded `inputs`; a `score` formula (evalexpr); an
  optional `category` list, each entry a `when` predicate (evalexpr) plus score,
  glyph, and colour. Active framework via `[tasks.priority] framework = "…"`.
- Prelude ships thin packagers (`ice(…)`, `rice(…)`, `eisenhower(…)`) that only
  shape `(framework, inputs)` — no math in Typst.
- Validation: input shape and bounds; categorical exhaustiveness and disjointness
  for boolean / enum inputs (numeric inputs use bands, not categories).
- In-bounds by construction: at config load, interval-evaluate `score` over the
  declared input bounds and reject any framework whose achievable range exceeds
  the scale; a per-task runtime backstop guards the rest. No silent clamping.
- Manual escape hatch: `[tasks.priority] allow-manual` — with a framework active,
  a task may set a raw int on the scale; strict mode forbids it. Manual ints
  validate within bounds and sort uniformly with framework scores (same scale).
- Ship built-ins: ICE (the active default here), RICE, WSJF, Eisenhower, MoSCoW.
