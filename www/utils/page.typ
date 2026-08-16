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
// Passing `title:`, `summary:` or `home:` here is now an ERROR, not an
// override. They were accepted while the fifteen pages migrated off writing
// them twice; leaving that door open would let the second copy grow back one
// page at a time, silently winning over the manifest and un-deriving the
// README list that reads from it. A page whose title differs from its sidebar
// label says so in the manifest — that is what its `title` field is for.
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

#let _owned = ("title", "summary", "home")

#let _resolve(meta) = {
  let route = meta.at("active", default: none)
  let taken = _owned.filter(k => k in meta)
  if taken.len() > 0 {
    panic(
      "page "
        + repr(route)
        + " passes "
        + taken.join(", ")
        + " — the manifest in utils/site.typ owns "
        + "these; set them on that page's NAV entry instead",
    )
  }
  let entry = page-meta(route)
  let resolved = (title: entry.title, summary: entry.summary)
  // Only the home page carries the flag; spreading it unconditionally would
  // write `home: false` into every other page's metadata for no reader.
  if "home" in entry { resolved.insert("home", entry.home) }
  // Dictionary `+` merges; nothing collides now that the three keys above are
  // rejected outright. Spreading — `(..resolved, ..meta)` — cannot express
  // this anyway: with no `key: value` pair to disambiguate it, Typst reads the
  // parentheses as an array and fails with "cannot spread dictionary into
  // array".
  resolved + meta
}

#let page = wrap-page(
  base: base,
  head: make-head,
  view: (body, m) => layout(body, meta: m),
  transform-meta: _resolve,
)
