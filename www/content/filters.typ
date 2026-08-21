#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal, terminal-ansi
#import "../utils/site.typ": u

#let meta = (
  route: "/filters/",
  label: "Filter language",
  title: "Filter language",
  summary: "The expression language mt ls and mt rm use to select tasks: fields, operators, functions, and dependency predicates.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt ls` and `mt rm` take a filter expression — a boolean query over task fields.
Here, every task not yet in a terminal status:

#terminal-ansi(read("../generated/ls-open.ansi"))

= Fields

Comparable fields: `status`, `priority`, `difficulty`, `kind`, `title`, `id`,
`due`, `start`, `reviewed`, `file`, `depth`, `topo`, `blocks`, `assignee`,
`role`. `priority` and `difficulty` compare numerically (`priority >= 7`). The
opaque `estimate` dict supports presence only (`has(estimate)`), never
comparison.

`assignee` compares the actor name as a plain string (`assignee == "ada"`).
`role` is resolved from the `[[actors]]` registry at read time: `role ==
"reviewer"` matches a task whose assignee holds that role, and `role !=
"reviewer"` is its exact complement — an unassigned task, or one whose
assignee is not a declared actor, holds no role.

`file` compares the task's project-root-relative source path with `/`
separators on every platform — deliberately not the
#link(u("/cli/"))[`file` column]'s working-directory-relative display: a
filter saved in `.mindtape/config.toml` (a view, an `[[ls.rules]]` entry) must
mean the same thing regardless of where `mt` runs from, so it stays anchored
to the root rather than to whichever directory issued the command.

`depth` is the task's nesting under its root — 0 for a task in the root itself,
1 for one in a sub-folder, and so on. It is always numeric, so `depth == 0`
lists the root tasks with no sub-folder (`depth > 0` the nested ones).

`topo` and `blocks` read the dependency graph rather than the task's own
fields — see _Graph position and leverage_ below. Both are always numeric and
never absent: a task with no links carries 0, not nothing.

= Operators

`==` `!=` `<` `>` `<=` `>=` for comparison; `&&` `||` `!` for logic; parentheses
to group.

```sh
mt ls 'kind == "bug" && (priority >= 7 || has_tag("urgent"))'
```

A field the task does not carry is unequal to every literal and orders against
none: `kind != "bug"` matches a task with no `kind` at all, while `kind == "bug"`,
`kind < "bug"` and the rest stay false. A `field == "x"` / `field != "x"` pair
therefore partitions the whole backlog with no third case — the property
#link(u("/config/"))[work-in-progress limits] rest on. Select absence explicitly
with `miss(field)`.

= Functions

