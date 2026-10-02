#import "../utils/page.typ": page
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../utils/config-ref.typ": config-keys-table, config-values-table
#import "../generated/just.typ" as just

#let meta = (
  route: "/links/",
  label: "Links",
  title: "Task links",
  summary: "Typed links between tasks, the built-in and custom link kinds, and what `mt check` refuses.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

One field, `links:`, holds every typed edge a task has to another. An edge is a
kind plus one or more targets; a target is a filename stem, and a kind is one of
six built-ins or a project-declared one. Every one of them is checked on load:
a target that does not resolve, a cycle, or a note a kind requires and does not
get fails #link(u("/cli/"))[`mt check`].

No block here is hand-written. Each is a slice of `examples/links/`, taken by
the docs generator out of the project #just.check-examples() validates, so a
block cannot describe a shape the DSL stopped accepting.

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
  [The previous occurrence in a recurring series — see #link(u("/recurring/"))[Recurring tasks].],
)

#let schema = json("../generated/config-schema.json")
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

#let slices = checked-slices(
  json("../generated/links.json"),
  "links",
  parts.map(p => p.at(0)),
)

#raw(render-slices(slices, parts), lang: "typ", block: true)

= Parents and progress

A `parent` edge makes this task one of the target's children. Every task
carries two computed fields over those edges: `children`, the count of tasks
naming it as their parent, and `progress`, the share of them in a terminal
status. A cancelled child counts as finished — nothing is left to wait for.

```sh
mt ls 'children > 0' --columns title,status,progress
mt ls 'progress == 1 && !terminal'   # parents ready to close
```

`progress` is one value in two spellings. The filter language, `--sort`,
`--json` and `--format csv` carry the fraction `0..1`; the table's `progress`
column renders the counts behind it, `3/5`. A task with no children has no
progress at all — `null` in JSON, a blank cell, and absent to a filter, so
`progress < 1` never matches a leaf.

Closing a parent over unfinished children is refused: a status both
`terminal` and #link(u("/statuses/"))[`unblocked = true`] requires every child
finished, `mt flip` names the ones that are not, and `--force` overrides.

A helper is variadic, and `+` combines helpers because each returns an array —
the same value every other multi-valued field takes. The array spelling shows
the same thing explicitly. A target may keep or drop its `.typ`; edges dedupe
on kind and target, so re-applying either form writes nothing new.

= Project-declared kinds

`[[tasks.links.kinds]]` adds edge kinds beside the six built-ins, and a task
writes one with `edge("<kind>", …)`, as the `discharged-by` line above does.
These are the kinds `examples/links/` declares:

#let kinds = checked-slices(
  json("../generated/links-config.json"),
  "links-config",
  ("kinds",),
)

#raw(
  render-slices(
    kinds,
    (
      (
        "kinds",
        "three kinds whose targets live outside the backlog, one that caps a task to one edge",
      ),
    ),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

#config-keys-table(schema, "tasks.links.kinds")

`validate` takes one of these values:

#config-values-table(schema, "TargetValidator")

Any `validate` other than `local-task-path` says the targets are artifacts
outside the backlog. `mt` never resolves such a target to a task, never counts
it against `max`, and never follows it in a cycle check or a readiness check,
even with `blocking` set. `mt check` reports a target that does not fit the
validator, naming the kind, the validator and the value. `mt graph` draws each
such target as a leaf node of its own.

= Writing edges

`mt` writes edges back into a task file with #link(u("/cli/"))[`mt link`]: it
upserts single edges and applies a whole adjacency map — stem, kind, targets —
in one call, serializing through the same helpers, so a written file reads as
if it were hand-authored. A bare target stem names a task in any folder and is
written as the path from the linking task to it (`../cli/other.typ`), so the
file still resolves without `mt`.
