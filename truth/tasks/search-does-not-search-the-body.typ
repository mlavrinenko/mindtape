#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "search does not search the body",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli", "docs"),
  estimate: (files: 3),
  status: proposed(2026, 8, 7),
)

== Summary

`www/content/filters.typ` documents `search("query")` as "Full-text search
across the task". The implementation (`filter.rs`, the `"search"` arm) matches
`title` and `id` only. `Task.body` carries the whole prose body verbatim and is
already loaded and emitted in `--json`, so the body is available and simply
never consulted.

Measured on this backlog: `mt ls 'search("tombstone")'` returns 1 task, while
`rg -l tombstone truth/tasks/` finds 6 files. Five tasks are invisible to the
only search the filter language offers, and the docs say otherwise.

== Scope

- Decide the contract, then make code and docs agree: either `search()` reads
  `title` + `id` + `body` (matching the documented promise), or the docs stop
  calling it full-text.
- Recommended: extend it to the body. It is the reading everyone assumes, the
  data is already in memory, and it needs no new CLI surface — `search("x") &&
  ready` composes with every existing filter for free.
- Consider whether the marker fields inside the `#show: task.with(…)` head
  should be excluded from the haystack, since `Task.body` is already
  marker-free: a match on `tags:` or a link target is not a prose match.
- Regression test pinning a body-only match, and one pinning that a match in
  the marker head does not count twice.
- Update `www/content/filters.typ` and `skills/mindtape/SKILL.md` to whichever
  contract wins.
