#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "assignee field",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 4,
  tags: ("cli", "substrate"),
  estimate: (files: 4),
  status: done(
    2026,
    8,
    5,
  )[field, config, validation, env-backed splice, docs and e2e landed. A harness session verifies],
  links: depends-on("add-slot-env-fill.typ")[
    the required assignee splice reads `MINDTAPE_PRINCIPAL` through the generic `env` slot.
  ]
    + related("actors-roles-registry.typ")[
      the assignee value may reference a declared actor, and the registry lands as its own task.
    ]
    + related("assignee-field-manual-test.typ")[
      the harness session this task's implementation relies on.
    ],
)

== Summary

A task's assignee is the actor currently working it. The field is orthogonal to
status: a task keeps the same assignee through proposed/accepted/wip/done, and a
blocked task still belongs to whoever unblocks and finishes it. It is not
ownership, because the worker is mutable and can ping-pong between actors.
Saying who works it carries no statement about who else may help, which is a
routing decision for the reader, not a facet of the field. An unset
`MINDTAPE_PRINCIPAL` is not a default: it marks an unknown actor, which cannot
satisfy a required assignee. The template splice then falls to the interactive
fill.

== Scope

- Core assignee field: a plain string `arg` in `task()` (`lib/prelude.typ`), a
  marker shape matching `kind`, parsing into `Task` (`model.rs`), and one
  rendered row in the typeset card.
- `[tasks.assignee]` config: an optional free-vocabulary `allowed` list (absent
  accepts any) and a required filter expression (default none).
- Splice a required assignee through `required_core_slots` (`add_slots.rs`) like
  `kind`: a choice slot over the allowed or declared actors, carrying
  `env: MINDTAPE_PRINCIPAL`. `mt add` then fills it silently under a harness that
  sets the variable and prompts otherwise.
- Validation: `mt check` flags an assignee outside `allowed`. A required assignee
  treats an unset `env` value as absent (unknown is not a value), so an
  unresolvable required field surfaces instead of silently writing an empty
  worker.
- Tests: parse and render, allowed-list and required rejection, and the
  env-driven interactive prompt fallback.
