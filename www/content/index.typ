// www/content/index.typ: the site root. The hero shows the tagline, the
// intro and the showcase task. The recipe table comes from www/utils/demo.typ,
// which README.md renders too.
#import "../utils/page.typ": page
#import "../utils/ui.typ": hero, lsp-demo
#import "../utils/demo.typ": recipe-table, tagline
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

#recipe-table((route, body) => link(u(route), body))

= The elephant in the room: Typst vs Markdown + frontmatter

You may ask why Typst and not Markdown. It's a Turing-complete programming
language, which gives us:

+ #strong[Language server autocompletion.] `task` declares every field as a
  named parameter, so your editor completes field names and flags a misspelled
  one.
+ #strong[Shared helpers.] `#import` your own Typst module to fill tasks the
  way you like.
+ #strong[Custom validators.] Bring your own checks by writing a Typst
  function. See #link(u("/self-validating-tasks/"))[Self-validating tasks].

Here's how #link("https://myriad-dreamin.github.io/tinymist/")[tinymist] helps
with autocompletion and highlights issues:

#lsp-demo()

= For people and for agents

`mt ls` prints a table for you to read, and JSON, CSV or a bare list of paths
for scripts. When `mt` refuses a command, it exits non-zero and says what to
fix, so an agent can read the error and correct its own work. See
#link(u("/agents/"))[For AI agents] to drive `mt` from one.
