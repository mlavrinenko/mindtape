#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "derive the drift-prone doc facts from their sources",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("docs", "tooling"),
  estimate: (files: 8),
  links: related(
    "generate-the-root-markdown-from-typst-with-typlite.typ",
  )[the rendering hop that consumes what this extracts],
  status: proposed(2026, 8, 13),
)

== Summary

The facts that rot in the root docs are the ones with a machine-readable source: the crate map, dependency versions, the task DSL field list, and the just check gate table. Extract each into www/generated so a doc states it by reading rather than by repeating it. This is the layer that kills drift; the Typst/typlite rendering hop is separate and useless without it.

== Scope

- cargo metadata -> crate map + substrate deps + versions; rust-toolchain.toml -> pinned rustc; Cargo.lock -> typst/evalexpr/clap
- lib/prelude.typ task signature -> the DSL field list, via typst-cst (already an in-tree capability) or typst query
- just --dump --dump-format json -> the Quality Gates table; check: already names its sub-gates as deps and each recipe carries a doc comment
- .linecop.yaml -> the caps quoted in CONTRIBUTING.md
- Out of scope, deliberately: the design invariants (typst eval-only, evalexpr walked not evaluated, the prelude-vs-Rust split). They change with the design, not with a bump. Back never layout with a dependency assertion instead of scraping a comment
