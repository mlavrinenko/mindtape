#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Recurring tasks",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 3,
  ),
  tags: (),
  depends-on: (
    link("typed-relation-edges.typ"),
    link("configurable-task-stem-naming.typ"),
  ),
  status: proposed(2026, 7, 10),
)

== Summary

Add a `recur` field so a recurring task can advance to its next occurrence.
File-native, daemon-free: the next occurrence materializes only on an explicit
`mt recur`, never by a background process and never as a side effect of a read
(see command-semantics-cqrs). Two materialization models are supported — spawn a
new file per occurrence, or refresh one file in place — both driven by `mt recur`
and selected per task. `mt done` stays a pure close and creates nothing.

Design shaped by the neighbour survey: todoman (spawn a new UID on completion,
link via `RELATED-TO`) is the structural model to copy; Taskwarrior (parent
template + `mask`/`imask`, lazy-on-report generation) is the model to avoid.

== Field shape

- The `recur` field carries a dict shaped by a helper, no date math in Typst —
  same discipline as the priority frameworks:
  - `recur: every(weeks: 1)` — simple period.
  - `every(days:, weeks:, months:, years:)` — one period unit.
  - `every(weeks: 1, on: (mon, wed, fri))` — a weekday set.
  - `recur: rrule("FREQ=MONTHLY;BYDAY=1MO")` — the escape hatch (its own task,
    rrule-escape-hatch; feature-gated, due-anchored only).
- Recurrence policy rides inside the same call, not detached into sibling fields:
  `every(weeks: 1, anchor: "completion", until: on(2027, 1, 1), count: 10,
  materialize: "spawn")`. `rrule(...)` takes the same policy args.
- The helper shapes `(rule, anchor, until, count, materialize)`; the Rust layer
  validates against `[tasks.recur]` and computes the next date (chrono for
  periods; rrule for the escape hatch).

== Anchor

- `anchor: "completion"` is the default. Because recurrence advances only on an
  explicit `mt recur`, the completion date is just today at advance time, so a
  completion anchor is always in the future and needs no catch-up. This is the
  chore-natural mode Taskwarrior specced (`rtype:chained`) but never shipped.
- `anchor: "due"` is opt-in for calendar-fixed cadences (rent on the 1st,
  standup every Monday). Due-anchored defaults to catch-up: advance by interval
  until strictly after today (Org `++`), so a late completion never generates an
  already-overdue occurrence. A preserve-every-period mode (Org `+`) is a later
  `catch-up: false`, not now.

== Materialization

Declared per task via `materialize` (default `spawn`, overridable by
`[tasks.recur] materialize`). A series uses one model, not a per-invocation flag.

- `spawn` (default): `mt recur X` writes the next instance — a new file, stem
  minted by configurable-task-stem-naming (occurrence anchor date), carrying the
  `recur` rule forward and a `recurred-from(X)` edge (cardinality 1) back to its
  predecessor. It does NOT change X's status. Guard: refuse if X already has a
  successor (a loaded task whose `recurred-from` names X) unless `--force`; this
  prevents double-spawn without mutating the done file. `mt done X` closes an
  instance normally; the series lives as long as you keep recurring, and stops
  when you stop (or when `until`/`count` is exhausted).
- `mutate`: `mt recur X` refreshes the same file in place — reset status to the
  initial legend value dated today, advance `due`/`start` per the anchor, and
  increment an occurrence counter (needed for `count`/`until`). One stem forever,
  no proliferation; the audit trail is git history. Here `mt done X` ends the
  series (terminal, no further advance).

== Commands

- `mt recur X` — advance one step per the materialization model.
- `mt recur X --dry-run` — preview the spawn/mutate; write nothing.
- `mt recur X --advance N` — pre-materialize N future instances (spawn only)
  without closing the current; the get-ahead / batch path.
- `mt recur X --skip` — advance the anchor and counter, create/refresh nothing.
- `mt done X` — pure close; on a recurring task, print a notice ("'X' is
  recurring — run `mt recur X` to advance") and do not strip `recur`.

== Stop conditions

- `until` bounds the series (stop spawning/advancing once the next date passes
  it). `count` caps total occurrences (decremented on the carried instance in
  spawn mode; tracked by the counter in mutate mode).
- Keep "series ends" and "instance expires" distinct — Taskwarrior conflates
  them in one inherited `until` (issue #2308, open for years). `until` here means
  series end only.

== Config

`[tasks.recur]`: `enabled`, `units` (allowed period units), `anchors` (allowed),
`default-anchor` (`completion`), `materialize` (`spawn`), `rrule` (feature flag,
off), `max-advance` (runaway cap; one instance per `mt recur` unless
`--advance N`).

== Constraints

- CQRS: `mt recur` is the only recurrence writer; `mt ls`/`mt check` never spawn
  (regression-tested under command-semantics-cqrs).
- Depends on typed-relation-edges (the `recurred-from` edge) and
  configurable-task-stem-naming (unique occurrence stems).
- Cap generation at one instance per `mt recur`; never backfill a stale anchor —
  only `--advance N` makes several, bounded by N.

== Scope

- prelude `every()` (and the shared shape for `rrule()`); `recur` in the marker.
- model + validation for the `recur` field against `[tasks.recur]`.
- `mt recur` (spawn / mutate / dry-run / advance / skip) and the `mt done` notice.
- chrono period arithmetic; completion default; due + catch-up.
- `recurred-from` link on spawn; back-link double-spawn guard.
- Tests: weekly + monthly; completion vs due + catch-up; spawn vs mutate;
  `until` / `count` end; `--dry-run` writes nothing; back-link guard; a read
  command spawns nothing.
