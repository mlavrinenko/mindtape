#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "push main from just release",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 2),
  status: done(2026, 8, 10),
)

== Summary

just release pushes only the v-tag; GitHub main is 764 commits behind and pages.yml deploys the docs site only on main pushes. Push origin main before tagging.

== Scope

- TODO
