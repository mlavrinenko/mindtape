#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "gate the justfile format with just fmt",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling", "gating"),
  estimate: (files: 3),
  status: done(2026, 8, 14),
)

== Summary

Turn `just --fmt --check` into a gate. Every other language in this repo has a formatter under `just fmt-check` — Rust, TOML, Typst — and the Justfile, the file that defines those gates, has none.

== Why

Verified: `just --unstable --fmt --check` exits 1 on today's Justfile, so it is already unformatted. `--fmt` is still an unstable subcommand and needs `set unstable := true` in the Justfile (or `--unstable` / `JUST_UNSTABLE`).

== Scope

- `set unstable := true` at the top of the Justfile, next to `set quiet := true`.
- Apply `just --fmt` once as its own reformat, so the mechanical diff never rides along with a behavioural change.
- Add the check to `just fmt-check` alongside `cargo fmt`, `tombi format --check` and `typstyle --check`, and to `just fmt` on the writing side — one path list feeding both, the house rule already stated in CONTRIBUTING.md.
- `.mmz/config.yaml`: `just fmt-check` already lists `gates-meta` (which is the Justfile), so no scope edit is needed — confirm rather than assume.
- Check whether `just --fmt` preserves the rationale comment blocks byte-for-byte before committing the reformat. It preserved them in a spot check; the whole file is the test.
