#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Rebuild www as a tola Typst docs site",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 4,
  ),
  tags: (),
  status: done(2026, 7, 6),
)

== Summary

Replace the single hand-written www/index.html landing page with a proper multi-page documentation site built with tola-ssg (Typst-native SSG). Audience: CLI/plaintext power users plus AI agents. Editorial + monospace design, calm and concise, three-faces-of-a-task hero (source, terminal table, typeset artifact). The docs use Typst and dogfood the project's own medium. Search (pagefind) and build-time mt inputs deferred to follow-ups.

== Scope

- TODO
