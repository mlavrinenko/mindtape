#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal, terminal-ansi
#import "../utils/site.typ": u

#show: page.with(
  title: "Filter language",
  active: "/filters/",
  summary: "The expression language mt ls and mt rm use to select tasks: fields, operators, functions, and dependency predicates.",
)

`mt ls` and `mt rm` take a filter expression — a boolean query over task fields.
Here, every task not yet in a terminal status:

#terminal-ansi(read("../generated/ls-open.ansi"))

= Fields

Comparable fields: `status`, `priority`, `difficulty`, `kind`, `title`, `id`,
`due`, `start`, `reviewed`, `file`, `depth`, `assignee`, `role`. `priority` and
`difficulty` compare numerically (`priority >= 7`). The opaque `estimate` dict
supports presence only (`has(estimate)`), never comparison.

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
    `title`, `id`, and the body prose; one or more field names narrows it.],

  [`stale(N)`],
  [Not re-affirmed (or, lacking `reviewed`, last status-changed) for more than N days.],

  [`links("kind")`],
  [The task carries a #link(u("/dsl/"))[`links:`] edge of that kind — built-in or a project's custom kind.],

  [`in_view("name")`],
  [The named #link(u("/config/"))[view]'s filter matches. `@name` is sugar for it.],
)

Everything date-shaped — `stale(N)`, `now()` in an `[[ls.rules]]` condition,
`due < now()` and the overdue marker — reads "today" as the calendar date on
the machine running `mt`, in its local timezone (`$TZ` on Unix), never UTC. So
a task filed at 01:00 carries that day's date, not the previous one, and
`$MINDTAPE_NOW` overrides the lot — see #link(u("/config/"))[Environment
  variables].

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

= Default filters

Omitting the filter first tries the project's ordered
#link(u("/cli/"))[`[[ls.rules]]`] — auto-activation rules keyed off a separate,
environment-only mini-language (`on_branch(...)`, `env(...)`, `weekday()`,
`now()` — no task fields; same operators and parser as this page's filters).
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
  Filters are parsed, not evaluated as code — the expression syntax is fixed.
  There is no arbitrary function call surface to worry about.
]
