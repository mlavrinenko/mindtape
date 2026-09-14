#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Document the created and changed edge kinds",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("mt", "docs"),
  estimate: (files: 4),
  kind: "docs",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("record-the-artifacts-a-task-expects-to-touch.typ"),
)

== Summary

Two built-in edge kinds join the six the docs count by name, so every page that says six built-ins, and every edge-helper example, goes stale the moment `record-the-artifacts-a-task-expects-to-touch.typ` lands.


== Scope

Add `created` and `changed` with their glob rule to the links section of `www/content/dsl.typ`, update the built-in count in `www/content/config.typ`, add the helpers to the edge examples in `skills/mindtape/references/hand-editing.md`, and add a CHANGELOG.md entry.

