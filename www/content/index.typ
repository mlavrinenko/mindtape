#import "../utils/page.typ": page
#import "../utils/ui.typ": hero
#import "../utils/demo.typ": demo
#import "../utils/site.typ": u

#let meta = (
  route: "/",
  label: "Overview",
  title: "MindTape",
  summary: "A daemon-free task tracker whose tasks are Typst files, live-evaluated on every command.",
  home: true,
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

#hero()

= What it is

A task is a single Typst file. Every command reads the files under your
configured roots instead of loading a store, so there is no database to back
up, no index to rebuild, and no daemon to keep alive: the repository you
already version is the tracker. A task's identity is its filename stem, its
fields are declared in a Typst call your editor completes, and one config file
decides which values are legal. Tools built on MindTape add their own fields
through an `x-<ns>` seam — #link(u("/concepts/"))[Concepts] has the model.

= One task, two ways

#demo((route, body) => link(u(route), body))

= Why Typst

A task file is real code.

- #strong[Plain text.] Read it, diff it, grep it, version it with the code it tracks.
- #strong[No lock-in.] No proprietary store, no cloud, no binary blobs.
- #strong[Composable.] `#import` a shared library or your own helpers, compute
  values, reuse definitions across tasks.

= Built for humans and agents

The CLI is scriptable and quiet by default: table, JSON, CSV, or a bare path
list for piping into another program, honest exit codes, and a filter language
for querying (`mt find` is a shortcut over it). Structured JSON and slot
schemas make it equally legible to AI coding agents — see
#link(u("/agents/"))[For AI agents].
