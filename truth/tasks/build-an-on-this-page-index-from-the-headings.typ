#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Build an on-this-page index from the headings",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("www",),
  estimate: (files: 3),
  status: proposed(2026, 7, 23),
)

== Summary

Long reference pages have no way to skim their own structure. Derive a per-page index from the headings themselves, so it can never fall out of step with the content it indexes.

== Scope

- Derive the index from the rendered headings rather than a hand-written list
- Show it only where a page has enough headings to be worth indexing
- Keep it out of the way on narrow screens
