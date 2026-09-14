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
  status: proposed(2026, 9, 14),
  links: depends-on("write-the-mindtape-plain-prose-rule-pack.typ"),
)

== Summary

Rule packs and the plain-prose pack ship as code and tests. The generated schema does not yet accept the `include` and disable keys on `[tasks.body.content]`, and the config page does not say a pack exists or how to drop one rule from it.


== Scope

Regenerate `schema/config.schema.json`, document the `include` and per-rule disable keys, the reserved `mindtape-` prefix and the rule list of `mindtape-plain-prose` in `www/content/config.typ`, and add a CHANGELOG.md entry.

