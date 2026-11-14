#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "pass each page's metadata dict to its show rule",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (files: 15),
  links: (
    parent("put-page-metadata-in-the-pages-and-derive-the-manifest.typ")
      + depends-on("derive-site-pages-json-from-the-pages-page-meta-blocks.typ")
  ),
  status: done(2026, 8, 16),
)

== Summary

After the manifest derivation from the pages, a page still writes its route
twice: once as `route:` in its `<page-meta>` block, once as `active:` three
lines below. The whole umbrella is about not writing a fact twice. A mismatch
between those two is worse than the one it replaced, because the gate compares
the declared route against the filename. `active:` can point somewhere else
entirely, and only the sidebar highlight and the resolved title go wrong.

Bind the dict once and spread it:

```typ
#let meta = (
  route: "/cli/",
  label: "CLI",
  title: "CLI reference",
  summary: "Every mt command, grouped, with the deep topics linked out to their own pages.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)
```

== Scope

- `www/content/*.typ`, all fifteen: bind `meta`, emit it, spread it. Drop the
  `active:` argument. `page` maps `route` to it.
- `www/content/index.typ` keeps `home: true` inside the dict.
- The prose, imports and values all stay unchanged. The rendered site must be
  byte-identical. `just docs check` plus a `<tola-meta>` query per page is how
  you know.

== Out of scope

Closing the shim in `www/utils/page.typ` (the sibling leaf). While this lands,
`page` accepts both forms.
