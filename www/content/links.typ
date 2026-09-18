#import "../utils/page.typ": page
#import "../utils/site.typ": u
#import "../generated/just.typ" as just

#let meta = (
  route: "/links/",
  label: "Links",
  title: "Task links",
  summary: "Typed edges between tasks: the built-in kinds, project-declared kinds with their own target validation, and what mt check refuses.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

One field, `links:`, holds every typed edge a task has to another. An edge is a
kind plus one or more targets; a target is a filename stem, and a kind is one of
six built-ins or a project-declared one. Every one of them is checked on load:
a target that does not resolve, a cycle, or a note a kind requires and does not
get fails #link(u("/cli/"))[`mt check`].

Everything below is sliced out of `examples/links/`, a real project that
#just.check-examples() validates on every gate run, so no block here can say
something the DSL stopped doing.

= The six built-in kinds

#table(
  columns: 2,
  table.header([Kind], [A `kind("target.typ")` edge means]),
  [`depends-on`],
  [This task cannot start before the target finishes. The only kind that feeds `ready` / `blocked` by default.],

  [`parent`], [This task belongs to a larger one.],
  [`related`],
  [Connected, but not a dependency. The note is required — that is the point of the kind.],

  [`supersedes`], [The target is obsolete work this task replaces.],
  [`duplicates`], [This task is the same work as the target, filed twice.],
  [`recurred-from`],
  [The previous occurrence in a recurring series — see #link(u("/dsl/"))[Task DSL].],
)

#let links-fact = json("../generated/links.json")

#let parts = (
  (
    "depends-on-related",
    "`depends-on` and `parent`, with the note `related` requires",
  ),
  (
    "supersedes-and-edge",
    "the array spelling, mixing a built-in with the project's own kind",
  ),
  (
    "duplicates",
    "the same work filed twice; the duplicate points at the canonical task",
  ),
)

#let expected = parts.map(p => p.at(0))
#assert(
  expected.all(id => id in links-fact.slices)
    and links-fact.slices.keys().len() == expected.len(),
  message: "www/generated/links.json slices ("
    + links-fact.slices.keys().join(", ")
    + ") do not match this page's list ("
    + expected.join(", ")
    + "); regenerate the docs and fix whichever side moved",
)

#let block = (
  parts
    .map(p => {
      let s = links-fact.slices.at(p.at(0))
      "// " + s.file + " — " + p.at(1) + "\n" + s.code
    })
    .join("\n\n")
)

#raw(block, lang: "typ", block: true)

A helper is variadic, and `+` combines helpers because each returns an array —
the same value every other multi-valued field takes. The array spelling shows
the same thing explicitly. A target may keep or drop its `.typ`; edges dedupe
on kind and target, so re-applying either form writes nothing new.

= Project-declared kinds

The `edge("discharged-by", …)` line above is a kind this project declared in
`[[tasks.links.kinds]]`, where `validate` picks how its targets are checked: a
URL, a git SHA, a UUID, or a regex. A target that cannot be a local task never
pretends to be one, and never joins the task graph.
#link(u("/config/"))[Configuration] has the table of variants and a declaration
to copy.

= Writing edges

`mt` writes edges back into a task file with #link(u("/cli/"))[`mt link`]: it
upserts single edges and applies a whole adjacency map — stem, kind, targets —
in one call, serializing through the same helpers, so a written file reads as
if it were hand-authored.
