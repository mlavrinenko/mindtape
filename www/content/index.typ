// www/content/index.typ — the site root. The hero shows the tagline, the
// intro and the showcase task. The problem list comes from www/utils/demo.typ,
// which README.md renders too.
#import "../utils/page.typ": page
#import "../utils/ui.typ": hero, lsp-completion, lsp-diagnostics
#import "../utils/demo.typ": solves, tagline
#import "../utils/site.typ": u

#let meta = (
  route: "/",
  label: "Overview",
  title: "MindTape",
  summary: tagline,
  home: true,
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

#hero()

#solves((route, body) => link(u(route), body))

= Why Typst

A task file is Typst code. Over a Markdown file, that gives you four things.

- #strong[Field completion.] `task` declares every field as a named parameter,
  so the Typst language server completes field names in your editor.
- #strong[Values checked on load.] `mt` evaluates every task file on every
  command. A misspelled field, a month 13 or a text where a number belongs
  fails with the file named.
- #strong[Shared helpers.] A task file can `#import` your own Typst module,
  and every task that imports it uses the same definitions.
- #strong[Your own validator.] A rule can call a Typst function you write, as
  #link(u("/self-validating-tasks/"))[Self-validating tasks] shows.

The two blocks below are what tinymist, the Typst language server, returned
when this site was built. On a blank line inside `task.with(...)`, it offers
each field the file does not set yet.

#lsp-completion()

On a file that writes `titel:`, tinymist reports the unknown slot as it
compiles the file. The error sits at the prelude's check, so an editor lists
it among its problems.

#lsp-diagnostics()

= Built for humans and agents

`mt ls` prints a table for people, and JSON, CSV or a bare path list for
scripts. A refused command exits non-zero and prints what to fix, so an agent
can read the error and correct its work.
#link(u("/agents/"))[For AI agents] shows how to drive `mt` from an agent.
