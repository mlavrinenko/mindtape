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
// this render naming an unknown one, and the page list reads each page's own
// summary out of that same manifest — a page added, renamed or re-summarised
// on the site lands here with no edit.
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
// There is deliberately no Acknowledgements section. It existed to carry an
// ISC attribution for a Lucide brain icon; the logo is now this repo's own
// hand-authored cassette mark (www/assets/images/logo.svg), so no third-party
// artwork credit is owed, and Typst — the one dependency a reader of this
// file actually meets — is credited in the tagline where they meet it.
//
// The centered header is a deliberate rebuild, not a literal port:
// `<p align="center">` is how a README normally centers its logo, title,
// tagline and doc links, but typlite silently strips every `<p>`
// (docs/src/lib.typ). `<h1 align="center">`/`<h4 align="center">` survive both
// typlite and GitHub's HTML sanitizer (`<center>` survives typlite but is
// stripped by GitHub's sanitizer), so the header below uses those instead —
// closest achievable to the original centered look, not a byte-for-byte
// match. This document therefore has no separate `doc-title` ATX `#`: the
// centered `<h1>` IS the title, exactly as in the hand-written original.
#import "lib.typ": fact, www-link, www-summary
#import "/www/utils/demo.typ": demo

#metadata((
  output: "README.md",
)) <mindtape-md>

// Every docs-site page except the site root. `www-link` renders a page's own
// title, and the root's title is "MindTape" over a summary that restates this
// README's own opening line — an index entry pointing the reader back at
// where they already are. The header's Documentation link is that page.
//
// Read through `fact` rather than written out, so a page added to
// www/utils/site.typ's NAV appears here on the next render and one removed
// stops being linked. Each route still goes through `www-link` /
// `www-summary` below, so nothing can reach the output with an empty title.
#let doc-routes = fact("site-pages").pages.keys().filter(route => route != "/")

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

#www-link("/quickstart/") covers the prebuilt binaries, `nix run`, and building
from source.

= Usage

One task, start to finish:

```bash
mt install                                         # once per machine: put the Typst prelude where Typst looks
mt init                                            # once per project: write .mindtape/config.toml
mt add --set title="Wire OAuth" --set priority=7   # file a task from the default template
mt ls 'status != "done" && priority >= 7'          # query it with a filter expression
mt flip done wire-oauth                            # set its status, dated today
mt check                                           # validate the whole project
```

The docs site has the rest:

#list(..doc-routes.map(route => [
  #www-link(route) — #www-summary(route)
]))

= Contributing

See #link("CONTRIBUTING.md")[CONTRIBUTING.md].
