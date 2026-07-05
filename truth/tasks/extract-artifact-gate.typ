#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Extract artifact-gate to its own repo (~/projects/home/artifact-gate) and publish",
  status: wip(2026, 7, 5)[
    Plane-agnostic, no typst, no internal deps — extractable immediately.
  ],
  priority: ice(impact: 6, confidence: 0.8, ease: 7),
  tags: ("substrate", "tooling", "gating"),
)

== Summary

Move `crates/artifact-gate` into `~/projects/home/artifact-gate` (cratemplate),
then publish. Independent: plane-agnostic structural rule engine, no typst, no
internal deps — blocks on nothing.

== Scope

Crate facts: match a node tree against require/forbid/order/allow path rules, emit
diagnostics. External deps: `regex = "1"`, `thiserror = "2"`. keywords/categories
per current manifest. Consumer-blind — keep it free of any task/typst concept.

== Extraction procedure

1. `cargo generate gh:mlavrinenko/cratemplate` -> name `artifact-gate`, gh-username
   `mlavrinenko`, license MIT. Lands `~/projects/home/artifact-gate`.
2. LIB, not bin — drop `[[bin]]` + `src/main.rs`, keep `[lib]`; drop clap/anyhow/
   log/env_logger.
3. Copy `crates/artifact-gate/src/` (eval.rs, lib.rs, `_tests.rs`) into new `src/`.
4. Port metadata + external deps (regex, thiserror). Reset version to 0.1.0.
5. Reconcile lints (template `cast_*` at deny) — fix new hits.
6. `git init` + commit; GitHub repo `mlavrinenko/artifact-gate`; push.
7. `just check` + `just nix-build` green; top up tests for cargo-crap/coverage.
8. `cargo publish --dry-run` then `cargo publish`. Record version.

== Tests

- New repo `just check` + `just nix-build` green; `cargo publish --dry-run` clean.

== Notes

- Do NOT touch mindtape workspace / `truth/tasks/*` — repointing is the umbrella
  publish task's job.
