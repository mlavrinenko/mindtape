#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Render an empty list slot as bracket-pair in the mt add scaffold",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 2),
  status: wip(2026, 7, 23),
)

== Summary

The interactive \`mt add\` YAML scaffold renders a list-kind slot with an empty or absent default as a blank scalar (\`tags:\`), which reads as an unfilled value rather than an empty collection. Render it as an explicit empty flow sequence (\`tags: \[\]\`) so the list kind is visible and the fill affordance is unambiguous.

== Scope

- \`crates/mindtape-cli/src/add\_interactive.rs\`: render empty-default list slots as \`\[\]\`.
