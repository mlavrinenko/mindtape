#import "../utils/page.typ": page
#import "../utils/ui.typ": diagram, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../utils/config-ref.typ": config-keys-table, config-values-table

#let meta = (
  route: "/links/",
  label: "Links",
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

The `links:` field holds a task's typed edges to other tasks. Each edge has a
kind and a target. The target is the path from this file to the other task's
file, as #link(u("/concepts/"))[Concepts] explains. `mt check` reports a
target that names no task, a cycle, and an edge that lacks a note its kind
requires.

= Writing edges by hand

Each kind has a helper that takes one or more targets and an optional trailing
note. A helper returns an array, so `+` joins several:

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

An array of helper calls means the same as `+`. `edge("<kind>", ...)` writes a
kind that has no helper, such as one the project declares:

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
  [This task cannot start until the target is finished. It is the only built-in kind that makes a task blocked. `mt check` refuses a cycle of these edges.],

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

Each task has two computed fields over the `parent` edges that name it.
`children` counts those tasks. `progress` is the share of them in a
#link(u("/statuses/#the-default-legend"))[terminal] status, so a cancelled
child counts as finished. In the table, the `progress`
column shows the counts:

#terminal("links-progress")

A filter, `--sort`, `--json` and `--format csv` read `progress` as a fraction
from 0 to 1, so `mt ls 'progress == 1 && !terminal'` lists the parents ready
to close. A task without children has no `progress`. It prints as `null` in
JSON and an empty cell in the table, and a comparison such as `progress < 1`
never matches it.

`mt flip` refuses to move a parent with unfinished children into a status
that is both terminal and `unblocked`, as
#link(u("/statuses/"))[The status legend] explains.

= Writing edges with `mt link`

`mt link <task> <kind> <target>` adds an edge and writes the path for you. A
target can be a #link(u("/concepts/#identity"))[stem], a stem with `.typ`, or a
path. A bare stem names a task
in any folder, and `mt link` writes the path from the linking file to it. An
edge already on the task is not written again:

#terminal("links-write")

`mt link --apply <file>` merges a whole map of task to kind to targets, from
YAML or JSON. The #link(u("/cli/link/"))[`mt link` page] lists its other flags.

= Project-declared kinds

`[[tasks.links.kinds]]` adds kinds beside the six built-ins. A task writes
one with `edge("<kind>", ...)`, as the `discharged-by` edge above does.
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

A kind whose `validate` is not `local-task-path` points outside the backlog,
at a URL, a commit or a ticket. `mt check` reports a target that does not fit
the validator, naming the kind, the validator and the value. `mt` never
resolves such a target to a task. It leaves it out of `max`, of cycle checks
and of readiness, even when the kind sets `blocking`. `mt graph` draws each
such target as a leaf node.

= The graph

`mt graph` prints the tasks and their edges as a Graphviz DOT graph, or as a
Mermaid flowchart with `--format mermaid`. A filter limits it to the tasks
that match, and `--no-terminal` leaves out the finished ones. Each node shows
a task's title and status. A ready task is green, a blocked one red and a
finished one grey. This is the graph of `examples/links/`, the project whose
edges this page quotes:

#diagram("links-graph", [`mt graph` on `examples/links/`, drawn by Graphviz])
