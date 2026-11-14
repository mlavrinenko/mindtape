#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "actors and roles registry",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("substrate", "tooling"),
  estimate: (files: 3),
  status: done(2026, 8, 5)[
    config, validation, assignee constraint, role filters, schema and docs landed. A harness session verifies
  ],
  links: depends-on("assignee-field-core.typ")[
    the assignee value references a declared actor once `[tasks.assignee]` allowed has a value
  ]
    + related("actors-roles-registry-manual-test.typ")[
      session verifies the registry end to end
    ],
)

== Summary

Multi-actor work needs named participants the config can reference by name,
beyond what the assignee field alone gives. Introduce a general `[[actors]]`
registry (a name, a kind of `human` or `llm`, and the roles it may hold) and a
shared `[[roles]]` table. A role is just a name: the shared dimension multiple
actors can carry so filters can group across them. A role carries identity only.
Whatever tooling or instructions a harness should load for a reviewer is that
harness's own concern, keyed by the same role name by convention, never embedded
here.

== Scope

- Config: top-level `[[actors]]` (name, `kind` of `"human"` or `"llm"`, optional
  `roles`) and `[[roles]]` (name). General, not assignee-scoped, so other
  features can name actors.
- Validate the registry. Actor names must be unique and non-empty, and the kind
  must come from the closed set. Each listed role must reference a declared
  `[[roles]]` entry, and assignee values must name declared actors once
  `[tasks.assignee]` allowed has a value.
- Filter integration: an equality test on the assignee's actor name. Resolve the
  actor's role and kind at read time from the registry, with no actor object or
  role in the marker.
- Explicitly out of scope: a role carrying tooling, prompt or instruction
  payloads. That is the reader's harness configuration, referencing the role name
  by convention. Keep the artifact and config declarative, and the policy at the
  point of use.
- Tests: unique-name, closed-kind and undefined-role validation, the
  assignee-to-actor constraint, and a filter that groups actors by role.
