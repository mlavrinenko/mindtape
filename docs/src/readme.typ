// docs/src/readme.typ — source for the repo-root README.md. Rendered to
// Markdown by `docs/generate-md.sh` via typlite; see docs/src/lib.typ for
// what that render path can and cannot do.
//
// The README is a front door, not a manual. It owes a reader four things, in
// order — what this is, what a task file looks like, how to get it, where the
// docs are — and every second copy of a docs-site page kept here has rotted:
// the install story the Quickstart page derives from the release workflow, a
// platform matrix that undercounted its own releases, seven hand-written page
// blurbs beside seven pages that each already carry one, and a credit for
// artwork this repo stopped shipping.
//
// So no page is restated below. Every site link goes through `www-link`,
// which resolves the route against www/generated/site-pages.json and fails
// this render naming an unknown one. The index groups the pages as the site
// nav does, read from the same manifest, so a page added to NAV or moved to
// another group lands here with no edit.
//
// The opening does not carry a synthetic task example, and does not carry its
// own copy of the real one either. The block lives in www/utils/demo.typ,
// which this file and www/content/index.typ both render: each passes its own
// link helper (`www-link` here, tola's `u()` there) and reads everything else
// — the YAML twin, the trimmed marker, the `x-goals` rule — from
// www/generated/readme-task.json, so the two front doors cannot drift.
// `www/generate-facts.sh` derives that fact (the marker trimmed of the body
// below it, plus the hand-authored Markdown+YAML twin) and drift-checks the
// twin; see that script's own `readme-task.json` section. Whatever
// `@local/mindtape:...` version the quoted file's `#import` names is whatever
// shows here, so this file has no version of its own to keep in sync —
// outdatty.yaml's `typst-pkg-version` group still does not list it, for the
// same reason it never did: a value read off disk cannot drift from itself.
//
// There is no Acknowledgements section. The logo is this repo's own cassette
// mark (www/assets/images/logo.svg), so no third-party artwork credit is owed,
// and Typst, the one dependency a reader of this file meets, is credited in
// the tagline.
//
// The centered header is a deliberate rebuild, not a literal port:
// `<p align="center">` is how a README normally centers its logo, title,
// tagline and doc links, but typlite silently strips every `<p>`
// (docs/src/lib.typ). `<h1 align="center">`/`<h4 align="center">` survive both
// typlite and GitHub's HTML sanitizer (`<center>` survives typlite but is
// stripped by GitHub's sanitizer), so the header below uses those instead.
// This document has no separate `doc-title` ATX `#`, because the centered
// `<h1>` is the title.
#import "lib.typ": www-groups, www-link
#import "/www/utils/demo.typ": demo

#metadata((
  output: "README.md",
)) <mindtape-md>

// The docs-site sections for the index at the end, in the site's own order.
// The site root is left out: its title is "MindTape" over a summary that
// restates this README's opening, and the header's Documentation link is that
// page. A group with no page left after that is dropped.
#let doc-groups = {
  let without-root = group => (
    title: group.title,
    routes: group.routes.filter(route => route != "/"),
  )
  www-groups.map(without-root).filter(group => group.routes.len() > 0)
}

#html.elem("h1", attrs: (align: "center"))[
  #html.elem("img", attrs: (
    src: "https://raw.githubusercontent.com/mlavrinenko/mindtape/main/www/assets/images/logo.svg",
    alt: "MindTape",
    width: "120",
  ))
  #html.elem("br")
  MindTape
]

#html.elem("h4", attrs: (align: "center"))[
  A #link("LICENSE")[MIT]-licensed, daemon-free file-based task tracker powered by #link("https://typst.app/")[Typst].
]

#html.elem("h4", attrs: (align: "center"))[
  #www-link("/")[Documentation] · #www-link("/quickstart/") · #www-link("/comparison/")
]

\-\-\-

MindTape keeps no daemon, database or index: every command evaluates your task
files where they sit, so the repository is the tracker. A task is one Typst
file, its `id` is the filename stem, and links target files — rename one
without updating its dependents and `mt check` says so instead of dangling.

#demo((route, body) => www-link(route, body))

= Installation

```bash
cargo install mindtape-cli
```

#www-link("/quickstart/") covers the prebuilt binaries, `nix run`, building from
source, and a first project from `mt init` to `mt check`.

= Documentation

#list(..doc-groups.map(group => [
  *#group.title*: #group.routes.map(route => www-link(route)).join[ · ]
]))

= Contributing

See #link("CONTRIBUTING.md")[CONTRIBUTING.md].
