#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Numeric field primitive: scale, classification, display",
  status: proposed(2026, 6, 19),
  priority: "high",
  estimate: (days: 1),
  depends-on: (link("field-legend-glyphs.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Shared substrate for the numeric scalar fields (priority, difficulty) — the
numeric cousin of the legend primitive. It owns the numeric value machinery and
the classification layer so neither priority nor difficulty reinvents either, and
so neither has to depend on the other.

== Scope

- Scale config: `kind` (int | real), `min`, `max`. Numeric model storage, numeric
  filter comparison, and numeric sort — fixing today's lexicographic mis-sort
  (high < low < normal) generically.
- Classification of the scale into labelled cells carrying glyph + colour, in two
  styles:
  - bands — contiguous ranges tile `[min, max]`; full-coverage and no-overlap
    validation (int: inclusive, contiguous; real: half-open, closed last).
  - anchors — sparse labelled points; a value between anchors inherits the nearest
    lower anchor's glyph + colour, label derived as `lower-label + "+"×offset`
    (require an anchor at `min`; values below all anchors derive upward with `−`).
- Display through the `Field` trait from field-legend-glyphs. DSL label sugar maps
  an anchor label (and a band's representative value) back to a number.

== Boundary

Instances (priority, difficulty) pick a scale and a style and ship their own
defaults. Priority frameworks are a separate layer above this primitive.
