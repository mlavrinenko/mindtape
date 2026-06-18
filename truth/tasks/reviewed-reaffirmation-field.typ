#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Idea: reviewed re-affirmation date as a core field",
  status: proposed(2026, 6, 18),
  priority: "low",
  estimate: (hours: 2),
  relates-to: (link("convergence-boundary-adr.typ"),),
  tags: ("substrate",),
)

== Summary

Idea for assessment. Add an optional core `reviewed` date: the last time a
proposed or accepted task was re-affirmed as still relevant. speconaut stamps
this to fight backlog rot — a task left untouched and unreviewed for long is a
staleness signal. Generic enough to live in MindTape core rather than
`x-speconaut`.

== Scope

- `reviewed` core slot holding a date; a `mindtape review <id>` could stamp
  today, and the filter language could gain a staleness predicate (reviewed
  older than N days).
- Open question: is a single `reviewed` date enough, or does this want a small
  review log? History-in-git argues for the single date; a log would break the
  forward-only model. Lean single date.
