#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Wire the cassette logo mark",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("www",),
  estimate: (files: 1),
  status: done(2026, 7, 23),
)

== Summary

Replace the 0.1.0 neon brain-and-tape logo with a monochrome-first cassette mark carrying a tape strip on top, so the mark matches the site's editorial accent palette and survives the header's flat CSS mask.

== Scope

- Redraw www/assets/images/logo.svg as one evenodd path, single colour
- Keep the meaning in the outline so the mask and favicon both read at 16 to 26px
- Verify it renders at 120, 26 and 16px and as a flat silhouette
