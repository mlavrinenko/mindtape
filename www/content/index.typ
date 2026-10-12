// www/content/index.typ: the site root. The hero shows the name, the lede and
// the showcase task. The recipe table and the Typst section come from
// www/utils/demo.typ, which README.md renders too.
#import "../utils/page.typ": page
#import "../utils/ui.typ": hero, lsp-demo
#import "../utils/demo.typ": recipe-table, tagline, typst-section
#import "../utils/site.typ": mindtape, u

#let meta = (
  route: "/",
  label: "Overview",
  title: mindtape,
  summary: tagline,
  home: true,
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

#hero()

#let site-link = (route, body) => link(u(route), body)

#recipe-table(site-link)

#typst-section(site-link)

Here's how #link("https://myriad-dreamin.github.io/tinymist/")[tinymist] helps
with autocompletion and highlights issues:

#lsp-demo()

= For people and for agents

`mt ls` prints a table for you to read, and JSON, CSV or a bare list of paths
for scripts. When `mt` refuses a command, it exits non-zero and says what to
fix, so an agent can read the error and correct its own work. See
#link(u("/agents/"))[For AI agents] to drive `mt` from one.
