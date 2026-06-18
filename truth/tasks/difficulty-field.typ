#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Difficulty: numeric field replacing grade, with anchor labels",
  status: proposed(2026, 6, 19),
  priority: "normal",
  estimate: (hours: 4),
  relates-to: (link("priority-bands.typ"), link("numeric-priority-scale.typ")),
  tags: ("substrate", "cli"),
)

== Summary

Replace the `grade` enum slot (speconaut's `grade: "senior"`) with a numeric
`difficulty` field — the same scale + classification primitive as priority, minus
the framework layer. Default scale 0..9. Generalises skill and complexity into one
ordered axis; the difficulty→model-tier mapping stays consumer (speconaut) policy,
keeping mindtape model-agnostic.

== Scope

- Rename the core slot `grade` → numeric `difficulty` across the prelude
  (`_core-slots`), model, config, validation, filter, and json output. Breaking,
  acceptable while 0.2 is pre-release.
- `[tasks.difficulty]` scale (`kind` / `min` / `max`, default 0..9) reusing the
  numeric-field primitive shared with priority.

== Classification: sparse anchors with derivation

Difficulty labels its scale by anchors, not bands. An anchor is a labelled point
carrying a glyph and colour. A value at an anchor shows its label; a value between
anchors inherits the nearest lower anchor's glyph and colour, with a label derived
as `lower-label + "+"×offset` (so `7` under senior@6 reads `senior+`). Require an
anchor at `min`; a value below all anchors derives upward with `−`. Bands (full
coverage ranges) stay priority's style — two styles of one classification
primitive; build it once and let either field pick either style.

== DSL

`difficulty: senior` resolves to that anchor's value; `difficulty: 7` is the raw
point and derives `senior+`. Ship default anchors junior(0), mid(3), senior(6),
staff(9), each with a glyph and colour.
