#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "de-slop the contributor deep dives",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("docs", "tooling"),
  estimate: (files: 7),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ"),
  status: accepted(2026, 8, 16),
)

== Summary

`docs/src/contributing.typ` and the six deep dives behind it total 1082 lines
of source, much of it arguing decisions at a length that made sense while the
decision was being made and no longer serves a reader who has to follow the
process. They also restate each other: the gate list appears in
`contributing.typ` and again in `contributing/gates.typ`, and coverage
thresholds appear in two of the six.

Tighten them, and fold each duplicated fact into the one place that derives it.

== Scope

`docs/src/contributing.typ` and `docs/src/contributing/`: `gates.typ`,
`generated-docs.typ`, `coverage-crap.typ`, `mutation-testing.typ`,
`signal-sweep.typ`, `windows.typ`.

== Rules

Every gate name, recipe, threshold, flag and file path stays. The `#just.*`
calls and the `fact(...)` reads stay — those are the anti-drift machinery, not
prose. What goes: the second telling of a rule, the paragraph that explains
why an obvious thing is obvious, and hedged advice that never resolves into an
instruction. This task does not touch `docs/src/readme.typ`, `agents.typ`,
`skill.typ` or `lib.typ`.
