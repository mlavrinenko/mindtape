#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt rm must retarget links: edges to tombstones",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 4,
  ),
  tags: (),
  status: done(2026, 7, 11)[rm scans links: helper edges; retargets every kind to tombstone],
)

== Summary

`mt rm` retires a task and rewrites its inbound links into SWHID tombstones via
the external `typst-cleanup` / `typst-edit` crates, whose scanner finds targets
by locating literal `link(...)` calls in the source text. The typed-relation-edges
change authors every edge as a helper call over plain stem strings
(`depends-on("x.typ")`), never `link(...)` (an explicit spec decision), so
retiring a task no longer retargets inbound `links:` edges — they dangle, and a
follow-up `mt check` then reports the survivor's edge as an unresolved target.

Regression introduced by typed-relation-edges and confirmed empirically: after
the backlog migration, `mt rm --dry-run typed-relation-edges` reports only the
deletion, with no inbound rewrite of `recurring-tasks.typ`'s
`depends-on("typed-relation-edges.typ")` edge.

== Scope

- Give the rm path a string-target scan over the new `links:` helper arguments
  (or extend `typst-cleanup`'s `Policy`/scanner to recognise helper-shaped string
  targets), so inbound edges of every kind retarget to the SWHID tombstone.
- Preserve the atomic plan/apply and the dirty-tree guard.
- Test: a survivor with `depends-on` / `related` / custom-kind edges to a retired
  task has each edge rewritten to the SWHID pointer, and `mt check` stays green
  afterward.
