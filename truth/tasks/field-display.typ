#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Field display: the Field trait and generic cell rendering",
  status: done(2026, 6, 19),
  priority: ice(impact: 10, confidence: 0.9, ease: 10),
  estimate: (hours: 4),
  depends-on: (link("tty-render-discipline.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

The seam every displayable task field renders through, sitting between the
terminal mechanics (tty-render-discipline) and the field families. It defines the
`Field` trait and the generic cell rendering — resolving a field's value to text,
glyph, and colour and laying it out width-aware — so the discrete (legend) and
numeric (scale + classification) families plug in without depending on each other.

== Scope

- `Field` trait: `NAME`, `values(task)`, and the per-field config lookup (legend
  or classification) yielding a value's label, glyph, and colour.
- Generic cell rendering: value → `{ text, glyph, colour }`, gated and measured
  through tty-render-discipline. Text by default, glyph as adornment, glyph-only a
  density mode.
- Unit impls live in the family tasks (field-legend-glyphs, numeric-field); this
  task owns only the trait and the rendering.
