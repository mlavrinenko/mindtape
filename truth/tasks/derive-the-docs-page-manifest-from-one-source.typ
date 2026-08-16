#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "derive the docs page manifest from one source",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("docs", "www", "tooling"),
  estimate: (files: 9),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ"),
  status: accepted(2026, 8, 16),
)

== Summary

Three facts about a docs page are written by hand in three places today: its
route (`www/utils/site.typ`'s `NAV`, and again as each page's `active:`), its
title, and its one-line summary (`www/content/*.typ`'s `summary:`). A fourth,
the site's public URL, is typed in `www/tola.toml` and again as `PREFIX` in
`site.typ` with a "keep in sync" comment. Nothing checks any of it, and
`docs/src/readme.typ` links fifteen absolute site URLs that no build verifies
— a renamed page leaves README pointing at a 404.

Make `site.typ` the one source: route, sidebar label, page title, summary.
`page.typ` resolves them from `active:`, so a page declares its route and
nothing else. Derive the site URL and its path prefix from `tola.toml`. Emit
the whole thing as `www/generated/site-pages.json`, and give `docs/src/lib.typ`
a `www-link` that fails the render on an unknown route instead of shipping a
dead link.

== Scope

- `www/utils/site.typ`: `NAV` entries carry `title` and `summary`; a flat
  `PAGES` lookup by route; `SITE_URL`/`PREFIX` read out of `../tola.toml`.
- `www/utils/page.typ`: `page.with(active: "/x/")` resolves title and summary
  from `PAGES`; an unknown route panics naming it.
- `www/generate-facts.sh`: a `site-pages.json` artifact, read back off
  `site.typ` through a throwaway driver the way `check-doc-coverage` reads
  `cli-notes.typ`.
- `docs/src/lib.typ`: `www-link(route)[body]`, `www-title`, `www-summary` —
  each panicking on a route absent from the manifest.
- A gate arm asserting `www/content/*.typ` and `PAGES` are one-to-one, so a
  new page without a nav entry (or a nav entry without a page) fails by name.
- The Justfile / gates-doc / outdatty reconcile that arm forces.

== Constraints

`docs/generate-md.sh` runs `generate-facts.sh` and nothing else, so the
manifest must be derivable WITHOUT `www/generate.sh`'s captures — evaluating a
`www/content/*.typ` page reads `www/generated/*.json`, which is why the
manifest reads `site.typ` rather than the pages.

Do not touch `www/content/*.typ` here at all. `page` keeps accepting `title:`
and `summary:` as optional overrides that default to the manifest, so every
page still builds unchanged; the three prose tasks strip the now-redundant
arguments from the pages they own, and the umbrella's close removes the
override parameters once no page passes them.
