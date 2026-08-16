// Page wrapper used by every content file.
//
//   #import "../utils/page.typ": page
//   #show: page.with(active: "/quickstart/")
//
// A page declares its ROUTE. Its title, summary and home flag are resolved
// from the PAGES manifest in utils/site.typ by that route, so the facts the
// page and the sidebar both need are written once, in the place that already
// had to know every page exists. An unknown route panics naming it (see
// page-meta), which turns a renamed page from a silent 404 in the nav into a
// build failure.
//
// `title:`, `summary:` and `home:` are still accepted, as OPTIONAL overrides
// that win over the manifest — every content page passes them today and
// builds unchanged; the parameters go away once no page does.
//
// The seam is wrap-page's `transform-meta` hook, which rewrites the named
// arguments before tola-page ever sees them. Resolving there rather than in
// `view` is what makes the manifest's values reach BOTH consumers: the
// <tola-meta> block tola reads (title, summary) and the meta dict layout()
// renders the <h1>, the lede and the home-page branch from.

#import "../templates/tola.typ": wrap-page
#import "../templates/base.typ": base
#import "../templates/layout.typ": layout, make-head
#import "../utils/site.typ": page-meta

#let _resolve(meta) = {
  let entry = page-meta(meta.at("active", default: none))
  let defaults = (title: entry.title, summary: entry.summary)
  // Only the home page carries the flag; spreading it unconditionally would
  // write `home: false` into every other page's metadata for no reader.
  if "home" in entry { defaults.insert("home", entry.home) }
  // Dictionary `+` takes the RIGHT side on a collision, so the page's own
  // arguments win over the manifest's. Spreading — `(..defaults, ..meta)` —
  // cannot express this: with no `key: value` pair to disambiguate it, Typst
  // reads the parentheses as an array and fails with "cannot spread
  // dictionary into array".
  defaults + meta
}

#let page = wrap-page(
  base: base,
  head: make-head,
  view: (body, m) => layout(body, meta: m),
  transform-meta: _resolve,
)
