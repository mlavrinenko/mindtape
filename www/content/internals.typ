#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u
#import "../utils/crate-notes.typ": crate-notes
#import "../generated/just.typ" as just

#let meta = (
  route: "/internals/",
  label: "Internals",
  title: "Internals",
  summary: "How MindTape is built: the crate layout, the content-hash cache that keeps live evaluation cheap, and what differs per platform.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Nothing here is needed to use MindTape. It is the shape of the implementation,
for anyone auditing it, packaging it, or building on top.

= Workspace

// Facts (the crate list, versions, direct-vs-transitive) come from
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

#let transitive = crate-map.substrate.filter(c => not c.direct)

#crate-map.substrate.len() general-purpose crates sit underneath, published
separately because none of them knows what a task is
(#(crate-map.substrate.len() - transitive.len()) direct,
#transitive.len() reached only transitively — never named in a workspace
manifest, and marked below with the crate that pulls it in):

// `via` is the cargo resolve graph's answer to WHICH crate pulls a transitive
// one, so this page names the parent instead of gesturing at "another crate".
#for c in crate-map.substrate [
  - #raw(c.name) #raw(c.version)#(
      if not c.direct [ (transitive, via #c.via.map(raw).join(", "))]
    ) — #crate-notes.at(c.name)
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

// The platform list is the release workflow's own build matrix, read through
// www/generated/release-targets.json — the same file
// www/content/quickstart.typ lists the download artifacts from. Adding or
// dropping a target is then a workflow edit, never a prose edit, and an
// unrecognized triple fails the build here rather than being silently dropped
// from the sentence.
#let OS-FAMILIES = (
  ("linux", "Linux"),
  ("apple-darwin", "macOS"),
  ("windows", "Windows"),
)
#let os-of(target) = {
  for (needle, name) in OS-FAMILIES {
    if target.contains(needle) { return name }
  }
  panic("internals: unknown release target family: " + target)
}
// Braces, not a bare chain broken over lines: a `#let` in markup mode ends its
// expression at the newline, so `json(…)\n  .targets` binds the whole JSON
// dictionary and the rest is read as markup.
#let platforms = {
  let release = json("../generated/release-targets.json")
  release.targets.map(t => os-of(t.target)).dedup()
}

#platforms.join(", ", last: " and ") all get a prebuilt binary per release, and
CI runs the test suite on Linux and Windows. What differs on Windows differs by
necessity, not omission:

- An `[[on.flip]]` `run` command goes through `cmd /C`, not `sh -c`. It is shell
  text, so a command written for one shell does not carry to the other.
- The terminal tests driven by a pty (`mt ls --interactive`, terminal-width and
  `NO_COLOR` handling) are Unix-only, because their harness is util-linux
  `script`. The behaviour is platform-independent and `crossterm` implements it
  over ConPTY; what is unverified there is the harness, not the feature.

Everything else — the config and data directories, the Typst local-package path,
editor resolution, path separators in derived tags and link resolution — is the
same code on every platform, and the Windows CI job covers it.

#callout("note")[
  Building on MindTape? #link(u("/agents/"))[For AI agents] covers the structured
  output and describable schemas a tool drives it through.
]
