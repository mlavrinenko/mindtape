#import "@local/mindtape:0.2.0": *

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

The publish job in .github/workflows/release.yml still runs \`cargo publish -p typst-harvest --locked\`, but typst-harvest is no longer a workspace member (consumed from crates.io) — the step fails the job. Drop it and its sleep, keep mindtape -> sleep 30 -> mindtape-cli.

== Scope

- TODO
