// Site-wide constants for the MindTape docs. Single source of truth for the
// site URL and its path prefix, and the nav tree's group names and reading
// order. The site URL and the Typst package version are not constants at all
// any more — both are read from the file that owns them, and neither is the
// page manifest (route -> label, title, summary): that is now `PAGES`,
// read back off every page's own `<page-meta>` block via
// www/generate-site-pages.sh, under
// truth/tasks/derive-site-pages-json-from-the-pages-page-meta-blocks.typ.
// This file no longer restates a label, a title or a summary anywhere.

// Typst library version, read from lib/typst.toml through the crate map — the
// same `typst-lib.version` docs/src/{agents,readme,skill}.typ already read, so
// the site stops being the last place that hand-types it and a bump propagates
// with no human reconciliation. Not read from lib/typst.toml directly: the
// site builds under `--root www`, and Typst refuses a read that escapes the
// root. www/generate.sh runs generate-facts.sh (which writes crate-map.json)
// before generate-site-pages.sh evaluates this module to query a page, so the
// file is always there by the time anything imports it.
#let PKG_VERSION = json("../generated/crate-map.json").at("typst-lib").version
#let PKG = "@local/mindtape:" + PKG_VERSION

#let REPO = "https://github.com/mlavrinenko/mindtape"

// The site's public URL, read out of the tola config rather than retyped
// beside it: "keep in sync with tola.toml" is a comment, not a mechanism, and
// this string is what every absolute link into the site is built from
// (docs/src/lib.typ's `www-link`, via www/generated/site-pages.json).
//
// Typst resolves a relative path against the FILE, not the root, so this one
// read works under both roots this module is evaluated with — `--root www`
// for the site build and for www/generate-site-pages.sh's page queries,
// `--root .` for that same script's site-meta driver. The trailing-slash trim
// is what lets `SITE_URL + route` compose cleanly for a route that already
// opens with `/`.
#let TOLA = toml("../tola.toml")
#let SITE_URL = TOLA.site.info.url.trim("/", at: end)

// GitHub Pages serves this as a project page, under the path component of
// that URL ("/mindtape") — derived here rather than hand-typed a second time,
// so moving the site to another host or path is a one-line tola.toml edit.
// u() builds internal page links; assets stay source-relative (/assets/...)
// — tola applies the prefix to those on build.
#let _url-path(url) = {
  let parts = url.split("://").last().split("/")
  if parts.len() < 2 { "" } else { "/" + parts.slice(1).join("/") }
}
#let PREFIX = _url-path(SITE_URL)
#let u(path) = PREFIX + path

#let SITE = (
  title: "MindTape",
  tagline: "Tasks as Typst files. No daemon, no database, no lock-in.",
)

// Grouped sidebar: group title, and its members' reading order as ROUTES.
// This is the one fact no single page owns — spreading it across the page
// files as `order:` integers would make the sidebar's shape invisible and let
// two pages claim one slot (see the umbrella task's "Out of scope"). Every
// other fact a nav entry used to carry (label, title, summary, home) now
// lives on the page itself and is resolved from `PAGES` below by route.
#let NAV = (
  (title: "Start", items: ("/", "/quickstart/")),
  (
    title: "Guide",
    items: (
      "/concepts/",
      "/dsl/",
      "/templates/",
      "/derived-tags/",
      "/self-validating-tasks/",
    ),
  ),
  (
    title: "Reference",
    items: (
      "/cli/",
      "/filters/",
      "/priority/",
      "/config/",
      "/rule-packs/",
      "/named-sets/",
      "/hooks/",
      "/estimates/",
      "/schema/",
      "/import-export/",
    ),
  ),
  (title: "More", items: ("/agents/", "/internals/", "/comparison/")),
)

// The page manifest — route -> {label, title, summary, home?} — derived from
// every page's own `<page-meta>` block by www/generate-site-pages.sh, never
// hand-written here. Empty only during that script's bootstrap pass (see its
// header): compiling a page imports layout.typ -> this file -> this artifact,
// so the artifact has to exist, even empty, before the query that fills it.
#let PAGES = json("../generated/site-pages.json").pages

// Resolve one page by route. Three outcomes, not two:
//
//   - PAGES is EMPTY (the bootstrap pass, before a single page has been
//     queried yet): return `none`. Panicking here would make the bootstrap
//     pass itself un-compilable, which is exactly the chicken-and-egg
//     www/generate-site-pages.sh exists to break.
//   - PAGES is non-empty and has the route: return its entry.
//   - PAGES is non-empty and lacks the route: panic naming it. A silent skip
//     here is the exact failure this whole change removes — a route is a
//     string typed by hand in a page header, in NAV above, or in doc prose,
//     and once the manifest is real, an unknown route is always a mistake,
//     never a pass still warming up.
#let page-meta(route) = {
  if PAGES.len() == 0 { return none }
  if type(route) == str and route in PAGES { return PAGES.at(route) }
  panic(
    "unknown docs-site route: "
      + repr(route)
      + " — known routes: "
      + PAGES.keys().join(", "),
  )
}
