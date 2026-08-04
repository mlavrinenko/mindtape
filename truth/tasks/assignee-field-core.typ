#import "@local/mindtape:0.2.0": *

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
  )[field, config, validation, env-backed splice, docs, and e2e shipped; harness session verifies],
  links: depends-on("add-slot-env-fill.typ")[
    the required assignee splice reads MINDTAPE_PRINCIPAL through the generic env slot
  ]
    + related("actors-roles-registry.typ")[
      the assignee value may reference a declared actor; the registry lands as its own task
    ]
    + related("assignee-field-manual-test.typ")[
      the harness session this task's implementation relies on
    ],
)

== Summary

A task's assignee is the actor currently working it. It is orthogonal to status: a task keeps the same assignee through proposed/accepted/wip/done, and a blocked task is still assigned to whoever unblocks and finishes it. It is not ownership — the worker is mutable and can ping-pong between actors, and saying who works it now carries no statement about who else may help (that is a routing decision for the reader, not a facet of the field). An unset `MINDTAPE_PRINCIPAL` is not a default: `MINDTAPE_PRINCIPAL` unset is an unknown actor, which cannot satisfy a required assignee, so the template splice falls to the interactive fill.

== Scope

- Core assignee field: a plain string arg in task() (lib/prelude.typ), marker shape matching kind; parse into Task (model.rs); one rendered row in the typeset card.
- `[tasks.assignee]` config: optional free-vocabulary allowed list (absent = accept any) and a required filter expression (default none).
- Required assignee is spliced through required_core_slots (add_slots.rs) like kind: a choice slot over the allowed/declared actors carrying env: `MINDTAPE_PRINCIPAL`, so mt add fills it silently under a harness that sets the variable and prompts otherwise.
- Validation: mt check flags an assignee outside allowed; a required assignee treats an unset env value as absent (unknown is not a value), so an unresolvable required field surfaces rather than silently writing an empty worker.
- Tests: parse/render, allowed-list and required rejection, the env-driven interactive prompt fallback.
