#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Trim release CI for lib-only extracted crates (cratemplate binary-matrix)",
  status: proposed(2026, 7, 5)[
    Surfaced by the extraction wave: two agents independently flagged it.
  ],
  priority: ice(impact: 6, confidence: 0.8, ease: 7),
  tags: ("tooling", "gating"),
)

== Summary

cratemplate's `.github/workflows/release.yml` assumes a binary target: on a `v*`
tag it runs a 5-target `build` matrix (`cargo build --release --target X` then
`cp target/<triple>/release/<binary> ...` + upload) and a `release` job that
attaches those artifacts. For a lib-only crate the build compiles but the binary
rename/upload steps fail (no binary exists). The `publish` job (`cargo publish
--locked`) is independent and works. Every substrate crate extracted from mindtape
is lib-only, so each ships this broken-on-tag workflow.

== Scope

Two fronts:

- Each extracted lib repo (~/projects/home/{swhid-mint, artifact-gate, typst-cst,
  typst-edit, typst-world, typst-harvest, typst-cleanup}): trim `release.yml` to
  the `check` + `publish` jobs (drop the `name`/`build`/`release` binary jobs, or
  guard them out). Keep the crates.io publish path intact. Owner-driven, since the
  owner owns repo-create + `CARGO_REGISTRY_TOKEN` + tagging.
- cratemplate itself: add a lib-vs-bin mode so generated lib crates get a
  lib-appropriate release workflow (and drop `[[bin]]`/`src/main.rs`/`tests/cli.rs`/
  the CLI-oriented README/AGENTS/CONTRIBUTING prose). A cargo-generate placeholder
  (`crate-kind = lib|bin`) with conditional includes is the clean fix. This is the
  generalising lesson — the extraction agents each had to hand-strip the bin
  scaffolding and rewrite the docs.

== Tests

- A `v0.1.0` tag on a trimmed lib repo runs `check` + `publish` green (dry-run the
  publish first).
- `cargo generate` in lib mode produces a repo whose `just check` + `nix build` are
  green with no bin residue.

== Notes

- Not a blocker for extraction/publish per se (publish job works), but must land
  before the owner tags any extracted lib crate, else the release run goes red.
- Also trim the stale `AGENTS.md`/`CONTRIBUTING.md` CLI-testing references the
  extraction agents left (out of their scope).
