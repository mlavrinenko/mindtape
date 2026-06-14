#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Parse-plane structure extraction (typst-cst)",
  status: proposed(2026, 6, 14),
  priority: "normal",
  estimate-hours: 4,
  relates-to: (link("extract-typst-harvest.typ"),),
  tags: ("substrate", "gating"),
)

== Summary

The parse-plane counterpart to `typst-harvest`. Where the harvester evaluates a
file to collect computed `metadata()` markers (expensive, needs a `World`), this
extracts the file's *source structure* — the heading tree (depth, plain text,
file:line span) — by parsing only. `typst_syntax::parse` is pure: no eval, no
`World`, no IO, and `typst-syntax` 0.14 is already a transitive dependency.

== Scope

- New crate/module over `typst-syntax` 0.14. Walk the CST: on a
  `SyntaxKind::Markup` node whose parent is `SyntaxKind::Heading`, take
  `into_text()` for the title and `ast::Heading::depth()` for the level; map the
  node range to file:line via `source.lines().byte_to_line(range.start)`.
- Output: a depth-ordered heading list plus the derived nesting tree, each node
  carrying its source span.
- Optional `serde` JSON emit for external consumers (editors, other languages).
- Extraction only — no rules, no domain types.

== Decision

Use Typst's own parser; reject the alternatives surveyed:

- ast-grep + tree-sitter-typst: a custom language needs a compiled grammar
  `.so` registered via `sgconfig.yml` (CLI-only; no documented Rust-library
  path), and community grammars lag (uben0 ~0.11, none confirmed at 0.14). A
  second parser drifts from the canonical one we already ship.
- mephistypsteles: a WASM plugin wrapping `typst-syntax`; a Rust → WASM →
  serialized-value round-trip, version-coupled. Built for Typst-side
  reflection, not Rust tooling.
- tinymist-query: its `get_lexical_hierarchy` is pure-syntactic and exactly the
  right pattern, but the crate is unusable as a dependency — `pub(crate)`, drags
  the full LSP stack, and pins a patched `typst` fork that conflicts with
  crates.io `typst` 0.14. Lift the pattern (Apache-2.0), not the crate.

Follow-up to `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.
