#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rebuild www as a tola Typst docs site",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 4,
  ),
  tags: (),
  status: wip(2026, 7, 6),
)

== Summary

Replace the single hand-written www/index.html landing page with a proper multi-page documentation site built with tola-ssg (Typst-native SSG). Audience: CLI/plaintext power users plus AI agents. Editorial + monospace design, calm and concise, three-faces-of-a-task hero (source, terminal table, typeset artifact). Docs authored in Typst; dogfoods MindTape's own medium. Search (Pagefind) and build-time mt inputs deferred to follow-ups.

== Scope

- TODO
