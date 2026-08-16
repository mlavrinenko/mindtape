// Page wrapper used by every content file. Two call forms, both live:
//
//   #import "../utils/page.typ": page
//   #show: page.with(active: "/quickstart/")
//
// The OLD form: a page declares only its ROUTE, and its title, summary and
// home flag are resolved from the PAGES manifest in utils/site.typ by that
// route — the fifteen pages still write this. An unknown route panics naming
// it (see site.typ's page-meta), which turns a renamed page from a silent 404
// in the nav into a build failure; during www/generate-site-pages.sh's
// bootstrap pass, before the manifest exists, page-meta returns `none`
// instead, and this render resolves no title/summary/home rather than
// crashing the very query that would produce the manifest.
//
//   #show: page.with(..page-meta-dict)
//
// The NEW form: a page's own `<page-meta>` metadata dict — `route`, `label`,
// `title`, `summary`, `home` — spread directly. `route` maps to `active` and
// `label` is dropped (tola-page has no use for the sidebar string); `title`,
// `summary` and `home` are used exactly as given, no manifest lookup at all.
// This is what the sibling leaf (add-a-queryable-page-meta-block-to-every-
// docs-page.typ) migrates the fifteen pages to, closing the "second copy
// grows back" hole `title:`/`summary:` used to guard against by erroring —
// that guard is gone now because the page IS the manifest's source, not a
// second copy of it.
//
// The seam is wrap-page's `transform-meta` hook, which rewrites the named
// arguments before tola-page ever sees them. Resolving there rather than in
// `view` is what makes the resolved values reach BOTH consumers: the
// <tola-meta> block tola reads (title, summary) and the meta dict layout()
// renders the <h1>, the lede and the home-page branch from.

#import "../templates/tola.typ": wrap-page
#import "../templates/base.typ": base
#import "../templates/layout.typ": layout, make-head
#import "../utils/site.typ": page-meta

#let _resolve(meta) = {
  let route = meta.at("route", default: meta.at("active", default: none))
  let rest = meta
  for k in ("route", "label") {
    if k in rest { rest.remove(k) }
  }
  rest.insert("active", route)

  // New form: the page already supplies its own title/summary/home, so there
  // is nothing to resolve and no reason to touch PAGES at all — a page using
  // this form never depends on the manifest existing, unlike the old one.
  if ("title" in meta) or ("summary" in meta) or ("home" in meta) {
    return rest
  }

  // Old form: resolve from the manifest by route, exactly as before. `entry`
  // is `none` only in the bootstrap pass (see the header comment above); this
  // render then carries no title/summary/home rather than panicking.
  let entry = page-meta(route)
  if entry == none { return rest }
  let resolved = (title: entry.title, summary: entry.summary)
  // Only the home page carries the flag; spreading it unconditionally would
  // write `home: false` into every other page's metadata for no reader.
  if "home" in entry { resolved.insert("home", entry.home) }
  // Dictionary `+` merges; `rest` no longer carries `title`/`summary`/`home`
  // (this branch only runs when none of the three was in `meta`), so nothing
  // collides. Spreading — `(..resolved, ..rest)` — cannot express this
  // anyway: with no `key: value` pair to disambiguate it, Typst reads the
  // parentheses as an array and fails with "cannot spread dictionary into
  // array".
  resolved + rest
}

#let page = wrap-page(
  base: base,
  head: make-head,
  view: (body, m) => layout(body, meta: m),
  transform-meta: _resolve,
)
