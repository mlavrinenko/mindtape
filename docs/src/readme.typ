// docs/src/readme.typ — the source of README.md, rendered by typlite.
//
// The opening comes from www/utils/demo.typ, which the site root renders too.
// Every site link goes through `www-link`, which fails the render on a route
// the page manifest lacks. The header uses `<h1 align>` and `<h4 align>`,
// because typlite drops `<p>` and GitHub strips `<center>`. The index groups
// the pages as the site nav does, minus the pages the opening already links.
#import "lib.typ": www-groups, www-link
#import "/www/utils/demo.typ": intro, problem-routes, showcase, solves, tagline
#import "/www/utils/release.typ": cli-crate

#metadata((
  output: "README.md",
)) <mindtape-md>

#let doc-groups = {
  let linked = ("/", ..problem-routes)
  let unlinked = group => (
    title: group.title,
    routes: group.routes.filter(route => route not in linked),
  )
  www-groups.map(unlinked).filter(group => group.routes.len() > 0)
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

#html.elem("h4", attrs: (align: "center"))[#tagline]

#html.elem("h4", attrs: (align: "center"))[
  #www-link("/")[Documentation] · #www-link("/quickstart/")
]

\-\-\-

#intro

#showcase

#solves((route, body) => www-link(route, body))

= Install

// docs:hand-typed the crate name comes from the release workflow, and the build does not run cargo install
#raw("cargo install " + cli-crate, lang: "sh", block: true)

#www-link("/quickstart/") covers prebuilt binaries, `nix run` and a first
project from `mt init` to `mt check`.

= Documentation

#list(..doc-groups.map(group => [
  *#group.title*: #group.routes.map(route => www-link(route)).join[ · ]
]))

= Contributing

See #link("CONTRIBUTING.md")[CONTRIBUTING.md].

= License

#link("LICENSE")[MIT].
