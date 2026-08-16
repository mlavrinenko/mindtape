#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "de-slop the docs site reference pages",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("docs", "www"),
  estimate: (files: 6),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ")
    + depends-on("derive-the-docs-page-manifest-from-one-source.typ"),
  status: done(2026, 8, 16),
)

== Summary

The reference pages carry the most prose per fact, and `config.typ` is the
longest Typst file in the repo at 747 lines. A reference page is consulted,
not read — its prose should be short enough that the table underneath is the
thing the eye lands on. Tighten the language, and strip the `title:`/`summary:`
arguments these pages no longer need once the manifest lands.

== Scope

`www/content/`: `cli.typ`, `filters.typ`, `priority.typ`, `config.typ`,
`schema.typ`, `import-export.typ`.

== Rules

Facts are not the target — slop is. Every command, flag, field, config key and
derived read stays exactly as it is, and nothing that a gate reads (the
`cli-notes` keys, the config-gate's section coverage) may lose an entry. What
goes: a paragraph that re-narrates the table beneath it, a sentence that only
restates its own heading, hedging, and worked examples that teach nothing the
one above them did not. Do not restructure headings or move content between
pages.
