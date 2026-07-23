#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Document schema_version truthfully",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (files: 2),
  status: proposed(2026, 7, 23),
)

== Summary

The schema and agents pages show a schema\_version JSON snippet that no real output contains, and the full-record example omits it. Re-document both against the real envelope once the binary emits it.

== Scope

- Update www/content/schema.typ to show the field in the envelope, not as a standalone object
- Update www/content/agents.typ envelope description
- Rely on the generated capture so the pages cannot drift again
