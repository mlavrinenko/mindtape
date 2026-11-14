#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "bump workspace version and cut the changelog section",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 2),
  status: done(2026, 8, 10),
)

== Summary

Bump the workspace version in Cargo.toml and move the CHANGELOG \[Unreleased\] content into a dated section. The next release number needs a decision from the user. The outdatty release-notes group drives CHANGELOG.md re-confirmation.

== Scope

- TODO
