#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "derive site-pages.json from the pages' page-meta blocks",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 3,
  ),
  difficulty: 6,
  tags: ("docs", "www", "tooling", "gating"),
  estimate: (files: 8),
  links: (
    parent("put-page-metadata-in-the-pages-and-derive-the-manifest.typ")
      + depends-on("add-a-queryable-page-meta-block-to-every-docs-page.typ")
  ),
  status: wip(2026, 8, 16),
)

== Summary

Make the fifteen `<page-meta>` blocks the source and `www/utils/site.typ` the
consumer. After this, a page's label, title and summary exist in exactly one
place — the page — and its route is written once instead of twice.

== Scope

- `www/generate.sh`: query each `www/content/*.typ` for `<page-meta>` and emit
  `www/generated/site-pages.json` in the shape `docs/src/lib.typ` already reads
  (`url`, `prefix`, `repo`, `pages`), plus `www/generated/site-pages.typ` — a
  Typst module `site.typ` imports for the label and route of each page. Runs
  under the lock that script already holds. Seed an empty manifest first when
  absent: compiling a page imports `layout.typ` #sym.arrow `site.typ` #sym.arrow
  this artifact, so the query that produces it needs it to exist. Verified an
  empty `NAV` compiles and queries clean.
- `www/generate-facts.sh`: drop its site-pages step and the driver, and amend
  the seven-artifacts header. Its `SITE_URL`/`PREFIX` derivation from
  `www/tola.toml` moves with the artifact.
- `docs/generate-md.sh`: run `www/generate.sh` before `generate-facts.sh`. Its
  header states the opposite today and has to be rewritten to say why — the
  standalone render now pays a `cargo build --release -p mindtape-cli`, and
  inside `just check` it costs nothing because `docs::check` already ran it.
- `www/utils/page.typ`: accepts the page's own metadata dict spread as named
  arguments — `route`, `label`, `title`, `summary`, `home` — mapping `route` to
  `active` and dropping `label` before `tola-page` sees it. It keeps accepting
  the current `active:`-only form, resolving title and summary from the
  manifest as it does today, so the fifteen pages can migrate in a following
  leaf rather than inside this one; the leaf after that closes the door. This
  is the same shim the previous manifest migration ran and then removed, and it
  is a shim only because a task cannot touch 23 files at once.
  The `_owned` guard that makes `title:`/`summary:` an error goes now: it
  existed to stop a second copy growing beside the manifest, and the page IS
  the manifest after this.
- `www/utils/site.typ`: `NAV` keeps the group titles and the reading order,
  referencing the generated entries; `PAGES` and `page-meta` go. Nothing in this
  file restates a label, a title or a summary afterwards.
- `www/templates/layout.typ`: follow whatever `NAV`'s new shape needs.
- `.just/scripts/check-doc-facts.sh`: the pages arm becomes three-way —
  filename stem, the route the page declares, and nav membership. A page whose
  `<page-meta>` route disagrees with its filename fails by name. This is the
  hole that motivated the umbrella: today the arm compares filenames against the
  manifest and never reads the page's own route, so an `active:` pointing at
  another live route passes green.
- `.just/docs.just`: the `gen` recipe's comment argues generate.sh-then-facts
  ordering; extend it to cover the moved artifact.

== Regression tests

Two, both of which must be seen to FAIL before the fix:

+ A page whose declared route does not match its filename fails
  `check-doc-facts`, naming the page.
+ A page missing its `<page-meta>` block fails the producer, naming the page —
  not silently omitted from the nav.

== Out of scope

The fifteen page edits (the sibling leaf) and the reading order itself. Group
names and page order stay hand-written in `site.typ`: that is the one fact no
single page owns, and spreading it across fifteen files as `order:` integers
would make the sidebar's shape invisible and let two pages claim one slot.
