#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Priority bands with glyph and colour",
  status: proposed(2026, 6, 19),
  priority: "normal",
  estimate: (hours: 4),
  depends-on: (link("numeric-priority-scale.typ"),),
  relates-to: (link("tty-render-discipline.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Classify the numeric priority into named bands for display and filtering. Bands
tile the whole `[min, max]` scale; mindtape validates full coverage and no
overlap at config load (integer scale: inclusive and contiguous; real scale:
half-open with a closed last band). Each band carries a label, glyph, and colour.

== Scope

- `[[tasks.priority.band]]` entries: range, label, glyph, colour.
- Coverage and overlap validation at config load.
- Band label usable in filters (`priority:high` means "in the high band") and
  shown in the list.
- Glyph and colour rendered through the field legend.
