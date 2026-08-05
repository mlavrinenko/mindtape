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
  status: accepted(2026, 8, 5)[],
  links: related("actors-roles-registry.typ")[
    the implementation this session verifies
  ],
)

== Summary

Verify the registry end to end in a real session: filters resolve an assignee's declared roles at read time, and validation rejects actors, roles, and assignees outside the registry.

== Outcome

Session not yet run. Probes:

- Scratch project declaring `[[roles]]` (reviewer, author) and `[[actors]]` (ada human with reviewer, gpt llm with author).
- `mt ls 'role == "reviewer"'` lists tasks assigned to ada and none else.
- `mt ls 'assignee == "gpt"'` matches by actor name.
- Task assigned to an undeclared name passes `mt check` (free text) and fails when `[tasks.assignee] allowed` is set.
- Config with an actor referencing an undeclared role fails `mt check`.

== Scope

- TODO
