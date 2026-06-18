#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Reviewed re-affirmation date",
  status: proposed(2026, 6, 18),
  priority: "low",
  estimate: (hours: 2),
  relates-to: (link("convergence-boundary-adr.typ"),),
  tags: ("substrate",),
  reviewed: on(2026, 6, 18),
)

== Summary

Idea for assessment. Add an optional core `reviewed` date: the last time a
proposed or accepted task was re-affirmed as still relevant. speconaut stamps
this to fight backlog rot — a task left untouched and unreviewed for long is a
staleness signal. Generic enough to live in MindTape core rather than
`x-speconaut`.

== Scope

- `reviewed` core slot holding a date (sits beside `due` / `start` in
  `_core-slots`). The filter language gains a staleness predicate (reviewed
  older than N days, or unreviewed-since-creation).
- Open question: is a single `reviewed` date enough, or does this want a small
  review log? History-in-git argues for the single date; a log would break the
  forward-only model. Lean single date.

== DSL: date shortcut

`reviewed: datetime(year: 2026, month: 6, day: 18)` is too wordy. Add a generic
date constructor to the prelude — `on(2026, 6, 18)` wrapping `datetime(...)` —
consistent with the existing positional `(y, m, d)` status constructors
(`proposed(2026, 6, 18)`). Make it serve every date slot uniformly (`due`,
`start`, `reviewed`), not review-only:

```typ
reviewed: on(2026, 6, 18),
```

`on` reads naturally in slot position; `date` is the more self-documenting
alternative. Pick one, export from the prelude, retrofit `due` / `start`
examples. Prelude stays consumer-blind — it is just a `datetime` shorthand.

== CLI: `mindtape review <id>`

MindTape has no task-mutation CLI yet (speconaut's `task review` is slated for
deletion/absorption — see speconaut `delete-task-commands-reanchor-session`).
`mindtape review` lands as part of that mutation surface, alongside `show` /
`add`. Semantics: stamp `reviewed:` with today, idempotent same-day, refuse on
terminal status (`done` / `cancelled`).

Task-reference resolution is generic, shared by every task-taking command
(`show`, `review`, `relabel`, …) — a single library helper, not a per-command
concern. It accepts:

- slug — task filename stem without `.typ`
- relative path
- absolute path

== Editing: `typst-edit`

Stamping `reviewed:` is a slot rewrite over `typst-syntax` spans via the planned
`typst-edit` crate (ADR 003): locate the `reviewed:` argument span (or its
insertion point in `task.with(...)` when absent), plan the changeset, apply
atomically, abort on a wrong-shaped file. `review` is the first `typst-edit`
writer beyond `cleanup`.
