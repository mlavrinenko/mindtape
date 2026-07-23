#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Give ANSI terminal renders an accessible text twin",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("www",),
  estimate: (files: 2),
  status: proposed(2026, 7, 23),
)

== Summary

The colourised terminal transcripts render as inline SVG, so a screen reader gets nothing at all. Pair each with the plain-text capture the generator already writes.

== Scope

- Add an accessible label and a visually hidden text twin to the colour terminal helper
- Source the twin from the existing plain-text capture so it cannot diverge from the image
- Style the hidden twin so it stays out of the visual layout
