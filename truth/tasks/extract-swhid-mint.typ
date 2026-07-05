#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Extract swhid-mint to its own repo (~/projects/home/swhid-mint) and publish",
  status: proposed(2026, 7, 5)[
    No typst, no internal deps — extractable immediately, in parallel with the
    0.15 bump. A good first move to shake out the cratemplate process.
  ],
  priority: ice(impact: 7, confidence: 0.8, ease: 7),
  tags: ("substrate", "tooling"),
)

== Summary

Move `crates/swhid-mint` into a standalone repo `~/projects/home/swhid-mint`
generated from cratemplate, then publish to crates.io. Fully independent: no typst
dependency, no internal crate deps — this task blocks on nothing.

== Scope

Crate facts: SWHID mint/resolve; `default = ["git"]` (gix backend),
`test-support` feature (tempfile). keywords/categories as in the current manifest.
External deps: `gix = { 0.84, default-features=false, features=["revision","sha1"], optional }`,
`tempfile` (optional). It exposes `test_support` for sibling/CLI tests via the
`test-support` feature — preserve that feature and its git fixtures.

== Extraction procedure

1. `cargo generate gh:mlavrinenko/cratemplate` -> name `swhid-mint`, gh-username
   `mlavrinenko`, license MIT. Lands `~/projects/home/swhid-mint`.
2. It scaffolds lib+bin; this is a LIB — drop `[[bin]]` + `src/main.rs`, keep
   `[lib]`; drop clap/anyhow/log/env_logger unless used (swhid-mint uses none).
3. Copy `crates/swhid-mint/src/` (git.rs, lib.rs, test_support.rs, `_tests.rs`)
   into the new `src/`.
4. Port package metadata + the `[features]` block + external deps above into the
   generated Cargo.toml. Reset version to 0.1.0.
5. Reconcile lints: template pins `cast_*` at `deny` (was `warn`) — fix new hits.
6. `git init` + initial commit; create GitHub repo `mlavrinenko/swhid-mint`; push.
7. Green the gates in the new repo: `just check` + `just nix-build`; the template
   adds cargo-crap + 70% coverage — top up tests if the gate trips.
8. `cargo publish --dry-run` then `cargo publish`. Record the published version.

== Tests

- New repo `just check` + `just nix-build` green.
- `cargo publish --dry-run` clean.

== Notes

- Do NOT touch `truth/tasks/*` or the mindtape workspace here — the orchestrator
  repoints mindtape to the published crate in the umbrella publish task.
