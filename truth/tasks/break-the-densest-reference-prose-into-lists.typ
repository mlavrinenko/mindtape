#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Break the densest reference prose into lists",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (files: 3),
  status: done(2026, 7, 24),
)

== Summary

The configuration, import and CLI pages carry paragraphs that stack several clauses into one sentence, which reads as specification rather than documentation. Reflow the worst offenders without changing a single fact.

== Scope

- Split the densest paragraphs into lists or shorter sentences
- Change presentation only, never a claim, so nothing new can drift
- Leave the reference tables and generated content untouched
