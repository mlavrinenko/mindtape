#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rewrite open task bodies into the four-section spec",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (files: 5),
  kind: "chore",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on(
    "replace-the-task-section-set-with-a-closed-four-section-spec.typ",
  ),
)

== Summary

The union allowlist lets old and new sections coexist, but the set cannot close while an open task still carries Summary and Scope. Rewriting one is not a heading rename: Ruled out and Done when usually have to be worked out from the task and the code it names.


== Scope

Rewrite the open task bodies still carrying Summary and Scope into AS-IS, TO-BE, Ruled out and Done when, with `mt check` green after each. Five did when this was filed; the backlog has grown since, so recount at pickup and re-estimate or split then.

