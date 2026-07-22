#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "task limits",
  priority: ice(
    impact: 8,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli", "gating"),
  status: proposed(2026, 7, 22),
)

== Summary

Let a project cap how many tasks may sit in a matched set at once, and have `mt check` report a breach. Bounded work-in-progress is the one Kanban mechanic with evidence behind it for solo work, and this is the mechanism that stops a session ending on an open leaf task.

Nothing about a limit may hardcode a status value. The legend is project-configured; a project may name its in-flight state anything, or have several. The matched set is a filter expression, so the same primitive caps a status, a view, a priority band or a `kind`.

== Scope

- New `[[tasks.limits]]` table: `filter` (a filter expression), `max` (int), optional `message`, optional `severity`. No status name appears in Rust.
- No `view` key. `in_view("name")` already selects a view from inside a filter, and one mechanism beats two spellings of one idea.
- `severity` is `"warn"` (default) or `"error"`. A breach is usually a state of the backlog rather than a malformed artifact, and it clears by finishing work rather than by editing a file — so warning is the right default. But a project that wants a hard stop should get one, and MindTape's own backlog wants exactly that.
- `mt check` evaluates each limit over the loaded set and reports the limit, the count, the cap and the offending tasks. `message` overrides the default text; the default should name the action to take, not restate the condition.
- Pin the missing-field comparison semantics this depends on. `compare` in `crates/mindtape/src/filter.rs` currently returns true only for `Neq` when a field is absent, so `kind != "umbrella"` matches a task carrying no `kind` at all. The umbrella rule below rests on that, and no test currently holds it in place. Document it and add one.
- Scope is `mt check` only. Surfacing counts in `mt ls` is a display concern and is excluded here.
- MindTape adopts it on its own backlog, as the structural fix for sessions that end mid-task:
  - `status == "wip" && kind == "umbrella"`, max 1, severity error.
  - `status == "wip" && kind != "umbrella"`, max 1, severity error.
  An umbrella coordinates a milestone and may legitimately stay open across sessions; a leaf may not. One of each is the rule, and the second filter catches every task with no `kind` by construction.
- Tests: under, at and over the cap; a limit whose filter matches nothing; two limits at once; `severity = "error"` fails the command where `"warn"` does not; a project whose in-flight status is not named `wip`; a task with no `kind` counts against a `kind != "..."` limit.
