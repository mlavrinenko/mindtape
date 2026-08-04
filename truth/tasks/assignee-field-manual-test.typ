#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "assignee field manual test",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli", "substrate", "tests"),
  estimate: (files: 2),
  status: done(2026, 8, 5)[session run 2026-08-05; all four probes behaved],
  links: related("assignee-field-core.typ")[
    the implementation this session verifies
  ],
)

== Summary

Run mt add under a harness with and without MINDTAPE\_PRINCIPAL against a required \[tasks.assignee\], and mt check a task with an out-of-list assignee.

== Outcome

Real shell session against a scratch project with `[tasks.assignee] allowed = ["ada", "lin"]` and `required = "!terminal"`, using the installed `mt` binary:

- `MINDTAPE_PRINCIPAL=ada mt add` filed silently with `assignee: "ada"`; `mt check` passed.
- No principal: hard error `missing required slot(s): assignee`, no file written.
- `MINDTAPE_PRINCIPAL=eve`: hard error naming the value and the choices, no file written.
- Hand-written task with `assignee: "eve"`: `mt check` flags ``assignee: `eve` is not an allowed assignee``.
- `mt ls --json` reads the authored assignee back.

All pass. No failures.

== Scope

- TODO
