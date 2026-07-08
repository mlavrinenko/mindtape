#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Recurring tasks",
  priority: ice(
    impact: 6,
    confidence: 0.5,
    ease: 3,
  ),
  tags: (),
  status: proposed(2026, 7, 8),
)

== Summary

Add a recurrence field so completing a task spawns its next instance. The
comparison calls this out under "where the others win": Taskwarrior and Org-mode
do genuine repeats, MindTape does none. Keep it file-native — no daemon, no
hidden scheduler. The next occurrence materializes only when you act (`mt done`,
or on demand), by writing the next task file. This is the design-sensitive one;
it earns its place only if recurrence stays true to one-file-per-task and
daemonless.

== Constraints

- No daemon or watcher: the next instance is generated on completion (or via an
  explicit `mt recur --advance`), never by a background process.
- One file per instance stays true — each occurrence is its own `.typ` with its
  own stem and identity; the recurrence rule lives on the task, not on a mutable
  shared row.
- Typed and validated: express recurrence as a typed field (e.g.
  `recur: every(days: 7)`) validated by config; the spawned file must pass
  `mt check`.
- Decide the anchor (due-based vs completion-based) and the stop condition
  (`until` / `count`) up front.

== Scope

- Design the `recur` field shape and its config validation.
- Generate the next instance on `mt done` (and/or `mt recur --advance`): carry
  fields forward, set the new due/start, guard against runaway generation.
- Decide whether instances link (`relates-to`) or stay independent.
- Tests: weekly and monthly, completion vs due anchor, `until`/`count` end.
