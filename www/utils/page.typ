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
// An `active:`-only call panics. Accepting it would give a second way to
// write a page whose facts come from somewhere other than the page.
//
// The seam is wrap-page's `transform-meta` hook, which rewrites the named
// arguments before tola-page ever sees them. Resolving there rather than in
// `view` is what makes the values reach both consumers: the <tola-meta> block
// tola reads (title, summary) and the meta dict layout() renders the <h1>,
// the lede and the home-page branch from.

#import "../templates/tola.typ": wrap-page
#import "../templates/base.typ": base
#import "../templates/layout.typ": layout, make-head
#import "mode.typ": MARKDOWN
#import "site.typ": render-summary

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

#let _html-page = wrap-page(
  base: base,
  head: make-head,
  view: (body, m) => layout(body, meta: m),
  transform-meta: _resolve,
)

// The Markdown copy (see mode.typ): the title, the summary and
// the body, with no tola wrapper and no chrome. typlite keeps an HTML h1 as a
// tag, and www/generate-md.sh turns it into `#`. The home page's hero carries
// the summary as its own lede.
#let _md-page(body, ..args) = {
  let m = _resolve(args.named())
  html.elem("h1")[#m.title]
  parbreak()
  if "summary" in m and not m.at("home", default: false) {
    par(render-summary(m.summary))
  }
  body
}

#let page = if MARKDOWN { _md-page } else { _html-page }
