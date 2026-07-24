#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Generate the favicon and touch icon from the logo",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("www", "tooling"),
  estimate: (files: 4),
  status: done(2026, 7, 24),
)

== Summary

The favicon is the raw SVG with no raster fallback, and a hand-maintained second copy would drift from the logo. Generate the raster icons from the single SVG source during the docs build.

== Scope

- Render a PNG favicon and an apple touch icon from the logo during the docs build
- Emit the matching link tags
- Add the rasteriser to the dev shell since it joins the build loop
- Keep one SVG as the only hand-maintained source
