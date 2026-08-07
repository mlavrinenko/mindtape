#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "actors and roles registry manual test",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("cli", "substrate", "tests"),
  estimate: (files: 1),
  status: done(2026, 8, 7)[session run 2026-08-07; every probe passed],
  links: related("actors-roles-registry.typ")[
    the implementation this session verifies
  ]
    + related("cover-the-actors-filters-end-to-end.typ")[
      the coverage gap this session surfaced
    ],
)

== Summary

Verify the registry end to end in a real session: filters resolve an assignee's declared roles at read time, and validation rejects actors, roles, and assignees outside the registry.

== Outcome

Session run 2026-08-07 against a scratch project under `/tmp`, scaffolded with
`mt init --no-input` and extended with `[[roles]]` (`reviewer`, `author`) and
`[[actors]]` (`ada` human holding `reviewer`, `gpt` llm holding `author`). Five
tasks: two assigned `ada`, one `gpt`, one unassigned, one assigned to the
undeclared `eve`.

- `mt ls 'role == "reviewer"'` listed exactly the two `ada` tasks.
- `mt ls 'role == "author"'` listed exactly the one `gpt` task.
- `mt ls 'assignee == "gpt"'` matched by actor name — same single row.
- `eve` never appeared under any `role ==` filter: an undeclared assignee
  resolves no roles rather than falling through to a match.
- With no `[tasks.assignee]` `allowed`, the `eve` task passed `mt check` as free
  text (`ok: 5 task(s) valid`); with `allowed = ["ada", "gpt"]` it failed with
  ``eve-one: assignee: `eve` is not an allowed assignee``.
- An actor holding an undeclared role failed at config load:
  ``actors.lin: role `approver` is not a declared role``.

Three extra registry probes, all rejected at load as intended: an `allowed`
entry naming a non-actor (``tasks.assignee.allowed: `mallory` is not a declared
actor``), a duplicate actor name, and `kind = "robot"` (``unknown variant
`robot`, expected `human` or `llm```).

All probes pass. This session was the first exercise of the `role` and
`assignee` filters through the real CLI — they had only library-level unit tests
against a hand-built `Config`. Follow-up filed:
`cover-the-actors-filters-end-to-end.typ`.

== Scope

- Scratch project declaring `[[roles]]` and `[[actors]]`, tasks assigned across
  them, and one assigned to an undeclared name.
- `role ==` / `assignee ==` filters through `mt ls`.
- `mt check` against an out-of-registry assignee, with and without
  `[tasks.assignee]` `allowed`.
- Config-level registry validation: undeclared role, non-actor in `allowed`,
  duplicate names, unknown actor kind.
