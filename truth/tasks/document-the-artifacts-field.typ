#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Document the artifacts field",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("mt", "docs"),
  estimate: (files: 3),
  kind: "docs",
  assignee: "agent",
  status: cancelled(
    2026,
    9,
    15,
  )[Backlog review 2026-09-15: git already records the files a task touched, and nothing would consume a declared list.],
  links: depends-on("read-the-artifacts-field-into-the-task-model.typ"),
)

== Summary

Once `read-the-artifacts-field-into-the-task-model.typ` lands, `artifacts` is a core task field that only its one-line entry in `www/utils/dsl-field-notes.typ` describes: the DSL page shows no `created` or `changed` example, the hand-editing reference never uses it, and the changelog is silent.

== Scope

Add the `artifacts` field to `www/content/dsl.typ`: the `created` and `changed` packagers, globs accepted on `changed` and refused on `created`, and why no path is ever checked against the tree. Add it to the worked example in `skills/mindtape/references/hand-editing.md`, and add a CHANGELOG.md entry under Unreleased.

== Dogfood

Not applicable: docs only; the field is dogfooded in `compare-declared-artifacts-against-the-working-tree.typ`.
