#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "add a queryable page-meta block to every docs page",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (files: 15),
  links: parent("put-page-metadata-in-the-pages-and-derive-the-manifest.typ"),
  status: proposed(2026, 8, 16),
)

== Summary

Give every `www/content/*.typ` page the metadata block the manifest will be
derived from, before anything reads it. Fifteen files, one shape:

```typ
#metadata((
  route: "/cli/",
  label: "CLI",
  title: "CLI reference",
  summary: "Every mt command, grouped, with the deep topics linked out to their own pages.",
)) <page-meta>
```

The four values are copied verbatim out of `www/utils/site.typ`'s `NAV` entry
for that route — this task invents no prose. `index.typ` also carries
`home: true`, the only page that does.

== Scope

- `www/content/*.typ`: the block, placed above the `#show: page.with(...)` line
  so it reads as the page's header rather than as body content.
- Nothing else. `site.typ` still owns the manifest, `page.typ` still resolves
  title and summary by route, and every page keeps its `active:` argument. The
  sibling leaf takes all of that over.

== Why this lands alone

The block is inert for one commit: no producer queries it, no page reads it, so
there is nothing for it to drift against. That is what makes the split safe —
the moment `site.typ` stops owning these values, the pages already have them.

Verify with the same query the producer will use:

```
typst query --root www www/content/cli.typ '<page-meta>' \
  --field value --one --target html --features html
```

It needs `www/generate.sh`'s captures to have run (the query is a full compile);
`just docs gen` provides them.
