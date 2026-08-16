#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "de-slop the docs and close the remaining drift seams",
  kind: "umbrella",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("docs", "www"),
  estimate: (files: 2),
  status: done(2026, 8, 16),
)

== Summary

The generated Markdown and the docs site now share facts through
`www/generated/`, but the PROSE around those facts is still hand-kept in two
places, and some of it is stale. README restates each site page's purpose,
re-tells the install and one-time-setup story the Quickstart page already
derives, hardcodes fifteen `https://mlavrinenko.github.io/mindtape/...` URLs
nothing verifies, and thanks Lucide for a logo that is no longer theirs. This
umbrella closes those seams the same way the fact seams were closed — one
source, read by both sides — and takes a prose pass over every doc while the
files are open.

== Scope

- A page manifest derived from one source, so a link to a docs page and that
  page's one-line description are both read, never retyped.
- README rewritten against it: no second install story, no restated page
  descriptions, no stale acknowledgement.
- A concision pass over every hand-written doc: the site pages, the
  contributor deep dives, and the agent-facing sources.

== Not in scope

The derived-fact pipeline itself (`www/generate.sh`,
`www/generate-facts.sh`'s existing artifacts) — it works. This is about the
prose that surrounds it.
