#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "make a page's active argument an error",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (files: 1),
  links: (
    parent("put-page-metadata-in-the-pages-and-derive-the-manifest.typ")
      + depends-on("pass-each-page-s-metadata-dict-to-its-show-rule.typ")
  ),
  status: proposed(2026, 8, 16),
)

== Summary

`www/utils/page.typ` accepts two forms during the migration: the page's own
metadata dict spread as named arguments, and the older `active:`-only form that
resolves title and summary back out of the generated manifest. Once no page
passes `active:`, close it — passing `active:` becomes a panic naming the page,
and the manifest-resolution branch goes with it.

Left open, the shim is a working second way to write a page whose facts come
from somewhere other than the page. That is the failure this umbrella exists to
remove, and the previous manifest migration left exactly this door open and had
to come back and shut it.

== Scope

- `www/utils/page.typ`: the `active:` branch and the `site.typ` import both go.
  `route` becomes required — a page passing neither fails naming what it passed.
- Verify the panic fires and names the offending page before calling this done.
