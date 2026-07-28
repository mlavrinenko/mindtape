#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Run the rrule feature gate in CI",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("gating",),
  estimate: (files: 1),
  status: done(
    2026,
    7,
    28,
  )[ci.yml runs just check-rrule; verified green locally],
)

== Summary

just check-rrule says it reproduces what CI runs for the rrule escape hatch, but no workflow mentions rrule: rg rrule .github/workflows/ is empty. The feature is off by default, so nothing anywhere compiles crates/mindtape/src/recur\_rrule.rs, and it can rot until someone enables it. The recipe comment was aspirational.

== Scope

- Add an rrule step to the CI check job, so the recipe comment becomes true.
- Keep it out of just check, which the recipe already explains: default features only.
