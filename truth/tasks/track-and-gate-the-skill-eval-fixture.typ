#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Track and gate the skill eval fixture",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 5),
  status: done(
    2026,
    7,
    26,
  )[fixture tracked under evals/files/, gated by check-examples with the mmz scope proven to catch changes],
)

== Summary

The mindtape skill's eval fixture lived at skills/mindtape-workspace/sandbox-template/, inside a gitignored scratch tree, and nothing validated it. It silently outlived two config migrations — the retired \[ls\] default-filter and the depends-on: to links: move — plus a priority max that no longer matched the default bands and a prelude stub at 110 lines against the real 560. Every eval in the tracked evals.json would have failed at config load. Move it under skills/mindtape/evals/files/ per skill-creator's files convention, and gate it so it cannot rot again.

== Scope

- Move sandbox-template to skills/mindtape/evals/files/sandbox/ (tracked) and point each eval's files array at it.
- Extend just check-examples to mt check the fixture, and add it to the mmz examples scope so the gate is not false-fresh.
- Track the vendored prelude and typst.toml copies as outdatty dependents, since the fixture must stay self-contained and cannot reference lib/.
- Document the fixtures gate in CONTRIBUTING.md.
