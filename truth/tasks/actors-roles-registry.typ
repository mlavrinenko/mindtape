#import "@local/mindtape:0.2.0": *

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
  status: accepted(
    2026,
    8,
    3,
  )[roles are identity-only; any tooling they should enable lives in the reader's harness, keyed by role name],
  links: depends-on("assignee-field-core.typ")[
    the assignee value references a declared actor when `[tasks.assignee]` allowed is set
  ],
)

== Summary

Multi-actor work needs named participants the config can reference by name, beyond what the assignee field alone gives. Introduce a general `[[actors]]` registry — a name plus a kind (human or llm) and the roles it may hold — and a shared `[[roles]]` table: a role is just a name, the shared dimension several actors can carry so filters can group across them. A role is identity only: whatever tooling or instructions a harness should load for a reviewer is that harness's own concern, keyed by the same role name by convention, never embedded here.

== Scope

- Config: top-level `[[actors]]` (name, kind: "human"|"llm", optional roles: ["name"]) and `[[roles]]` (name). General — not assignee-scoped, so other features can name actors.
- Validate: actor names unique and non-empty; kind from the closed set; each listed role references a declared `[[roles]]` entry; assignee values constrained to declared actors when `[tasks.assignee]` allowed is set.
- Filter integration: an equality test on the assignee's actor name; the actor's role/kind is resolved at read time from the registry (no actor object or role in the marker).
- Explicitly out of scope: a role carrying tooling, prompt, or instruction payloads. That is the reader's harness's configuration, referencing the role name by convention — keep the artifact/config declarative and the policy at the point of use.
- Tests: unique/closed-kind/undefined-role validation, assignee-to-actor constraint, filter naming an actor by role.
