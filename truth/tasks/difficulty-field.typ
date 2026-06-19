#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Difficulty as a numeric-field instance, replacing grade",
  status: done(2026, 6, 19),
  priority: ice(impact: 8, confidence: 0.8, ease: 9),
  estimate: (hours: 4),
  depends-on: (link("numeric-field.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Replace the `grade` enum slot (speconaut's `grade: "senior"`) with a numeric
`difficulty` field — a numeric-field instance in anchor style, no framework layer.
Default scale 0..9. Generalises skill and complexity into one ordered axis; the
difficulty→model-tier mapping stays consumer (speconaut) policy, keeping mindtape
model-agnostic.

== Scope

- Rename the core slot `grade` → numeric `difficulty` across the prelude
  (`_core-slots`), model, config, validation, filter, and json output. Breaking,
  acceptable while 0.2 is pre-release.
- Adopt the numeric-field primitive with anchor classification (the derivation
  rule lives in numeric-field): a value at an anchor shows its label; between
  anchors it derives `lower-label + "+"×offset` and inherits the lower anchor's
  glyph + colour.
- Ship default anchors junior(0), mid(3), senior(6), staff(9), each with a glyph
  and colour.

== DSL

`difficulty: senior` resolves to that anchor's value; `difficulty: 7` is the raw
point and derives `senior+`.
