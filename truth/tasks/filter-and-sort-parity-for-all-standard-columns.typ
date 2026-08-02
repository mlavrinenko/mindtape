#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Filter and sort parity for all standard columns",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 8,
  tags: (),
  estimate: (files: 2),
  status: proposed(2026, 8, 2),
)

== Summary

Currently not all fields are filterable and sortable. For example, we can't filter or sort by difficulty.
We should fix and, but also come up with some systematic approach so it wouldn't be possible in the future.
Apply refactoring if needed.

== Scope

- TODO
