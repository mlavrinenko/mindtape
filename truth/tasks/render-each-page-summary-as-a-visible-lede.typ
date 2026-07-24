#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Render each page summary as a visible lede",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("www",),
  estimate: (files: 3),
  status: done(2026, 7, 24),
)

== Summary

Every page already carries a written summary that is used only for the meta description. Rendering it under the heading gives each page a scannable one-line opening with no new copy to write and nothing to keep in sync.

== Scope

- Render the existing summary as a lede under the page heading
- Keep the same string feeding the meta description so the two cannot diverge
- Style it as a subdued subtitle in both themes
