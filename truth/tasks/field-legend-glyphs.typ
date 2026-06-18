#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Legend: discrete enum field display",
  status: proposed(2026, 6, 19),
  priority: "normal",
  estimate: (hours: 4),
  depends-on: (link("field-display.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Configure and display the discrete enum fields (status, kind, tags) through one
`Legend` data type, implementing the `Field` trait from field-display. A `Legend`
is an ordered list of entries; each binds a value to an optional glyph, colour,
and label, and carries field-specific flags (status: `terminal`). The same legend
defines the legal value set, its order, and its display — folding `status.values`
+ `status.terminal` + `fields.<name>.values` into one table.

== Scope

- `Legend` type: ordered entries `{ value, glyph?, color?, label?, .. }`. A plural
  `value` lets one glyph cover several values; lookup stays unambiguous.
- `Field` impls for `Status` / `Kind` / `Tags` over the legend.
- Config shape: `[[tasks.status]]` / `[[tasks.kind]]` / `[[tasks.tags]]` entry
  arrays. Migrate `.mindtape/config.toml` and default-config.toml.
