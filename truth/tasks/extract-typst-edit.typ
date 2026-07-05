#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Extract typst-edit to its own repo (~/projects/home/typst-edit) and publish",
  status: proposed(2026, 7, 5)[
    Leaf typst crate; wait for the 0.15 bump. Highest-value substrate crate — no
    external equivalent does source rewriting.
  ],
  priority: ice(impact: 8, confidence: 0.8, ease: 7),
  tags: ("substrate", "tooling"),
  depends-on: (link("typst-015-bump.typ"),),
)

== Summary

Move `crates/typst-edit` into `~/projects/home/typst-edit` (cratemplate), then
publish. Parse-plane call/link span queries + atomic non-overlapping changeset
over typst-syntax; no eval/World/IO, no internal deps. Blocks only on the 0.15
bump. `crates.io` name confirmed free (404 as of 2026-07-05).

== Scope

Crate facts: `find_calls`/`find_method_calls`/`find_link_targets`/`apply`, the
`Edit`/`Call`/`Arg`/`Body` types. External deps: `typst-syntax = "0.15"`
(post-bump), `serde` (optional, `serde` feature). keywords/categories per current
manifest. Keep the `serde` feature.

== Extraction procedure

1. `cargo generate gh:mlavrinenko/cratemplate` -> name `typst-edit`, mlavrinenko,
   MIT. Lands `~/projects/home/typst-edit`.
2. LIB — drop `[[bin]]`/`src/main.rs`; drop clap/anyhow/log/env_logger.
3. Copy `crates/typst-edit/src/` (lib.rs, lib_tests.rs) into new `src/`.
4. Port metadata + `[features] serde` + deps. Reset version to 0.1.0.
5. Reconcile lints (template `cast_*` deny).
6. `git init` + commit; GitHub repo `mlavrinenko/typst-edit`; push.
7. `just check` + `just nix-build` green; top up tests for coverage/crap.
8. `cargo publish --dry-run` then `cargo publish`. Record version.

== Tests

- New repo gates green; the `apply` doctest still passes; `cargo publish
  --dry-run` clean.

== Notes

- Orchestrator owns repointing mindtape — don't touch the mindtape workspace.
