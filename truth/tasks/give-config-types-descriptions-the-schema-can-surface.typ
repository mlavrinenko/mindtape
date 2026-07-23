#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Give config types descriptions the schema can surface",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt", "docs"),
  estimate: (files: 5),
  status: done(2026, 7, 23),
)

== Summary

A generated config reference is only as good as the doc strings behind it. Audit the config types so every section and key carries a description worth rendering, and fix any schema flaws found on the way.

== Scope

- Walk the config types and add or improve the doc comments that feed the JSON Schema
- Correct any wrong, missing or misleading description discovered
- Confirm the emitted schema carries the descriptions
