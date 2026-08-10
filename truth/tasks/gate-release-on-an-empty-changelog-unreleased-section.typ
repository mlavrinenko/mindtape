#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "gate release on an empty changelog unreleased section",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 2),
  status: done(2026, 8, 10),
)

== Summary

just release must refuse to tag while CHANGELOG.md has a non-empty \[Unreleased\] section — the section has to move into a dated \[X.Y.Z\] first.

== Scope

- TODO
