#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Document named rule packs",
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
  )[Superseded on 2026-09-15 by give-every-body-rule-a-name, let-a-config-extend-rule-packs, ship-the-task-practice-as-the-mindtape-tasks-pack and document-rule-packs, which add path packs, disable and same-name overrides.],
  links: depends-on("write-the-mindtape-plain-prose-rule-pack.typ"),
)

== Summary

Rule packs and the plain-prose pack ship as code and tests. The generated schema does not yet accept the `include` and disable keys on `[tasks.body.content]`, and the config page does not say a pack exists or how to drop one rule from it.


== Scope

Regenerate `schema/config.schema.json`, document the `include` and per-rule disable keys, the reserved `mindtape-` prefix and the rule list of `mindtape-plain-prose` in `www/content/config.typ`, and add a CHANGELOG.md entry.

== Dogfood

Not applicable: docs only; the pack itself is dogfooded in `write-the-mindtape-plain-prose-rule-pack.typ`.
