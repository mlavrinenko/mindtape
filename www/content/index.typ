#import "../utils/page.typ": page
#import "../utils/ui.typ": hero
#import "../utils/site.typ": u

#show: page.with(active: "/")

#hero()

= What it is

A task is a single Typst file. You write it, MindTape reads it: every command
re-evaluates the files under your configured roots with the Typst compiler, then
applies your config. A task's identity is its filename stem. Core fields are
constrained by one config file, and tools built on MindTape add their own through
an `x-<ns>` extension seam — #link(u("/concepts/"))[Concepts] has the model.

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
