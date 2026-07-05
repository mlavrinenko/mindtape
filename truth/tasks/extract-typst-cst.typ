#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Extract typst-cst to its own repo (~/projects/home/typst-cst) and publish",
  status: wip(2026, 7, 5)[Leaf typst crate; 0.15 bump done — extracting.],
  priority: ice(impact: 6, confidence: 0.8, ease: 7),
  tags: ("substrate", "tooling"),
  depends-on: (link("typst-015-bump.typ"),),
)

== Summary

Move `crates/typst-cst` into `~/projects/home/typst-cst` (cratemplate), then
publish. Parse-plane heading extraction over typst-syntax; no eval/World/IO, no
internal deps. Blocks only on the 0.15 bump (so the extracted crate ships on
typst-syntax 0.15).

== Scope

Crate facts: `parse_headings` + `tree` over `typst-syntax`. External deps:
`typst-syntax = "0.15"` (post-bump), `serde` (optional, `serde` feature).
keywords/categories per current manifest. Keep the `serde` feature.

== Extraction procedure

1. `cargo generate gh:mlavrinenko/cratemplate` -> name `typst-cst`, mlavrinenko,
   MIT. Lands `~/projects/home/typst-cst`.
2. LIB — drop `[[bin]]`/`src/main.rs`; drop clap/anyhow/log/env_logger.
3. Copy `crates/typst-cst/src/` (lib.rs, lib_tests.rs) into new `src/`.
4. Port metadata + `[features] serde` + deps (typst-syntax 0.15, serde optional).
   Reset version to 0.1.0.
5. Reconcile lints (template `cast_*` deny).
6. `git init` + commit; GitHub repo `mlavrinenko/typst-cst`; push.
7. `just check` + `just nix-build` green; top up tests for coverage/crap.
8. `cargo publish --dry-run` then `cargo publish`. Record version.

== Tests

- New repo gates green; the doctest in `lib.rs` still passes; `cargo publish
  --dry-run` clean.

== Notes

- Orchestrator owns repointing mindtape — don't touch the mindtape workspace.
