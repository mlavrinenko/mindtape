#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Generate the config reference from the schema",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("www", "docs"),
  estimate: (files: 4),
  status: proposed(2026, 7, 23),
)

== Summary

The configuration page enumerates sections by hand, which already drifted: it over-attributes add-only keys to the go section and omits several live ones. Generate the enumerated tables from the captured JSON Schema, the way the CLI page is generated from the command tree.

== Scope

- Capture the config schema into the generated directory during the docs build
- Build the sections table from the schema, using each description as the purpose column
- Add a gate asserting every schema section appears on the page, mirroring the CLI gate
- Keep prose and examples hand-written, generate only the enumerations
