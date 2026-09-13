#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Gate the estimate ceiling on difficulty",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

`[tasks.estimate.max] files = 15` is a flat error, so a mechanical change touching one line in a hundred files must be split into seven tasks that each carry the same single decision. The cap was calibrated on this repos own files-per-commit distribution at p95 to create decomposition pressure, but that pressure belongs on unbounded judgment rather than on size — which is what `difficulty` already measures, and what the config itself says when it tells the reader to score JUDGMENT REQUIRED, not size.

== Scope

Let the estimate bounds carry a `when` filter like every other rule, so a project can write a tight ceiling for `difficulty >= 3` and a loose one for mechanical work. Carry a `message` beside it too, so a breach says why the ceiling sits where it does instead of only that it was crossed — `[tasks.estimate.drift]` already takes one, and a bound without an explanation is the kind of number an author learns to inflate rather than respect. The schema cost is turning `[tasks.estimate.max]` from a table into an array of tables, first matching entry wins; the same applies to `[tasks.estimate.min]` for symmetry.
