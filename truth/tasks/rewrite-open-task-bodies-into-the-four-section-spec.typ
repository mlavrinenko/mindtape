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
  estimate: (files: 2),
  kind: "dev",
  assignee: "agent",
  status: done(
    2026,
    9,
    15,
  )[Every open task was rewritten, superseded or closed in the six-section shape in the same change as the rules.],
  links: depends-on(
    "replace-the-task-section-set-with-a-closed-four-section-spec.typ",
  )
    + depends-on("add-a-docs-page-showing-a-rejected-agent-spec.typ"),
)

== Summary

The union allowlist lets old and new sections coexist, but the set cannot close while an open task still carries Summary and Scope. Rewriting one is not a heading rename: Ruled out and Done when usually have to be worked out from the task and the code it names.


== Scope

Rewrite the open task bodies still carrying Summary and Scope into AS-IS, TO-BE, Ruled out and Done when, with `mt check` green after each. Closed tasks are never rewritten. This runs after every other open task, so at pickup the old shape should survive only in the four-section chain's own open tasks and in anything filed since: recount then, and re-estimate or split if that count has grown.

== Dogfood

Built in: the rewrite is this repo's own backlog.
