#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": REPO, u
#import "../utils/crate-notes.typ": crate-notes
#import "../generated/just.typ" as just

#let meta = (
  route: "/internals/",
  label: "Internals",
  title: "Internals",
  summary: "How MindTape is built, from its crate layout and evaluation cache to what differs per platform.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Nothing here is needed to use MindTape. It is the shape of the implementation,
for anyone auditing it, packaging it, or building on top.

= Workspace

// Facts (the crate list and versions) come from
// www/generated/crate-map.json (`cargo metadata` + rust-toolchain.toml); this
// page owns only the presentation and the per-crate prose (crate-notes.typ) —
// a machine cannot write "treating Typst as a data format". `just
// check-doc-facts` fails the build if a crate here has no matching prose.
#let crate-map = json("../generated/crate-map.json")

A #crate-map.workspace.len()-crate `cargo` workspace (edition
#crate-map.workspace-package.edition, pinned to rustc
#crate-map.rust-toolchain.channel):

#for c in crate-map.workspace [
  - #raw(c.name) #raw(c.version) — #crate-notes.at(c.name)
]

#crate-map.substrate.len() general-purpose crates sit underneath, published
separately because none of them knows what a task is:

#for c in crate-map.substrate [
  - #raw(c.name) #raw(c.version) — #crate-notes.at(c.name)
]

That split is the #link(u("/concepts/"))[extension seam] one layer down: a tool
that wants Typst-as-data gets it without inheriting a task model.

= The cache

Evaluating every file on every command is not free, so a content-hash cache
under each root skips re-evaluating files whose bytes have not changed. On a
generated backlog of 2000 tasks:

#table(
  columns: 3,
  table.header([Command], [Cold cache], [Warm cache]),
  [`mt ls`], [0.44 s], [0.18 s],
  [`mt check`], [0.33 s], [0.08 s],
)

// Hand-measured, and deliberately stamped as such: nothing in this repo
// derives these four numbers. #just.bench() prints hyperfine's report to the
// terminal and persists nothing, so there is no artifact a page could read —
// and inventing a fact pipeline for a number a reader is told to re-measure
// anyway would cost more than it settles. The stamp is what makes the figures
// safe to leave here: a date and a machine let a reader discount them.
Measured 2026-08-16 on one developer machine (Linux, i7-12700H). Nothing
regenerates these numbers, so read them as an order of magnitude and run
#just.bench() on yours — that recipe reports the warm column; the cold one is
the same two commands with `<root>/.mindtape/cache` removed before each run.

The cache is derived state, not a source of truth: delete it and the next run
rebuilds it; change the config, the prelude, or the `mt` version and it busts
itself. A read command may populate it and still leave the authored tree
byte-identical — that is the one exemption to the read/write split.

= Platform support

Every tagged release ships a prebuilt binary for each supported platform —
#link(u("/quickstart/"))[Quickstart] lists them — and CI runs the test suite on
Linux and Windows. Platform seams live where they act:
#link(u("/hooks/"))[Flip hooks] owns the platform shell a `run` command goes
through, and
#link(
  REPO + "/blob/main/docs/contributing/windows.md",
)[docs/contributing/windows.md]
owns the pty tests that stay Unix-only.

Everything else — the config and data directories, the Typst local-package path,
editor resolution, path separators in derived tags and link resolution — is the
same code on every platform, and the Windows CI job covers it.

#callout("note")[
  Building on MindTape? #link(u("/agents/"))[For AI agents] covers the structured
  output and describable schemas a tool drives it through.
]
