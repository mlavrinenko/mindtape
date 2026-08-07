#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, hero, pkg
#import "../utils/site.typ": u

#show: page.with(
  title: "MindTape",
  active: "/",
  home: true,
  summary: "A daemon-free, file-based task tracker powered by Typst. One task per .typ file, live-evaluated on every command.",
)

#hero()

= What it is

A task is a single Typst file. You write it, MindTape reads it. Every command
re-evaluates the files under your configured roots with the Typst compiler —
there is no daemon watching, no database to sync, no global index to rebuild.
The filename stem is the task's identity, so links between tasks break loudly
when a file moves.

Everything scalar or enum — the status legend, the numeric priority and
difficulty scales, `kind`, `estimate`, dates, tags, and task links — is a generic
core field constrained by one config file. Tools built on MindTape add their own
fields through an `x-<ns>` extension seam that MindTape carries untouched.

= Why Typst

Because a task file is real code. You can `#import` a shared Typst library or your
own helpers, compute values, and reuse definitions across tasks — the file is
structured and typed, not a fragile line of text. The same file a command reads
as a row can also be typeset into a document.

- #strong[Plain text.] Read it, diff it, grep it, version it with the code it tracks.
- #strong[No lock-in.] No proprietary store, no cloud, no binary blobs.
- #strong[Composable.] Import libraries and files; share templates; build on prior work.
- #strong[One source, many views.] Query it as a table or JSON, or typeset it as a document.

#callout("note")[
  New here? #link(u("/quickstart/"))[Quickstart] gets you from install to your first
  task in a few commands. #link(u("/concepts/"))[Concepts] explains the model behind it.
]

= Built for humans and agents

The CLI is scriptable and quiet by default: table, JSON, CSV, or a bare path
list for piping into another program, honest exit codes, and a filter language
for querying. Structured JSON and slot schemas make it equally legible to AI
coding agents — see #link(u("/agents/"))[For AI agents].
