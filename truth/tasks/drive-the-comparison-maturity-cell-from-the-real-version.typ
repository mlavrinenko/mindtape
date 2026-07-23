#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Drive the comparison maturity cell from the real version",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("www",),
  estimate: (files: 3),
  status: proposed(2026, 7, 23),
)

== Summary

The comparison page hardcodes 'Pre-1.0, unreleased' for MindTape, which goes stale the moment a tag lands. Read the version from the generated capture instead, and colour only the objective capability rows as a neutral heatmap.

== Scope

- Read the version from www/generated/version.txt so the maturity cell cannot drift
- Drop the editorialising wording in favour of the real version plus licence
- Pastel green, amber and grey applied by capability to every column including MindTape
- Leave descriptive rows such as on-disk format and identity uncoloured, since they carry no better or worse
