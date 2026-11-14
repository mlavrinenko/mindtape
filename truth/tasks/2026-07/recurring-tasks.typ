#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Recurring tasks",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 3,
  ),
  tags: (),
  links: depends-on(
    "typed-relation-edges.typ",
    "configurable-task-stem-naming.typ",
  ),
  status: done(2026, 7, 10),
)

== Summary

Add a `recur` field so a recurring task can advance to its next occurrence. It
stays file-native and daemon-free. The next occurrence materializes only on an
explicit `mt recur`, never from a background process, and never as a side effect
of a read (see command-semantics-cqrs). The design supports two materialization
models: spawn a new file per occurrence, or refresh one file in place. Both run
through `mt recur` and the task selects one. `mt done` stays a pure close and
leaves the series alone.

The neighbour survey shaped the design. `todoman` is the structural model to
copy: it spawns a new `UID` on completion and links via `RELATED-TO`.
Taskwarrior (parent template plus `mask`/`imask`, lazy-on-report generation) is
the model to avoid.

== Field shape

- The `recur` field carries a dict shaped by a helper, with no date math in Typst:
  same discipline as the priority frameworks:
  - `recur: every(weeks: 1)`: simple period.
  - `every(days:, weeks:, months:, years:)`: one period unit.
  - `every(weeks: 1, on: (mon, wed, fri))`: a weekday set.
  - `recur: rrule(...)`: the escape hatch (its own task, rrule-escape-hatch,
    feature-gated, due-anchored only). It takes an RRULE string:

    ```typ
    recur: rrule("FREQ=MONTHLY;BYDAY=1MO")
    ```
- Recurrence policy rides inside the same call, not detached into sibling fields:
  `every(weeks: 1, anchor: "completion", until: on(2027, 1, 1), count: 10,
  materialize: "spawn")`. `rrule(...)` takes the same policy `args`.
- The helper shapes `(rule, anchor, until, count, materialize)`. The Rust layer
  validates against `[tasks.recur]` and computes the next date, with `chrono` for
  periods and rrule for the escape hatch.

== Anchor

- `anchor: "completion"` is the default. Because recurrence advances only on an
  explicit `mt recur`, the completion date is just today at advance time, so a
  completion anchor is always in the future and skips catch-up. This is the
  chore-natural mode Taskwarrior specified (`rtype:chained`) but never implemented.
- `anchor: "due"` is opt-in for calendar-fixed cadences (rent on the 1st,
  standup every Monday). Due-anchored defaults to catch-up: advance by interval
  until strictly after today (Org `++`), so a late completion never generates an
  already-overdue occurrence. A preserve-every-period mode (Org `+`) is a later
  `catch-up: false`, not this version.

== Materialization

Declared per task via `materialize`, with a default of `spawn` that
`[tasks.recur] materialize` overrides. A series uses one model, not a per-invocation flag.

- `spawn` (default): `mt recur X` writes the next instance. It is a new file,
  with a stem minted by configurable-task-stem-naming (occurrence anchor date),
  carrying the `recur` rule forward and a `recurred-from(X)` edge (cardinality 1) back to its
  predecessor. It does not change X's status. Guard: refuse if X already has a
  successor (a loaded task whose `recurred-from` names X) unless `--force`. This
  prevents double-spawn without mutating the done file. `mt done X` closes an
  instance normally. The series lives as long as you keep recurring, and stops
  when you stop, or when `until` or `count` runs out.
- `mutate`: `mt recur X` refreshes the same file in place: reset status to the
  initial legend value dated today, advance `due`/`start` per the anchor, and
  increment an occurrence counter (needed for `count`/`until`). One stem forever,
  with no proliferation. The audit trail is git history. Here `mt done X` ends the
  series (terminal, no further advance).

== Commands

- `mt recur X`: advance one step per the materialization model.
- `mt recur X --dry-run`: preview the spawn or mutate, without writing.
- `mt recur X --advance N`: pre-materialize N future instances (spawn only)
  without closing the current, as the get-ahead or batch path. Only the first hop
  honours the anchor. Each later one steps off the occurrence before it, so a
  completion anchor (which resolves from today) spaces the chain by the rule
  instead of planning one date N times.
- `mt recur X --skip`: advance the anchor and counter, and create or refresh nothing.
- `mt done X`: pure close. On a recurring task, print a notice ("'X' is
  recurring. Run `mt recur X` to advance") and do not strip `recur`.

== Stop conditions

- `until` bounds the series (stop spawning/advancing once the next date passes
  it). `count` caps total occurrences (decremented on the carried instance in
  spawn mode, tracked by the counter in mutate mode).
- Keep "series ends" and "instance expires" distinct: Taskwarrior conflates
  them in one inherited `until` (issue #2308, open for years). `until` here means
  series end only.

== Config

`[tasks.recur]`: `enabled`, `units` (allowed period units), `anchors` (allowed),
`default-anchor` (`completion`), `materialize` (`spawn`), `rrule` (feature flag,
off), `max-advance` (runaway cap, one instance per `mt recur` unless
`--advance N`).

== Constraints

- CQRS: `mt recur` is the only recurrence writer. `mt ls` and `mt check` never
  spawn (regression-tested under command-semantics-cqrs).
- Depends on typed-relation-edges (the `recurred-from` edge) and
  configurable-task-stem-naming (unique occurrence stems).
- Cap generation at one instance per `mt recur`, and never backfill a stale
  anchor. Only `--advance N` makes more than one, bounded by N.

== Scope

- prelude `every()` (and the shared shape for `rrule()`), plus `recur` in the marker.
- model + validation for the `recur` field against `[tasks.recur]`.
- `mt recur` (spawn / mutate / dry-run / advance / skip) and the `mt done` notice.
- `chrono` period arithmetic, completion default, due and catch-up.
- `recurred-from` link on spawn, plus the back-link double-spawn guard.
- Tests: weekly and monthly. Completion vs due with catch-up. Spawn vs mutate.
  `until` / `count` end. `--dry-run` leaves the files alone. The back-link guard holds.
  A read command leaves the series untouched.
