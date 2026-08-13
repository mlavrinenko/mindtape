#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "trim root docs by delegating deep content",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (files: 6),
  status: wip(2026, 8, 13),
)

== Summary

README.md, CONTRIBUTING.md and AGENTS.md restate reference material that already exists in www/content (14 Typst pages) and docs/contributing. Cut each to what its audience needs in nearly every session and point at the existing deep content by local path. Also fixes AGENTS.md Key Docs, which names docs/decisions/ and docs/redesign/ — both deleted in 0875de2.

== Scope

- AGENTS.md 178 -> ~90: drop Templates, links, recur, derived-tags deep dives (covered by www/content/{templates,dsl,derived-tags}.typ and skills/mindtape/SKILL.md); keep workspace map, architecture pins, DSL skeleton, prelude-vs-Rust split
- CONTRIBUTING.md 239 -> ~135: move the coverage/CRAP reasoning (lines 106-158) to docs/contributing/coverage-crap.md, same pattern as signal-sweep.md
- README.md 182 -> ~120: platform support, performance table and architecture move into www/content
- fix the two dead docs/ references in AGENTS.md and CONTRIBUTING.md
