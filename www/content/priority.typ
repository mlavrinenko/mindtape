#import "../utils/page.typ": page
#import "../utils/site.typ": u
#import "../utils/framework-notes.typ": (
  band-example, difficulty-anchors, framework-call, framework-table,
  priority-bands, scale-config, scale-span,
)

// Every mechanical fact on this page — the framework list, each one's inputs
// and their bounds, the score formulas, the shipped bands and difficulty
// anchors — comes from www/generated/frameworks.json
// (crates/mindtape/src/frameworks.toml + defaults.toml, via
// www/generate-facts.sh). This page owns the presentation; the per-framework
// prose lives in framework-notes.typ beside the renderers, and `just
// check-doc-facts` fails the build if a framework here has none.
#let fw = json("../generated/frameworks.json")

#let meta = (
  route: "/priority/",
  label: "Priority",
  title: "Priority",
  summary: "How priority and difficulty work as numbers on a configured scale, set by hand or scored by a framework.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

The number is classified into labelled bands; a #strong[framework] scores typed
inputs into it. `difficulty` is the sibling field, classified by anchors instead.

= The scale and bands

#scale-config(fw)

Bands turn a raw number into a label. The shipped scale runs
#scale-span(fw.scales.priority), banded #priority-bands(fw). A task may set a
bare integer:

#band-example(fw)

= Difficulty

`difficulty` is configured under its own `[tasks.difficulty]`, with the same
`kind`, `min`, `max` and `required` keys as priority — but classifies with sparse
labelled #strong[anchors] instead of bands. The shipped scale runs
#scale-span(fw.scales.difficulty), anchored #difficulty-anchors(fw) — a value
between anchors borrows the nearest lower anchor's glyph and colour. A project
overrides the range and the anchors under the same key —
#link(u("/config/"))[Configuration] carries them.

= Frameworks

A framework scores typed inputs instead of a hand-set number, and
#fw.frameworks.len() ship baked-in — tabled at the end of this page. Pick the
active one in config:

```toml
[tasks.priority]
framework = "ice"       # score through ICE
allow-manual = false    # strict: forbid raw integers
```

A task then calls the framework in place of a number:

#framework-call(fw, "ice")

The Rust layer scores it, bands the result, and keeps it on the scale.

= Manual versus strict

- `allow-manual = true` (default when a framework is set) — tasks may use either a
  raw integer or the framework call.
- `allow-manual = false` — every task must score through the framework; there is
  no scalar form. #link(u("/templates/"))[Templates & slots] covers how a
  scaffold fills it.

= Built-in frameworks

#framework-table(fw)

A framework's inputs — names, types, bounds — are declared under
`[tasks.priority.frameworks.<name>]` (the baked-in ones in
`crates/mindtape/src/frameworks.toml`), where its scoring and bands are
overridable too; define your own with the generic `framework(name, ..inputs)`
call. A framework with no formula classifies into named categories instead of
banding a number.
