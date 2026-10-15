#import "../utils/page.typ": page
#import "../utils/ui.typ": diagram, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../utils/config-ref.typ": config-keys-table, config-values-table

#let meta = (
  route: "/links/",
  label: "Task links",
  title: "Task links",
  summary: "Typed links between tasks, the built-in and custom link kinds, and what `mt check` refuses.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The task and config blocks are slices of examples/links/, which
// `just check-examples` validates. The transcripts are runs against it and a
// copy of it, made by www/generate-model.sh, which also draws its graph.
#let schema = json("../generated/config-schema.json")
#let slices = checked-slices(
  json("../generated/links.json"),
  "links",
  ("depends-on-related", "supersedes-and-edge", "duplicates"),
)
#let kinds = checked-slices(
  json("../generated/links-config.json"),
  "links-config",
  ("kinds",),
)

Use `links:` to say how a task relates to others: it depends on one, it's
part of another, it replaces a third. Each edge has a kind and a target. The
target is the path from this file to the other task's file, as
#link(u("/concepts/"))[Concepts] explains. `mt check` reports a target that
names no task, a cycle, and an edge missing a note its kind requires.

= Writing edges by hand

Each kind has a helper that takes one or more targets and an optional
trailing note. A helper returns an array, so you join several with `+`:

#raw(
  render-slices(slices, (
    (
      "depends-on-related",
      "three kinds joined with +, and the note related requires",
    ),
    ("duplicates", "the duplicate points at the task it repeats"),
  )),
  lang: "typ",
  block: true,
)

An array of helper calls means the same as `+`. For a kind with no helper,
such as one you declare in config, use `edge("<kind>", ...)`:

#raw(
  render-slices(slices, (
    (
      "supersedes-and-edge",
      "an array mixing a built-in kind with the project's discharged-by",
    ),
  )),
  lang: "typ",
  block: true,
)

= The built-in kinds

#table(
  columns: 2,
  table.header([Kind], [A `kind("target.typ")` edge means]),
  [`depends-on`],
  [This task can't start until the target is finished. It's the only built-in kind that makes a task blocked. `mt check` refuses a cycle of these edges.],

  [`parent`],
  [This task is part of the target. `mt check` refuses a cycle of these edges.],

  [`related`],
  [The two tasks are connected. The edge needs a note that says how.],

  [`supersedes`], [This task replaces the target.],
  [`duplicates`], [This task is the same work as the target.],
  [`recurred-from`],
  [The target is the previous occurrence of this recurring task. `mt recur` writes it. See #link(u("/recurring/"))[Recurring tasks].],
)

= Parents and progress

Each task gets two computed fields from the `parent` edges that point at it.
`children` counts those tasks. `progress` is the share of them in a
#link(u("/statuses/#the-default-legend"))[terminal] status, so a cancelled
child counts as finished. In the table, the `progress` column shows the
counts:

#terminal("links-progress")

A filter, `--sort`, `--json` and `--format csv` read `progress` as a fraction
from 0 to 1, so `mt ls 'progress == 1 && !terminal'` lists the parents ready
to close. A task without children has no `progress`. It prints as `null` in
JSON and an empty cell in the table, and a comparison such as `progress < 1`
never matches it.

`mt flip` won't move a parent with unfinished children into a status that's
both terminal and `unblocked`. See #link(u("/statuses/"))[Statuses].

= Writing edges with `mt link`

Working out a relative path by hand is fiddly. `mt link <task> <kind>
<target>` adds the edge and writes the path for you. You name a target the way
you name any task, by its #link(u("/concepts/#identity"))[stem], the
#link(u("/cli/ls/"))[stem hash] `mt ls` prints, or a path, and a path starts
at the linking file. A stem or a hash finds the task in any folder, and
`mt link` writes the path from the linking file to it. If the edge is already
there, `mt link` leaves it alone:

#terminal("links-write")

To add many edges at once, `mt link --apply <file>` merges a map of task to
kind to targets, from YAML or JSON. The #link(u("/cli/link/"))[`mt link` page]
lists its other flags.

= Project-declared kinds

`[[tasks.links.kinds]]` adds your own kinds next to the six built-ins. A task
writes one with `edge("<kind>", ...)`, like the `discharged-by` edge above.
`examples/links/` declares these:

#raw(
  render-slices(
    kinds,
    (
      (
        "kinds",
        "three kinds whose targets live outside the backlog, and one that allows a task one edge",
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

A kind whose `validate` isn't `local-task-path` points outside the backlog,
at a URL, a commit or a ticket. `mt check` reports a target that doesn't fit
the validator, and names the kind, the validator and the value. `mt` never
resolves such a target to a task. It leaves it out of `max`, of cycle checks
and of readiness, even when the kind sets `blocking`. `mt graph` draws each
such target as a leaf node.

= The graph

To see the edges, `mt graph` prints the tasks as a Graphviz DOT graph, or as
a Mermaid flowchart with `--format mermaid`. Pass a filter to graph only the
tasks that match, and `--no-terminal` to leave out the finished ones. Each
node shows a task's title and status. A ready task is green, a blocked one red
and a finished one gray. Here's the graph of `examples/links/`, the project
this page quotes:

#diagram("links-graph", [`mt graph` on `examples/links/`, drawn by Graphviz])
