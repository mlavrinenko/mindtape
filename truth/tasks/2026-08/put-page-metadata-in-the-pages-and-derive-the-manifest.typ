#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "put page metadata in the pages and derive the manifest",
  kind: "milestone",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("docs", "www", "tooling"),
  estimate: (files: 1),
  links: related("derive-the-docs-page-manifest-from-one-source.typ")[
    Built the manifest this inverts, and argued why it had to live in
    `site.typ` at the time.
  ],
  status: done(2026, 8, 16),
)

== Summary

`www/utils/site.typ`'s `PAGES` manifest holds every page's sidebar label, `<h1>`
title and one-sentence summary in a file the page's author never opens. Those
facts are single-copy: `page.typ` resolves them by route and panics on an unknown
one. They still sit far from the prose they describe. A rewritten page then keeps a
summary nothing can catch as stale.

The route itself appears twice: `NAV`'s `href`, and again as the page's own
`active:`. No code compares them. The pages arm in
`.just/scripts/check-doc-facts.sh` derives routes from filenames and diffs those
against the manifest, so it never reads `active:`. Point `www/content/cli.typ` at
`active: "/dsl/"` and `/cli/` renders the DSL page's title, description,
`<meta name="description">` and sidebar highlight while the gate stays green.

Invert it. Each page carries its own
`#metadata((route, label, title, summary)) <page-meta>` block, and the manifest is
built from the fifteen pages with `typst query`. `site.typ` keeps only what no
single page owns: the group names and the reading order.

== Feasibility, measured

Against typst 0.14.2 on 2026-08-16, on throwaway probe pages under `www/content/`:

- The query works.
  `typst query --root www <page> '<page-meta>' --field value --one --target html
  --features html` returns the dict verbatim. 0.25 s per page, warm.
- No import cycle. The cycle that forced the manifest into `site.typ`
  (`site.typ` #sym.arrow page #sym.arrow `page.typ` #sym.arrow `layout.typ`
  #sym.arrow `site.typ`) exists only because `page.typ` reads the manifest. A
  page passing its own title and summary deletes that edge.
- Bootstrap works. Compiling a page still imports `layout.typ` #sym.arrow
  `site.typ` #sym.arrow the generated manifest, so the manifest must exist before
  the query that produces it. An empty `NAV` compiles and queries clean. The
  producer then seeds an empty manifest when absent, queries the pages, and
  writes the real one.
- The query is a full compile, so it needs `www/generate.sh`'s captures. With
  only `generate-facts.sh`'s own artifacts present, the query dies at
  `www/utils/ui.typ:135`'s top-level `read("../generated/hero-source.typ")`. The
  page bodies then read their own (`cli.typ`'s `json("../generated/cli.json")`).
  A chrome-only probe (`page.typ` + `site.typ`, no `ui.typ`) queries fine, so the
  blocker is page content, not the chrome.

== The cost this buys

The site-pages step moves out of `www/generate-facts.sh`, which runs standalone
without a built binary, and into `www/generate.sh`. `docs/generate-md.sh`
gains a `generate.sh` dependency it deliberately does not have today (see its
header). Inside `just check` that comes free: `docs::check` already runs
`generate.sh` first, and both share the `target/www-generated.lock`, so the
captures are there either way. A standalone `just docs md` then pays
`generate.sh`'s `cargo build --release -p mindtape-cli`.

Decide that trade before starting. If the standalone cost is unacceptable, the
alternative is not this task at all. A sidecar leaf module per page
(`www/meta/<stem>.typ`) would work instead. It imports only itself, so it works
without captures or an extractor: near-co-location instead of co-location.

Rejected: keep the facts in `#let` bindings and read them off the parse plane
with `typst-cst`, which works without captures. The extractor is a Rust binary,
so `generate-facts.sh` would still need a `cargo build`. That is the same
coupling this task is weighing, plus a second reader of the page that can
disagree with the real compile.

== Leaves

+ `add-a-queryable-page-meta-block-to-every-docs-page.typ`, the fifteen pages.
+ `derive-site-pages-json-from-the-pages-page-meta-blocks.typ`, the producer,
  `site.typ`, `page.typ`, and the gate.
