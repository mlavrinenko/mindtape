#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "CI `--in-diff` mutation advisory",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 6,
  ),
  tags: ("tooling", "tests"),
  status: cancelled(
    2026,
    7,
    21,
  )[decided against in simplify-the-mutants-daily-loop: TIMEOUT verdicts are untrustworthy on a shared runner; run just mutants-diff locally],
)

== Summary

Decide whether a per-PR `cargo mutants --in-diff` advisory check earns its keep now that the equivalent-mutant floor in `.cargo/mutants.toml` is stable (10 entries, each with a documented reason). Scoped to touched lines, so fast; non-blocking at first. Promote to a gate only if it proves low-noise.

== Scope

- Measure wall time of `--in-diff` on a representative PR diff
- Decide advisory (non-blocking) vs gate
- Wire into CI if it pays off, else close as not-worth-it
