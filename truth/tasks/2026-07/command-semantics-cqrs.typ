#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Command semantics: read/write separation (CQRS)",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 8,
  ),
  tags: (),
  status: done(2026, 7, 10),
)

== Summary

Codify a strict read/write split across the `mt` surface: read commands never
mutate the working tree. Only explicit write commands do. Recurrence, in
particular, never materializes as a side effect of a read. This is the single
structural lesson from Taskwarrior, whose recurrence generates instances lazily
as a side effect of running a report (`task list`). That habit is the root of
its `mask` drift, multi-client duplicate races, and single-writer-client workaround. A
daemon-free tracker sidesteps that whole bug class by construction, but only if
we write the invariant down and test it.

== Decisions

- Read (pure): `ls`, `check`, `show`, `export`, and `config` queries. They leave
  the tree byte-identical.
- Write (explicit): `add`, `recur`, `done`, `flip`, `rm`, `init`.
- `mt done` closes a task and never creates or advances anything. On a recurring
  task it prints a notice pointing at `mt recur`, and does not strip the `recur`
  field. So `mt recur` stays callable on the done task.
- `mt recur` is the only command that spawns or advances a recurrence.

== Constraints

- No read command may open a file for writing. Enforce with a regression test:
  hash the fixture tree before and after a read command, and assert it unchanged.

== Scope

- Add a "Command Semantics (read/write separation)" section to
  `CONTRIBUTING.md` with this exact content:

  #raw(
    block: true,
    lang: "markdown",
    "## Command Semantics (read/write separation)

MindTape follows a strict read/write split (CQRS). Read commands never mutate
the working tree; only explicit write commands do.

- Read (pure): `ls`, `check`, `show`, `export`, `config` queries — leave the tree
  byte-identical (enforced by test).
- Write (explicit): `add`, `recur`, `done`, `flip`, `rm`, `init`.

Recurrence never materializes as a side effect of a read. An instance is spawned
or refreshed only by an explicit `mt recur`; `mt done` closes a task and never
creates or advances anything.",
  )

- Regression test: a read command over a fixture leaves the tree byte-identical.
- The recurring-tasks task implements the `mt done` notice and the
  recur-on-done behaviour. This task owns only the principle, the doc, and the test.
