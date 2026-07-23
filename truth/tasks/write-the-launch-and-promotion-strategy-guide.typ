#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Write the launch and promotion strategy guide",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 8,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (files: 2),
  status: done(2026, 7, 23),
)

== Summary

Promotion for a low-budget hobby project needs a written plan rather than improvisation at launch. Capture a zero-cost playbook as a Typst document under var for later work.

== Scope

- Cover the free channels: Show HN, the Rust and command-line communities, the Typst community, Mastodon, Lobsters
- Cover the durable ones: awesome lists, This Week in Rust, a launch write-up
- Note the prerequisites that gate a launch, including a demo recording and a social preview card
- Keep it as a Typst document under var, outside the docs site
