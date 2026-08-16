// Site-wide constants for the MindTape docs. Single source of truth for the
// site URL and its path prefix, the page manifest (route -> label, title,
// summary), the nav tree those pages group into, and the Typst package
// version used in every `#import` example (kept in sync with lib/typst.toml;
// outdatty-tracked).

// Typst library version — mirrors lib/typst.toml [package] version. Bump both
// together; the outdatty `typst-pkg-version` group fails the build otherwise.
#let PKG_VERSION = "0.2.0"
#let PKG = "@local/mindtape:" + PKG_VERSION

#let REPO = "https://github.com/mlavrinenko/mindtape"

// The site's public URL, read out of the tola config rather than retyped
// beside it: "keep in sync with tola.toml" is a comment, not a mechanism, and
// this string is what every absolute link into the site is built from
// (docs/src/lib.typ's `www-link`, via www/generated/site-pages.json).
//
// Typst resolves a relative path against the FILE, not the root, so this one
// read works under both roots this module is evaluated with — `--root www`
// for the site build, `--root .` for www/generate-facts.sh's site-pages
// driver. The trailing-slash trim is what lets `SITE_URL + route` compose
// cleanly for a route that already opens with `/`.
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

// Grouped sidebar, and the page manifest it is built from. Each item's href
// is the page's ROUTE — site-root-relative, u() prefixes it — and carries the
// three other facts that page owns:
//
//   label   the sidebar entry (short; fits a nav column)
//   title   the page's own <h1> and <title> (index's differs from its label
//           on purpose: "Overview" in the nav, "MindTape" on the page)
//   summary one sentence, rendered as the page lede, as its <meta
//           name="description">, and as the README's page list — so it has to
//           read well standing completely alone, away from the page
//
// www/content/*.typ used to hand-write title and summary a second time in its
// own `page.with(...)` header; utils/page.typ now resolves both from here by
// the page's `active:` route.
#let NAV = (
  (
    title: "Start",
    items: (
      (
        label: "Overview",
        href: "/",
        title: "MindTape",
        home: true,
        summary: "A daemon-free task tracker whose tasks are Typst files, live-evaluated on every command.",
      ),
      (
        label: "Quickstart",
        href: "/quickstart/",
        title: "Quickstart",
        summary: "Install MindTape, scaffold a project, and file your first task.",
      ),
    ),
  ),
  (
    title: "Guide",
    items: (
      (
        label: "Concepts",
        href: "/concepts/",
        title: "Concepts",
        summary: "The model behind MindTape: files as tasks, live evaluation, identity by filename, and one extension seam.",
      ),
      (
        label: "Task DSL",
        href: "/dsl/",
        title: "Task DSL",
        summary: "Every field a task file can declare, and what the prelude versus the Rust layer enforce.",
      ),
      (
        label: "Templates & slots",
        href: "/templates/",
        title: "Templates & slots",
        summary: "How mt add scaffolds a task from a Typst template whose values are slot() calls.",
      ),
      (
        label: "Derived tags",
        href: "/derived-tags/",
        title: "Derived tags",
        summary: "Tasks inherit tags from their location under a root, without those tags ever being written to the file.",
      ),
    ),
  ),
  (
    title: "Reference",
    items: (
      (
        label: "CLI",
        href: "/cli/",
        title: "CLI reference",
        summary: "Every mt command, grouped, with the deep topics linked out to their own pages.",
      ),
      (
        label: "Filter language",
        href: "/filters/",
        title: "Filter language",
        summary: "The expression language mt ls and mt rm use to select tasks: fields, operators, functions, and dependency predicates.",
      ),
      (
        label: "Priority",
        href: "/priority/",
        title: "Priority",
        summary: "Priority is a number on a configured scale, set by hand or scored by a framework; difficulty is its twin.",
      ),
      (
        label: "Configuration",
        href: "/config/",
        title: "Configuration",
        summary: "How .mindtape/config.toml overlays a defaults floor through a four-layer merge, and the sections it can set.",
      ),
      (
        label: "Canonical schema",
        href: "/schema/",
        title: "Canonical schema",
        summary: "The task-JSON schema mt ls --json and mt import share — the contract a jq adapter codes against.",
      ),
      (
        label: "Import & export",
        href: "/import-export/",
        title: "Import & export",
        summary: "Bring tasks in from another tracker with mt import and jq, and reshape them back out with the same tools.",
      ),
    ),
  ),
  (
    title: "More",
    items: (
      (
        label: "For AI agents",
        href: "/agents/",
        title: "For AI agents",
        summary: "What makes MindTape legible to a coding agent: structured JSON, describable schemas, honest exit codes, and file-based state.",
      ),
      (
        label: "Internals",
        href: "/internals/",
        title: "Internals",
        summary: "How MindTape is built: the crate layout, the content-hash cache that keeps live evaluation cheap, and what differs per platform.",
      ),
      (
        label: "Comparison",
        href: "/comparison/",
        title: "Comparison",
        summary: "How MindTape compares, feature by feature, to the other local file-and-CLI task trackers, and when one of them fits better.",
      ),
    ),
  ),
)

// The same pages, flattened to a route -> facts lookup. The nav's grouping is
// presentation; every consumer that resolves a page BY ROUTE (utils/page.typ,
// www/generated/site-pages.json and through it docs/src/lib.typ's www-link)
// wants this shape instead, and deriving it here rather than writing it out
// keeps the two from ever disagreeing about which pages exist.
#let PAGES = {
  let out = (:)
  for group in NAV {
    for item in group.items {
      let entry = (:)
      for (k, v) in item {
        if k != "href" { entry.insert(k, v) }
      }
      out.insert(item.href, entry)
    }
  }
  out
}

// Resolve one page by route, or fail naming it. Panicking here — rather than
// returning none and letting a caller render an empty title — is the point:
// a route is a string typed by hand in a page header or in doc prose, and a
// renamed page otherwise degrades into a silent 404 nothing checks. The
// message lists the known routes because the fix is always one of them.
#let page-meta(route) = {
  if type(route) == str and route in PAGES { return PAGES.at(route) }
  panic(
    "unknown docs-site route: "
      + repr(route)
      + " — known routes: "
      + PAGES.keys().join(", "),
  )
}
