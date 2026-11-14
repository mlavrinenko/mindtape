#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "fix release workflow publish steps",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (files: 2),
  status: done(2026, 8, 10),
)

== Summary

The publish job in .github/workflows/release.yml still runs `cargo publish -p typst-harvest --locked`. typst-harvest is not a workspace member and comes from crates.io. That step fails the job. Drop it and its sleep, and keep mindtape -> sleep 30 -> mindtape-cli.

== Scope

- TODO
