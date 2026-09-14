#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add a max-words content assert",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  assignee: "agent",
  status: done(2026, 9, 14)[max-words variant added beside min-words],
)

== Summary

`ContentAssert` (crates/mindtape/src/body_content.rs) offers "non-empty", `min-words`, `must-match` and `must-not-match`, so a config can demand that a section say enough but never that it stop. Conciseness is the property the section rules most need and the only one not expressible today: nothing stops an agent padding AS-IS to four hundred words of narrative, which is exactly what turns a spec into prose nobody reads.

== Scope

Add a `{ max-words = N }` variant beside `min-words` in ContentAssert, its entry in schema/config.schema.json and the docs sites config page, with a red test for the boundary — exactly N passes, N+1 fails. Word counting reuses whatever `min-words` already does so the two agree by construction.
