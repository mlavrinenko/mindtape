// www/content/index.typ: the site root. The hero shows the tagline, the
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

A task file is Typst code, not Markdown. Here's what you get for that.

- #strong[Field completion.] `task` declares every field as a named parameter,
  so the Typst language server completes field names in your editor.
- #strong[Values checked on load.] `mt` evaluates every task file on every
  command. A misspelled field, a month 13 or text where a number belongs fails,
  and the error names the file.
- #strong[Shared helpers.] A task file can `#import` your own Typst module, so
  every task that imports it uses the same definitions.
- #strong[Your own validator.] A rule can call a Typst function you write. See
  #link(u("/self-validating-tasks/"))[Self-validating tasks].

Here's what tinymist, the Typst language server, returned when this site was
built. On a blank line inside `task.with(...)`, it offers each field the file
doesn't set yet:

#lsp-completion()

Write `titel:` by mistake and tinymist reports the unknown field as it compiles
the file. Your editor marks it under `titel`:

#lsp-diagnostics()

= For people and for agents

`mt ls` prints a table for you to read, and JSON, CSV or a bare list of paths
for scripts. When `mt` refuses a command, it exits non-zero and says what to
fix, so an agent can read the error and correct its own work. See
#link(u("/agents/"))[For AI agents] to drive `mt` from one.
