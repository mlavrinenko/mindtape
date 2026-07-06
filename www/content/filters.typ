#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, callout
#import "../utils/site.typ": u

#show: page.with(
  title: "Filter language",
  active: "/filters/",
  summary: "The expression language mt ls and mt rm use to select tasks: fields, operators, functions, and dependency predicates.",
)

`mt ls` and `mt rm` take a filter expression — a boolean query over task fields.

#terminal("$ mt ls 'status != \"done\" && priority >= 7'
#   STATUS  PRIORITY  DUE         TITLE
1e  wip     high      2026-07-20  Wire OAuth login")

= Fields

Comparable fields: `status`, `priority`, `difficulty`, `kind`, `title`, `id`,
`due`, `start`, `reviewed`, `file`. `priority` and `difficulty` compare
numerically (`priority >= 7`). The opaque `estimate` dict supports presence only
(`has(estimate)`), never comparison.

= Operators

`==` `!=` `<` `>` `<=` `>=` for comparison; `&&` `||` `!` for logic; parentheses
to group.

```sh
mt ls 'kind == "bug" && (priority >= 7 || has_tag("urgent"))'
```

= Functions

#table(
  columns: 2,
  table.header([Function], [Matches]),
  [`has(field)` / `miss(field)`], [The field is present / absent.],
  [`has_tag("name")`], [The tag is set — authored or #link(u("/derived-tags/"))[derived].],
  [`contains(field, "substr")`], [Substring match on a field.],
  [`search("query")`], [Full-text search across the task.],
  [`stale(N)`], [Not re-affirmed (or, lacking `reviewed`, last status-changed) for more than N days.],
)

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

```sh
mt ls --sort priority:asc      # or :desc
mt ls -n 10                    # limit to 10
```

= Default filters

`mt ls` applies `[ls] default-filter` (shipped as `status != "done"`) unless you
pass `--all`. `mt rm` applies `[rm] default-filter` (`terminal`). Both live in
#link(u("/config/"))[Configuration].

#callout("note")[
  Filters are parsed, not evaluated as code — the expression syntax is fixed.
  There is no arbitrary function call surface to worry about.
]