#table(
  columns: 2,
  table.header([Function], [Matches]),
  [`has(field)` / `miss(field)`], [The field is present / absent.],
  [`has_tag("name")`],
  [The tag is set — authored or #link(u("/derived-tags/"))[derived].],

  [`contains("substr")` / `contains("substr", field, ...)`],
  [Case-insensitive substring — needle first. No field named searches
    `title`, `id`, and the body prose (or `[ls] contains-fields`, when a
    project has configured one — see #link(u("/config/"))[Configuration]);
    one or more explicit field names always searches exactly those,
    regardless of the configured default. `note` (every `links[].note`,
    concatenated) and `tags` (authored and derived) are searchable this way
    too, though neither is in the shipped default.],

  [`stale(N)`],
  [Not re-affirmed (or, lacking `reviewed`, last status-changed) for more than N days.],

  [`links("kind")`],
  [The task carries a #link(u("/dsl/"))[`links:`] edge of that kind — built-in or a project's custom kind.],

  [`in_view("name")`],
  [The named #link(u("/config/"))[view]'s filter matches. `@name` is sugar for it.],
)

Everything date-shaped — `stale(N)`, `now()` in an `[[ls.rules]]` condition,
`due < now()` and the overdue marker — reads one "today", resolved once per
process and pinnable with `$MINDTAPE_NOW`: see
#link(u("/config/"))[Environment variables].

`@name` composes like any predicate:

```sh
mt ls '@backend && !@blocked'   # in the backend view, minus blocked
```

One predicate exists only where a transition does: `into("status")`, in a
#link(u("/config/"))[`[[on.flip]]`] hook's `when`, matches the status being
stamped, while the rest of the expression reads the task as it stands before the
write. Everything else on this page composes with it —
`into("done") && !has_tag("chore")` is an ordinary filter with one extra term.

= Dependency predicates

`ready` and `blocked` classify a task by its `depends-on` targets: ready when
every direct dependency is terminal, blocked otherwise. `terminal` matches a task
in a terminal status — it is the built-in `[rm] default-filter`. Use them bare or
called (`ready()`):

```sh
mt ls ready          # startable now — deps all done
mt ls blocked        # waiting on something
mt rm terminal       # retire finished work
```

`mt ls` floats ready work to the top by default.

= Graph position and leverage

`ready` and `blocked` answer one hop, and only yes or no — they separate
startable work from waiting work without ordering either group. `topo` and
`blocks` read the same edges transitively and answer with a number, so they
can sort.

`topo` is *position*: 0 for a task nothing in the set blocks, otherwise one
more than the deepest task it depends on. A dependency always sits on a lower
level than its dependents, so ascending `topo` is a valid working order. It is
a level, not a queue number — unrelated tasks tie, which is what leaves room
for a second `--sort` spec to decide between them.

`blocks` is *leverage*: how many tasks transitively wait on this one. This is
the number that reorders a backlog, because a scoring framework ranks a task on
its own properties alone — so the decision five other tasks depend on scores
low for being expensive and sorts last. `blocks` says how much that decision
unlocks.

```sh
mt ls 'blocks > 3'             # load-bearing work
mt ls 'topo == 0 && !terminal' # nothing in the way of these
```

Both read the same edges `ready` does: `depends-on`, plus any custom kind a
project flags #link(u("/config/"))[`blocking = true`]. An edge naming no loaded
task constrains nothing.

They differ on status, deliberately. `topo` ignores it: a finished dependency
still sits before its dependent in the graph. `blocks` counts only unfinished
dependents, and walks only through them — a done task is waiting on nothing and
passes nothing on, so it blocks 0. The count therefore *shrinks as work
closes*, which is the point: it measures what is still waiting, not what once
was.

`mt check` rejects a dependency cycle, but sorting never has to: a task on a
loop simply keeps the level its resolved dependencies gave it, and neither rank
can hang.

= Sort and limit

Sortable fields: every comparable field above except `role` (virtual,
comparison-only), plus `ready` (a computed rank — ready before blocked before
settled). `--sort` takes one or more `field` / `field:asc` / `field:desc`
specs, most-significant first; `[ls] default-sort` in
#link(u("/config/"))[Configuration] takes the same specs for the built-in
ordering.

```sh
mt ls --sort priority:asc      # or :desc
mt ls --sort status --sort due:desc   # status first, due breaks ties
mt ls -n 10                    # limit to 10
```

The graph ranks are what make a sort chain read as an order of work, since both
tie and hand the tie to the next spec:

```sh
mt ls --sort topo --sort priority:desc    # graph order, priority within a layer
mt ls --sort blocks:desc --sort priority:desc  # leverage first, score to break ties
```

= Default filters

Omitting the filter first tries the project's ordered
#link(u("/cli/"))[`[[ls.rules]]`] — auto-activation rules keyed off a separate,
environment-only mini-language: same operators and parser as this page's
filters, no task fields.
No rule matching falls back to the default view — `[ls] default-view` by name,
else the `[views.default]` view; an empty filter (`''`) skips all of that and
matches everything. The shipped default view filters `!terminal &&
(miss(start) || start <= now())` — open work whose start date, if any, has
arrived. Being a view (not a bare filter) it can carry sort and columns too.
`mt rm` applies `[rm] default-filter` (`terminal`). All three live in
#link(u("/config/"))[Configuration].

The same language also decides which tasks must carry a field: `[tasks.priority]
required = "!terminal"` (and its siblings) is a filter, evaluated per task by
`mt check` — see #link(u("/config/"))[Required fields].

#callout("note")[
  Filters are parsed, not evaluated as code: the expression syntax is fixed,
  with no arbitrary call surface.
]
