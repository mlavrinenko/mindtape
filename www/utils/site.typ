// Site-wide constants for the docs: the product name, the site URL and its
// path prefix, and the nav tree's group names and reading order. The name, the
// site URL and the Typst package version are read from the files that declare
// them. The page manifest (route -> label, title, summary) is `PAGES`, read
// back off every page's own `<page-meta>` block by www/generate-site-pages.sh,
// so this file never restates a label, a title or a summary.

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

// The product's name, read from the site title in tola.toml, so a doc source
// writes `#mindtape` and never types the name. `just check-doc-prose` fails on
// a typed one.
#let mindtape = TOLA.site.info.title

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

// A button beside a page's title that copies the page at `route` as Markdown,
// the copy www/generate-md.sh writes beside its HTML. The script in
// www/templates/layout.typ copies it, and CSS shows `data-tip` on hover.
#let md-copy-button(route) = html.elem("button", attrs: (
  class: "md-copy",
  type: "button",
  "aria-label": "Copy page as Markdown",
  "data-tip": "Copy page as Markdown",
  "data-md": PREFIX + md-path(route),
))

#let SITE = (
  title: mindtape,
  tagline: "Tasks as code. One Typst file per task, checked against your own rules.",
  pitch: "Write rules in `.mindtape/config.toml`, and `mt check` runs them over every task and commit.",
)

// Grouped sidebar: group title, and its members' reading order as routes.
// A group is named for the reader's job, not for the page's origin, so a
// reader scans one short group instead of every page:
//
//   - Start: a first look, and the one idea the rest rests on.
//   - Recipes: an engineer with a problem wants the config that solves it.
//   - Tasks: authoring a task file.
//   - Fields: setting the values a task's fields may take, such as the
//     statuses, the priority scale, the estimate units and the named sets.
//   - Checks: setting what `mt` refuses, in tasks, commits and flips.
//   - Reference: looking one thing up in a page that lists every command,
//     operator, config key or JSON field.
//   - Integrate: wiring `mt` into a repo, a tool or another tracker.
//
// No group holds more than six pages, because a longer run makes the reader
// read every label again. No single page can declare this fact. Spreading it
// across the page files as `order:` integers would hide the sidebar's shape
// and let two pages claim one slot. Every other fact about a nav entry
// (label, title, summary, home) is declared on the page itself and resolved
// from `PAGES` below by route.
#let NAV = (
  (
    title: "Start",
    items: ("/", "/quickstart/", "/concepts/", "/comparison/"),
  ),
  (
    title: "Recipes",
    items: (
      "/split-oversized-tasks/",
      "/require-a-task-shape/",
      "/start-titles-with-a-verb/",
      "/tie-work-to-goals/",
      "/limit-work-in-progress/",
      "/wait-on-upstream/",
      "/trace-commits-to-tasks/",
    ),
  ),
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
    title: "Fields",
    items: ("/statuses/", "/priority/", "/estimates/", "/named-sets/"),
  ),
  (
    title: "Checks",
    items: ("/rules/", "/rule-packs/", "/commit-rules/", "/hooks/"),
  ),
  (
    title: "Reference",
    items: ("/cli/", "/filters/", "/config/", "/schema/"),
  ),
  (
    title: "Integrate",
    items: (
      "/self-validating-tasks/",
      "/agents/",
      "/import-export/",
      "/internals/",
    ),
  ),
)

// The CLI commands by index group, in the order the /cli/ index lists them.
// www/content/cli.typ renders one section per group, and the sidebar lists the
// same commands in the same order under its CLI entry, so a command is placed
// here once. cli-ref.typ's `cli-gate` fails the build when a command in
// www/generated/cli.json is missing here or listed twice.
#let CLI_GROUPS = (
  ("Files", ("add", "import", "mv", "go", "eval")),
  ("Fields", ("set",)),
  ("Listing", ("ls", "find")),
  ("Graph", ("graph",)),
  ("Links", ("link",)),
  ("Views & dictionaries", ("views", "dicts")),
  ("Status", ("flip", "review")),
  ("Recurring tasks", ("recur",)),
  ("Validate & inspect config", ("check", "config")),
  ("Retire", ("rm",)),
  ("Setup", ("init", "install", "allow", "worktrees", "preset")),
)

#let cli-route(name) = "/cli/" + name + "/"

// The command pages' routes in index order.
#let CLI_ROUTES = CLI_GROUPS.map(g => g.at(1)).flatten().map(cli-route)

// Pages the sidebar lists under a `NAV` entry, by the entry's route. Only the
// CLI entry has any, and its commands come from the index's own list.
#let NAV_CHILDREN = ("/cli/": CLI_ROUTES)

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
  let body = eval(text, mode: "markup", scope: (mindtape: mindtape))
  if plain { _plain-text(body) } else { body }
}
