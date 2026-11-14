#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Gate the estimate ceiling on difficulty",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  assignee: "agent",
  status: done(2026, 9, 14),
)

== Summary

`[tasks.estimate.max] files = 15` is a flat error. A mechanical change touching one line in a hundred files has to split into seven tasks that each carry the same single decision. The cap came from this repo's own files-per-commit distribution at p95, to create decomposition pressure. That pressure belongs on unbounded judgment rather than on size. `difficulty` already measures judgment, and the config itself says so when it tells the reader to score judgment required, not size.

== Scope

Turn `[tasks.estimate.max]` from a table into an array of tables, first matching entry wins, each carrying a `when` filter and a `message` like every other rule:

```toml
[[tasks.estimate.max]]
when = "difficulty >= 3"
files = 5
message = "judgment work stays small; split it rather than widen the cap"

[[tasks.estimate.max]]
files = 15
```

Symmetry calls for the same change to `[tasks.estimate.min]`. `message` gives a breach a reason the ceiling sits where it does instead of only the fact of the breach. `[tasks.estimate.drift]` already takes one, and a bound without an explanation is the kind of number an author learns to inflate rather than respect.
