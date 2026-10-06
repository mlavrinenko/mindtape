// Site-wide constants for the MindTape docs: the site URL and its path
// prefix, and the nav tree's group names and reading order. The site URL and
// the Typst package version are read from the files that declare them. The
// page manifest (route -> label, title, summary) is `PAGES`, read back off
// every page's own `<page-meta>` block by www/generate-site-pages.sh, so this
// file never restates a label, a title or a summary.

#import "mode.typ": MARKDOWN

// Typst library version, read from lib/typst.toml through the crate map, so a
// bump reaches the site with no hand edit. The site builds under `--root www`,
// where Typst refuses a read of lib/typst.toml. www/generate.sh writes
// crate-map.json before anything imports this file.
#let PKG_VERSION = json("../generated/crate-map.json").at("typst-lib").version
#let PKG = "@local/mindtape:" + PKG_VERSION

#let REPO = "https://github.com/mlavrinenko/mindtape"

// The site's public URL, read out of the tola config rather than retyped
// beside it: "keep in sync with tola.toml" is a comment, not a mechanism, and
// this string is what every absolute link into the site is built from
// (docs/src/lib.typ's `www-link`, via www/generated/site-pages.json).
//
// Typst resolves a relative path against the file, not the root, so this one
// read works under both roots this module is evaluated with — `--root www`
// for the site build and for www/generate-site-pages.sh's page queries,
// `--root .` for that same script's site-meta driver. The trailing-slash trim
// is what lets `SITE_URL + route` compose cleanly for a route that already
// opens with `/`.
#let TOLA = toml("../tola.toml")
#let SITE_URL = TOLA.site.info.url.trim("/", at: end)

// GitHub Pages serves the site under the URL's path ("/mindtape"). u()
// prefixes internal page links with it. Assets stay source-relative
// (/assets/...), because tola prefixes those on build.
#let _url-path(url) = {
  let parts = url.split("://").last().split("/")
  if parts.len() < 2 { "" } else { "/" + parts.slice(1).join("/") }
}
#let PREFIX = _url-path(SITE_URL)

// The Markdown copy of a page route: `/cli/ls/#flags` becomes
// `/cli/ls/index.md#flags`. In the Markdown copy (mode.typ), u() links to the
// other pages' copies by full URL, because an agent often reads the file away
// from the site.
#let md-path(path) = {
  let (base, ..frag) = path.split("#")
  let file = if base.ends-with("/") { base + "index.md" } else { base }
  if frag == () { file } else { file + "#" + frag.join("#") }
}

#let u(path) = if MARKDOWN { SITE_URL + md-path(path) } else { PREFIX + path }

#let SITE = (
  title: "MindTape",
  tagline: "A task tracker in your repo that enforces your team's rules on every task and commit.",
)

// Grouped sidebar: group title, and its members' reading order as routes.
// The groups follow the reader's job, not the page's origin: Problems for an
// engineer who has a problem and wants the config that solves it, Start for a
// first look, Model for the one idea the rest rests on, Tasks for authoring a
// task file, Reference for looking one thing up, Integrate for wiring
// MindTape into a repo or a tool. No single page can declare this fact:
// spreading it across the page files as `order:` integers would make the
// sidebar's shape invisible and let two pages claim one slot (see the
// umbrella task's "Out of scope"). Every other fact about a nav entry (label,
// title, summary, home) is declared on the page itself and resolved from
// `PAGES` below by route.
#let NAV = (
  (
    title: "Problems",
    items: (
      "/split-oversized-tasks/",
      "/require-a-task-shape/",
      "/gate-done-on-checks/",
      "/one-task-at-a-time/",
      "/trace-commits-to-tasks/",
    ),
  ),
  (title: "Start", items: ("/", "/quickstart/", "/comparison/")),
  (title: "Model", items: ("/concepts/",)),
  (
    title: "Tasks",
    items: (
      "/task-files/",
      "/dsl/",
      "/links/",
      "/recurring/",
      "/templates/",
      "/derived-tags/",
    ),
  ),
  (
    title: "Reference",
    items: (
      "/cli/",
      "/filters/",
      "/priority/",
      "/config/",
      "/statuses/",
      "/rules/",
      "/rule-packs/",
      "/commit-rules/",
      "/named-sets/",
      "/hooks/",
      "/estimates/",
      "/schema/",
      "/import-export/",
    ),
  ),
  (
    title: "Integrate",
    items: ("/self-validating-tasks/", "/agents/", "/internals/"),
  ),
)

// The page manifest, route -> {label, title, summary, home?}, which
// www/generate-site-pages.sh builds from every page's `<page-meta>` block. It
// is empty during that script's bootstrap pass, because compiling a page reads
// this file before the query that fills it.
#let PAGES = json("../generated/site-pages.json").pages

// Resolve one page by route. Three outcomes, not two:
//
//   - `PAGES` is empty (the bootstrap pass, before a single page has been
//     queried yet): return `none`. Panicking here would make the bootstrap
//     pass itself un-compilable, which is exactly the chicken-and-egg
//     www/generate-site-pages.sh exists to break.
//   - `PAGES` is non-empty and has the route: return its entry.
//   - `PAGES` is non-empty and lacks the route: panic naming it. A silent skip
//     here is the exact failure this whole change removes — a route is a
//     string typed by hand in a page header, in `NAV` above, or in doc prose,
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

// A page summary is one sentence of Typst markup, with code in backticks.
// `render-summary` turns one into content for the page's lede. With
// `plain: true` it returns plain text for `<meta name="description">`, where
// markup cannot go.
#let _plain-text(it) = {
  if type(it) == str { return it }
  if it.has("text") { return it.text }
  if it.has("children") { return it.children.map(_plain-text).join("") }
  if it.has("body") { return _plain-text(it.body) }
  if it.func() == smartquote { return if it.double { "\"" } else { "'" } }
  " "
}
#let render-summary(text, plain: false) = {
  if text == none { return none }
  let body = eval(text, mode: "markup")
  if plain { _plain-text(body) } else { body }
}
