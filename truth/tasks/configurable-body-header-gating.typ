#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Configurable body-header gating in check",
  status: proposed(2026, 6, 14),
  priority: 5,
  estimate: (hours: 4),
  depends-on: (
    link("artifact-gate-rule-engine.typ"),
    link("swh:1:rev:8069f3182b74520070fcfeb584a5626df6db1203;path=truth/tasks/typst-cst-structure-extraction.typ"),
  ),
  relates-to: (link("converge-speconaut.typ"),),
  tags: ("gating", "cli"),
)

== Summary

Let a project gate the section headings of its task bodies from
`.mindtape/config.toml`, each rule scoped by a task-filter expression (the same
language as `query.default_filter`). Required because speconaut needs to require
and forbid headers per artifact kind. Today `check` validates only the
structured metadata fields; the body is opaque to the tool.

== Scope

- Config schema `[[tasks.body.headers.rules]]`, each rule:
  - `path`: list of segments. A bare string segment matches the heading text
    exactly, case-sensitive (matching `has_tag` semantics). A `{ match = "..." }`
    segment is a Rust regex against the heading's plain text. `recursive = true`
    on a segment means any descendant below the previous segment; the default is
    a direct child (depth + 1).
  - `action`: required, no default — `require` or `forbid`.
  - `when`: optional filter expression over task fields; absent means always.
  - `message`: custom diagnostic text.
- Wire into `check`: run the `artifact-gate` engine over each task's heading tree
  (from `typst-cst`) and fields (from `typst-harvest`).
- A rule that fails to parse fails `check` loudly as a config error — never
  skipped silently.
- Diagnostics carry file:line from the `typst-cst` spans.

== Example

```toml
[[tasks.body.headers.rules]]
when    = 'has_tag("milestone")'
path    = ["Approach", { match = "^Phase", recursive = true }]
action  = "forbid"
message = "Phases forbidden under Approach. Decompose instead."

[[tasks.body.headers.rules]]
when    = 'kind == "spec"'
path    = ["Acceptance Criteria"]
action  = "require"
```

Follow-up to `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.
