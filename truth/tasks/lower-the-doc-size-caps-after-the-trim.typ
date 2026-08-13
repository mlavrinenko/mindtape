#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "lower the doc size caps after the trim",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (files: 2),
  links: depends-on("trim-root-docs-by-delegating-deep-content.typ")
    + related(
      "stop-the-doc-size-cap-fighting-correctness-fixes.typ",
    )[revises the caps that task raised, on the same reasoning],
  status: proposed(2026, 8, 13),
)

== Summary

Once the root docs are trimmed by delegation, the 300-line caps on AGENTS.md/CONTRIBUTING.md and 250 on README.md no longer describe the intended shape. Lower them so the trim holds instead of silently refilling.

== Scope

- README.md 250 -> 150, AGENTS.md/CONTRIBUTING.md 300 -> 200 once the trim lands
- This partially reverses stop-the-doc-size-cap-fighting-correctness-fixes.typ, and its reasoning still holds: a cap must not make a correctness fix expensive. It does not apply at ~90 lines, where 200 is 110 lines of headroom rather than a wall. Record that argument in .linecop.yaml next to the numbers, replacing the stale one
- Re-check linecop --baseline 85 reports nothing right after the change
