// Site-wide constants for the MindTape docs. Single source of truth for the
// nav tree, the Pages path prefix, and the Typst package version used in every
// `#import` example (kept in sync with lib/typst.toml; outdatty-tracked).

// Typst library version — mirrors lib/typst.toml [package] version. Bump both
// together; the outdatty `typst-pkg-version` group fails the build otherwise.
#let PKG_VERSION = "0.2.0"
#let PKG = "@local/mindtape:" + PKG_VERSION

#let REPO = "https://github.com/mlavrinenko/mindtape"

// GitHub Pages project page is served under /mindtape. Keep in sync with
// `site.info.url` in tola.toml. u() builds internal page links; assets stay
// source-relative (/assets/...) — tola applies the prefix to those on build.
#let PREFIX = "/mindtape"
#let u(path) = PREFIX + path

#let SITE = (
  title: "MindTape",
  tagline: "Tasks as Typst files. No daemon, no database, no lock-in.",
)

// Grouped sidebar. Each item's href is site-root-relative; u() prefixes it.
#let NAV = (
  (
    title: "Start",
    items: (
      (label: "Overview", href: "/"),
      (label: "Quickstart", href: "/quickstart/"),
    ),
  ),
  (
    title: "Guide",
    items: (
      (label: "Concepts", href: "/concepts/"),
      (label: "Task DSL", href: "/dsl/"),
      (label: "Templates & slots", href: "/templates/"),
      (label: "Derived tags", href: "/derived-tags/"),
    ),
  ),
  (
    title: "Reference",
    items: (
      (label: "CLI", href: "/cli/"),
      (label: "Filter language", href: "/filters/"),
      (label: "Priority", href: "/priority/"),
      (label: "Configuration", href: "/config/"),
      (label: "Canonical schema", href: "/schema/"),
      (label: "Import & export", href: "/import-export/"),
    ),
  ),
  (
    title: "More",
    items: (
      (label: "For AI agents", href: "/agents/"),
      (label: "Comparison", href: "/comparison/"),
    ),
  ),
)
