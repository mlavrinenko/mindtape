#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Idea: ready-state predicates in the filter language",
  status: proposed(2026, 6, 18),
  priority: 2,
  estimate: (hours: 3),
  relates-to: (link("absorb-task-graph-into-lib.typ"),),
  tags: ("cli",),
)

== Summary

Idea for assessment. Extend the filter language with dependency-aware predicates
so `mindtape list` can answer "what can I start now": `ready()` (every
`depends-on` target terminal) and `blocked()` (its negation). Back them with the
readiness function from `absorb-task-graph-into-lib.typ`. Add a `--ready`
shorthand to `list` and `next`.

== Scope

- Filter functions `ready()` / `blocked()` evaluated against the loaded task set
  — they need the whole set, not one task, so confirm the walker can see it.
- `mindtape list --ready` as sugar for `list 'ready()'`.
- Optional: a `superseded()` predicate once `supersedes` is core, to hide work
  retired by a replacement.
