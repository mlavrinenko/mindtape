#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "task limits",
  priority: ice(
    impact: 6,
    confidence: 0.75,
    ease: 6,
  ),
  tags: (),
  status: proposed(2026, 7, 22),
)

== Summary

Let a project cap how many tasks may sit in a given state at once, and have `mt check` report a breach. Motivation: bounded work-in-progress is the one Kanban mechanic with evidence behind it for solo work, and it is the precondition that makes a per-task change budget attributable — if only one task is in flight, the working-tree diff belongs to that task. Nothing about the limit may hardcode the status value `wip`: the status legend is project-configured and a project may name its in-flight state anything, or have several.

== Scope

- New `[[tasks.limits]]` table: `filter` (the existing filter language, so the matched set is arbitrary — a status, a view, a priority band), `max` (int), optional `message`. No status name appears in Rust.
- Accept `view = "name"` as an alternative to `filter`, reusing view resolution.
- `mt check` evaluates each limit over the loaded set and reports a breach naming the limit, the count, the cap, and the offending tasks.
- Always a warning, and no per-limit severity key. A breach is a state of the backlog, not a malformed artifact: it is cleared by finishing work, not by editing a file, so failing a commit hook over it would block the only action that fixes it.
- Scope is `mt check` only. Surfacing counts in `mt ls` is a separate display concern and is deliberately excluded here.
- Tests: under, at, and over the cap; a limit whose filter matches nothing; two limits at once; a project whose in-flight status is not named `wip`.
