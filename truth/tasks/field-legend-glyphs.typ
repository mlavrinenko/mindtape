#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Legend primitive and Field trait for displayable fields",
  status: proposed(2026, 6, 19),
  priority: "normal",
  estimate: (days: 1),
  relates-to: (link("list-plaintext-render.typ"), link("tty-render-discipline.typ")),
  tags: ("substrate", "cli"),
)

== Summary

Introduce one display primitive for the discrete enum fields and a single `Field`
trait with one impl per field, replacing ad-hoc per-field handling. A `Legend` is
an ordered list of entries; each entry binds a value to an optional glyph, colour,
and label, and carries field-specific flags (status: `terminal`). The same legend
defines the legal value set, its order, and its display — folding `status.values`
+ `status.terminal` + `fields.<name>.values` into one table.

== Scope

- `Legend` type: ordered entries `{ value, glyph?, color?, label?, .. }`. A plural
  `value` lets one glyph cover several values; lookup must stay unambiguous.
- One `Field` trait (`NAME`, `values(task)`, `legend(cfg)`) with unit impls
  `Status` / `Kind` / `Tags`. Priority and difficulty are the numeric cousins
  (scale + classification) — same trait for rendering, different config.
- Config shape: `[[tasks.status]]` / `[[tasks.kind]]` / `[[tasks.tags]]` entry
  arrays. Migrate `.mindtape/config.toml` and default-config.toml.
- Render keeps the value text by default; glyph is adornment. Glyph-only is a
  density mode, not the default — grep, no-emoji terminals, and colourblind
  readers still need the word.

== Boundary

Colour resolution and TTY gating live in tty-render-discipline; legends only
declare named or `#RRGGBB` colours.
