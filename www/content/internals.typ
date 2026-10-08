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

You don't need anything on this page to use MindTape. It's for you if you
audit, package or build on the implementation.

= Workspace

// The crate list and versions come from www/generated/crate-map.json, and the
// one-line notes from www/utils/crate-notes.typ. `just check-doc-facts` fails
// when a crate has no note.
#let crate-map = json("../generated/crate-map.json")

A #crate-map.workspace.len()-crate `cargo` workspace (edition
#crate-map.workspace-package.edition, pinned to rustc
#crate-map.rust-toolchain.channel):

#for c in crate-map.workspace [
  - #raw(c.name) #raw(c.version): #crate-notes.at(c.name)
]

#crate-map.substrate.len() general-purpose crates sit underneath. They're
published separately, and none of them knows what a task is, so a tool that
reads Typst files as data can use them without the task model.

#for c in crate-map.substrate [
  - #raw(c.name) #raw(c.version): #crate-notes.at(c.name)
]

= The cache

`mt` evaluates task files with the Typst compiler, which takes time on a large
backlog. So each project keeps one cache in `.mindtape/cache/`, keyed by each
file's content hash, and `mt` evaluates only the files whose bytes changed
since the last run. A file that imports a module also counts as changed when
that module's bytes did. `mt` writes a `.gitignore` into that directory, so git
never sees the cache. On a generated backlog of 2000 tasks:

#table(
  columns: 3,
  table.header([Command], [Cold cache], [Warm cache]),
  [`mt ls`], [0.44 s], [0.18 s],
  [`mt check`], [0.33 s], [0.08 s],
)

// Measured by hand: `just bench` prints hyperfine's report and saves nothing a
// page could read, so the date and machine below let a reader judge the
// figures.
These figures were measured on 2026-08-16 on one Linux machine with an
i7-12700H, and nothing regenerates them. Run #just.bench() on your machine for
the warm column. For the cold column, set `MINDTAPE_NO_CACHE=1` to make `mt`
skip the cache.

A cache entry is a copy of what evaluation produced, so you lose nothing by
deleting the directory. The next run rebuilds it. A change to the config, the
prelude or the `mt` version starts a new cache and removes the old one. A read
command such as `mt ls` may write the cache, but it never changes a task
file.

= Platform support

Every tagged release ships a prebuilt binary for each supported platform,
listed in the #link(u("/quickstart/"))[Quickstart]. CI runs the tests on Linux
and Windows. Two things differ by platform. #link(u("/hooks/"))[Flip hooks]
says which shell runs a hook's `run` command, and
#link(
  REPO + "/blob/main/docs/contributing/windows.md",
)[docs/contributing/windows.md]
lists the terminal tests that run on Unix only.

Everything else is the same code on every platform: the config and data
directories, the Typst local-package path, editor lookup, and the path
separators in derived tags and links. The Windows CI job tests it.

#callout("note")[
  Building on MindTape? #link(u("/agents/"))[For AI agents] covers the JSON
  output and the schemas a tool reads.
]
