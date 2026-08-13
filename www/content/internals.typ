#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u

#show: page.with(
  title: "Internals",
  active: "/internals/",
  summary: "How MindTape is built: the crate layout, the content-hash cache that keeps live evaluation cheap, and what differs per platform.",
)

Nothing here is needed to use MindTape. It is the shape of the implementation,
for anyone auditing it, packaging it, or building on top.

= Workspace

A two-crate `cargo` workspace. `mindtape` is the library — config, the task
model, the filter language, and config-driven validation — and `mindtape-cli` is
the thin `mt` binary.

Three general-purpose crates sit underneath, published separately because none
of them knows what a task is:

- `typst-harvest` — evaluates a Typst file and collects the data it marked for
  export, treating Typst as a data format. Never lays out or renders a page.
- `typst-cst` — reads a file's structure (its heading tree, with source spans)
  by parsing alone, without evaluating it.
- `typst-edit` — rewrites a Typst file in place at known spans, applying a batch
  of edits as one change.

That split is the #link(u("/concepts/"))[extension seam] one layer down: a tool
that wants Typst-as-data gets it without inheriting a task model.

= The cache

Evaluating every file on every command is not free, so a content-hash cache
under each root skips re-evaluating files whose bytes have not changed. Measured
on a generated backlog of 2000 tasks (`just bench`, on one developer machine —
run it on yours):

#table(
  columns: 3,
  table.header([Command], [Cold cache], [Warm cache]),
  [`mt ls`], [0.46 s], [0.20 s],
  [`mt check`], [0.35 s], [0.12 s],
)

The cache is derived state, not a source of truth: delete it and the next run
rebuilds it; change the config, the prelude, or the `mt` version and it busts
itself. A read command may populate it and still leave the authored tree
byte-identical — that is the one exemption to the read/write split.

= Platform support

Linux, macOS and Windows all get a prebuilt binary per release, and CI runs the
test suite on Linux and Windows. Two things differ on Windows, both by necessity
rather than omission:

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
