#import "../utils/page.typ": page
#import "../utils/site.typ": u

#let meta = (
  route: "/priority/",
  label: "Priority",
  title: "Priority",
  summary: "Priority is a number on a configured scale, set by hand or scored by a framework; difficulty is its twin.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

The number is classified into labelled bands; a #strong[framework] scores typed
inputs into it. `difficulty` is the sibling field, classified by anchors instead.

= The scale and bands

```toml
[tasks.priority]
kind = "int"            # int | real
min  = 0
max  = 9
required = "!terminal"  # true | false | a filter expression

[[tasks.priority.bands]]
label = "high"
min   = 7
max   = 9
glyph = "🔴"
color = "red"
```

Bands turn a raw number into a label (`low` / `normal` / `high` in the shipped
defaults). A task may set a bare integer:

```typ
priority: 8,       // → band "high"
```

= Difficulty

`difficulty` is configured under its own `[tasks.difficulty]`, with the same
`kind`, `min`, `max` and `required` keys as priority — but classifies with sparse
labelled #strong[anchors] instead of bands. The shipped scale runs 0 to 9 with
anchors `junior` at 0, `mid` at 3, `senior` at 6 and `staff` at 9; a value
between anchors borrows the nearest lower anchor's glyph and colour. A project
overrides the range and the anchors under the same key —
#link(u("/config/"))[Configuration] carries them.

= Frameworks

A framework scores typed inputs instead of a hand-set number. Five ship baked-in:
`ice`, `rice`, `wsjf`, `eisenhower`, `moscow`. Pick the active one in config:

```toml
[tasks.priority]
framework = "ice"       # score through ICE
allow-manual = false    # strict: forbid raw integers
```

A task then calls the framework in place of a number:

```typ
priority: ice(impact: 8, confidence: 0.9, ease: 6),
```

The Rust layer scores it, bands the result, and keeps it on the scale.

= Manual versus strict

- `allow-manual = true` (default when a framework is set) — tasks may use either a
  raw integer or the framework call.
- `allow-manual = false` — every task must score through the framework; there is
  no scalar form. #link(u("/templates/"))[Templates & slots] covers how a
  scaffold fills it.

= Built-in frameworks

#table(
  columns: 2,
  table.header([Framework], [Inputs]),
  [`ice`], [`impact`, `confidence`, `ease`],
  [`rice`], [`reach`, `impact`, `confidence`, `effort`],
  [`wsjf`], [`cost_of_delay`, `job_size`],
  [`eisenhower`], [`urgent`, `important` (booleans → categories)],
  [`moscow`], [`level` (`must` / `should` / `could` / `wont`)],
)

A framework's inputs — names, types, bounds — are declared under
`[tasks.priority.frameworks.<name>]` (the baked-in five in
`crates/mindtape/src/frameworks.toml`), where its scoring and bands are
overridable too; define your own with the generic `framework(name, ..inputs)`
call. `eisenhower` and `moscow` classify into named categories rather than
banding a number.
