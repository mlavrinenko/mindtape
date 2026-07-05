#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Extract typst-cleanup to its own repo (~/projects/home/typst-cleanup) and publish",
  status: proposed(2026, 7, 5)[
    Depends on swhid-mint + typst-edit being published first (they are its deps).
  ],
  priority: ice(impact: 6, confidence: 0.7, ease: 6),
  tags: ("substrate", "tooling"),
  depends-on: (
    link("extract-swhid-mint.typ"),
    link("extract-typst-edit.typ"),
    link("typst-015-bump.typ"),
  ),
)

== Summary

Move `crates/typst-cleanup` into `~/projects/home/typst-cleanup` (cratemplate),
then publish. Consumer-blind retire-artifact orchestration (dirty-tree guard,
SWHID tombstone, atomic inbound-link rewrite). Depends on its crate deps
(`swhid-mint`, `typst-edit`) already being on crates.io — hence those two extract
tasks are prerequisites.

== Scope

Crate facts: `plan`/`apply` cleanup skeleton. Internal deps → published:
`swhid-mint` (with `test-support` in dev-deps), `typst-edit`. External dep:
`gix = { 0.84, default-features=false, features=["status","sha1"] }`.
keywords/categories per current manifest.

== Extraction procedure

1. `cargo generate gh:mlavrinenko/cratemplate` -> name `typst-cleanup`,
   mlavrinenko, MIT. Lands `~/projects/home/typst-cleanup`.
2. LIB — drop `[[bin]]`/`src/main.rs`; drop clap/anyhow/log/env_logger.
3. Copy `crates/typst-cleanup/src/` (git.rs, lib.rs, lib_tests.rs) into new `src/`.
4. Port metadata + deps; point `swhid-mint`/`typst-edit` at their PUBLISHED
   versions (dev-dep keeps `swhid-mint` `features=["test-support"]`). Reset
   version to 0.1.0.
5. Reconcile lints (template `cast_*` deny).
6. `git init` + commit; GitHub repo `mlavrinenko/typst-cleanup`; push.
7. `just check` + `just nix-build` green; top up tests for coverage/crap.
8. `cargo publish --dry-run` then `cargo publish`. Record version.

== Tests

- Gates green; `cargo publish --dry-run` clean against published swhid-mint /
  typst-edit.

== Notes

- Orchestrator owns repointing mindtape-cli (the sole consumer of typst-cleanup).
