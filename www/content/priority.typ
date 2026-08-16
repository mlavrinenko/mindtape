#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u

#metadata((
  route: "/priority/",
  label: "Priority",
  title: "Priority",
  summary: "Priority is a number on a configured scale, set by hand or scored by a framework; difficulty is its twin.",
)) <page-meta>

#show: page.with(active: "/priority/")

The number is classified into labelled bands; a #strong[framework] scores typed
inputs into it. `difficulty` is the sibling field with the same shape.

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

The Rust layer scores it (here `round(impact * confidence * ease * 0.09)` = 4),
bands the result, and keeps it on the scale. No math happens in Typst — the call
shapes a `(framework, inputs)` dict, nothing more.

= Manual versus strict

- `allow-manual = true` (default when a framework is set) — tasks may use either a
  raw integer or the framework call.
- `allow-manual = false` — every task must score through the framework. There is
  no scalar form, so when the template omits priority `mt add` splices the
  framework call itself: one slot per input, filled leaf by leaf
  (`--set impact=8 --set confidence=0.9 --set ease=6`).

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

Override a framework's scoring or bands under
`[tasks.priority.frameworks.<name>]`, or define your own with the generic
`framework(name, ..inputs)` call. `eisenhower` and `moscow` classify into named
categories rather than banding a number.

#callout("note")[
  Every scored result lands back on the numeric scale and is in-bounds by
  construction, so priority stays comparable in #link(u("/filters/"))[filters]
  (`priority >= 7`) regardless of how it was set.
]
