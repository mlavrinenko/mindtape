#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "de-slop the docs site start and guide pages",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("docs", "www"),
  estimate: (files: 6),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ")
    + depends-on("derive-the-docs-page-manifest-from-one-source.typ"),
  status: accepted(2026, 8, 16),
)

== Summary

The Start and Guide pages are the ones a new reader meets first, and they
repeat each other: the overview, the quickstart and the concepts page each
explain live evaluation and identity-by-filename in their own words. Cut the
repetition, tighten the language, and strip the `title:`/`summary:` arguments
these pages no longer need once the manifest lands.

== Scope

`www/content/`: `index.typ`, `quickstart.typ`, `concepts.typ`, `dsl.typ`,
`templates.typ`, `derived-tags.typ`.

== Rules

Facts are not the target — slop is. Every command, flag, field, config key and
derived read stays exactly as it is. What goes: a point made twice on one
page, a sentence that only restates its own heading, hedging, and the
self-congratulatory register ("this is what makes it powerful"). Do not
restructure headings or move content between pages.
