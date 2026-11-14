#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "de-slop the docs site closing pages",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 4,
  tags: ("docs", "www"),
  estimate: (files: 3),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ")
    + depends-on("derive-the-docs-page-manifest-from-one-source.typ"),
  status: done(2026, 8, 16),
)

== Summary

The three closing pages are where stale claims collect. `comparison.typ`
describes other trackers and their release cadence. `internals.typ` carries a
benchmark number and a crate story. `agents.typ` describes a CLI surface that
has grown since its last revision. Verify each claim against its source,
tighten the prose, and strip the `title:`/`summary:` arguments that the
manifest makes redundant.

== Scope

`www/content/`: `agents.typ`, `internals.typ`, `comparison.typ`.

== Rules

Facts are not the target. The target is slop, plus any false claim. Where a
claim has a machine-readable source in the repo, say so in the task's closing
note rather than hand-fixing the number. A fact worth stating twice is a fact
worth deriving. Do not restructure headings or move content
between pages.
