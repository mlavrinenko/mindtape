// Page wrapper used by every content file. One call form:
//
//   #import "../utils/page.typ": page
//   #let meta = (route: "/quickstart/", label: "Quickstart", title: …, summary: …)
//   #metadata(meta) <page-meta>
//   #show: page.with(..meta)
//
// The page's own `<page-meta>` dict, spread. `route` becomes `active` and
// `label` is dropped (tola-page has no use for the sidebar string); `title`,
// `summary` and `home` are used exactly as given. One binding per page feeds
// both consumers — the metadata block www/generate-site-pages.sh queries to
// build the manifest, and this show rule — so a page cannot disagree with
// itself about its own route, title or summary.
//
// The `active:`-only form this used to accept resolved title and summary out
// of that generated manifest instead. It was a shim, live only while the
// fifteen pages migrated (truth/tasks/pass-each-page-s-metadata-dict-to-its-
// show-rule.typ), and it is a panic now: left open it is a working second way
// to write a page whose facts come from somewhere other than the page, which
// is the whole failure this change set removes. The previous manifest
// migration left exactly this door open and had to come back and shut it.
//
// The seam is wrap-page's `transform-meta` hook, which rewrites the named
// arguments before tola-page ever sees them. Resolving there rather than in
// `view` is what makes the values reach BOTH consumers: the <tola-meta> block
// tola reads (title, summary) and the meta dict layout() renders the <h1>,
// the lede and the home-page branch from.

#import "../templates/tola.typ": wrap-page
#import "../templates/base.typ": base
#import "../templates/layout.typ": layout, make-head

#let _resolve(meta) = {
  if "active" in meta {
    panic(
      "page passes active: "
        + repr(meta.active)
        + " — that form is gone; spread the page's own <page-meta> dict "
        + "instead: #show: page.with(..meta)",
    )
  }
  if "route" not in meta {
    panic(
      "page declares no route — spread its <page-meta> dict "
        + "(#show: page.with(..meta)); got: "
        + meta.keys().join(", "),
    )
  }
  let out = meta
  let route = out.remove("route")
  if "label" in out { let _ = out.remove("label") }
  out.insert("active", route)
  out
}

#let page = wrap-page(
  base: base,
  head: make-head,
  view: (body, m) => layout(body, meta: m),
  transform-meta: _resolve,
)
