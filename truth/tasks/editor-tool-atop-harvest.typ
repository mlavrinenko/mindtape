#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Editor tool atop typst-harvest (only if needed)",
  status: proposed(2026, 6, 14),
  priority: "low",
  relates-to: (link("extract-typst-harvest.typ"),),
  tags: ("substrate", "cli"),
)

== Summary

Mutation commands (`set`, status toggles) are intentionally absent from the CLI.
`typst-harvest` collects data; it does not act on it, and shouldn't grow an edit
half. In-place editing is a distinct concern: it needs source spans and
write-back, not harvesting.

== Decision

If status / field mutation is ever wanted, build it as a separate edit-capable
tool on top of `typst-harvest` rather than bolting mutation onto the harvester or
the read-only CLI. Until a concrete need appears, MindTape stays read-only:
author tasks by editing `.typ` files directly.
