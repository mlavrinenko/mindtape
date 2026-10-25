#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, commands, exit-code, terminal, terminal-ansi
#import "../utils/site.typ": u
#import "../utils/filter-field-notes.typ": filter-field-notes

// The fields and which of them sort come from `FIELDS` and
// `is_sort_field`, read back through filter-fields.json. A notes key that
// isn't a field there is filter only.
#let filter-fields = json("../generated/filter-fields.json").fields
#for field in filter-fields {
  assert(
    field.name in filter-field-notes,
    message: "www/utils/filter-field-notes.typ has no note for the filter field "
      + field.name,
  )
}
#let sorted-fields = filter-fields.filter(f => f.sortable).map(f => f.name)
#let unsorted-fields = (
  filter-field-notes.keys().filter(name => name not in sorted-fields)
)
#let field-rows = {
  let rows = ()
  for (name, note) in filter-field-notes {
    let sorts = name in sorted-fields
    if (
      rows.len() > 0 and rows.last().note == note and rows.last().sorts == sorts
    ) {
      rows.last().names.push(name)
    } else {
      rows.push((names: (name,), note: note, sorts: sorts))
    }
  }
  rows
}

#let meta = (
  route: "/filters/",
  label: "Filter language",
  title: "Filter language",
  summary: "The expression language `mt ls` and `mt rm` use to select tasks, with its fields, operators and functions.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Every block is a capture www/generate-reference.sh makes in
// examples/api-service, or in a git copy of it for touches().

A filter picks tasks. It's a boolean expression over task fields. You pass
one to `mt ls` and `mt rm`, and write one in config for
#link(u("/named-sets/#views"))[views], `[[tasks.rules]]` and `[[on.flip]]`
hooks. The examples on this page ran in `examples/api-service`, a sample
backend project's backlog:

#terminal-ansi("filters-open")

= Fields

#table(
  columns: 2,
  table.header([Field], [Value]),
  ..field-rows
    .map(row => (
      row.names.map(raw).join([, ]),
      if row.sorts { row.note } else [#row.note It doesn't sort.],
    ))
    .flatten(),

  [`x-<ns>`],
  [An extension field, whether or not `[tasks.fields]` declares it. Text, a number or a boolean compares like any field. A list equals nothing, so use `has(field, value)` to read it. A dict takes only `has()` and `miss()`. If a filter in the config names an undeclared one while `[tasks.fields]` declares others, `mt` warns at load and suggests the nearest declared name.],
)

A field name may contain `-` between two letters or digits, as in
`estimate.src-files` and `x-goals`. Any other `-` is a minus sign. If you set
`[tasks.estimate] units`, a filter may name only those units.

`mt ls` shows a `file` column, but `file` isn't a field, and a filter that
names it fails. Use `path`.

= Operators

`==`, `!=`, `<`, `>`, `<=` and `>=` compare. `&&`, `||` and `!` combine, and
parentheses group:

#terminal("filters-operators")

The left side of a comparison is a field or a count. The right side is a
literal, `now()`, another field or a count. Two numbers compare as numbers.

A field the task doesn't have is unequal to every value and orders against
none. `kind != "bug"` matches a task with no `kind`, while `kind == "bug"` and
`kind < "bug"` don't, so `==` and `!=` split the backlog in two with nothing
left over. `miss(field)` selects the tasks without the field.

`!` flips the answer after the comparison, so `!(field <= 2)` isn't
`field > 2`. The finished tasks here have no estimate, and only the second
filter lets them through:

#terminal("filters-missing")

`now()` is today's date. `stale()`, `now()` and the overdue marker read the
same date, which `MINDTAPE_NOW` pins (see
#link(u("/config/"))[Environment variables]).

= Functions

#table(
  columns: 2,
  table.header([Function], [Matches]),
  [`has(field)`, `miss(field)`], [The task has the field, or doesn't.],
  [`has(field, value)`],
  [The field's list holds the value, its scalar equals it, or its dict has it as a key. `tags` and `estimate` read this way too.],

  [`exists`],
  [Inside `head(...)`, `staged(...)` or `worktree(...)` in a #link(u("/commit-rules/#task-scope"))[task-scope commit rule]: the commit state holds the task's file. Any other filter refuses it.],

  [`has_tag("name")`],
  [The task has the tag, written or #link(u("/derived-tags/"))[derived].],

  [`contains("text")`],
  [The text appears in the title, the id or the body, ignoring case. `[ls] contains-fields` changes those fields.],

  [`contains("text", field, ...)`],
  [The text appears in one of the named fields. `note` (every link note) and `tags` are searchable too.],

  [`stale(N)`],
  [The task wasn't reviewed in the last N days. With no `reviewed` date, its status date counts.],

  [`links("kind")`],
  [The task has a #link(u("/links/"))[`links:`] edge of that kind.],

  [`in_view("name")`, `@name`],
  [The named #link(u("/named-sets/"))[view]'s filter matches.],

  [`ready`, `blocked`, `terminal`],
  [Described below. Write them bare or as `ready()`.],
)

#commands("filters-functions")

One field may hold `"alpha"` on one task and `("alpha", "gamma")` on
another. `x-machine == "alpha"` matches only the first, and
`has(x-machine, "alpha")` matches both.

`into("status")` matches when a change moves the task into that status. It
works only in an #link(u("/hooks/"))[`[[on.flip]]`] `when` and in a task-scope
#link(u("/commit-rules/"))[`[[commits.rules]]`] entry, because those are the
only places a status change exists. That page lists the other functions a
task-scope commit rule reads.

= Text functions

Three functions read text. `word(field, n)` is the n-th word of a text field,
counting from 1, with punctuation trimmed from both ends, so `"Fix: the slug"`
has `Fix` as its first word. It returns text, so it only appears as an
argument. `matches(field, "regex")` matches a Rust `regex` pattern anywhere in
the text. `in_dict("name", text)` matches when the text is in the named
#link(u("/named-sets/"))[dictionary]. Both `matches()` and `in_dict()` take a
field or `word(...)` as their text:

#commands("filters-strings")

`api-service` extends the #link(u("/packs/"))[`mindtape-tasks`] pack,
which declares the `verbs` dictionary. Without the pack, use the built-in
`mindtape-verbs`, or declare your own `[dicts.verbs]`.

`word()` splits on whitespace and knows nothing about Typst markup. A title
that opens with a code span, such as #raw("`mt ls` prints paths"), has `mt`
as its first word, so the last example above lets `matches()` accept it.

`matches()` and `in_dict()` don't match a value that isn't there, such as a
missing field or a word past the end of the title. That's how `==` reads one,
so `!matches(...)` does match it. Anything else wrong is an error that names
the problem, such as an unknown dictionary, a pattern `regex` refuses, a
number where text is needed, or a wrong argument count. `mt ls` exits
#exit-code("filters-unknown-dict"):

#terminal("filters-unknown-dict")

`mt` also runs each filter saved in config once when it loads the config, so
a view or rule with a bad expression fails by name before any listing. A
status that no `[[tasks.status]]` entry declares is a bad expression too:
`status == "wpi"` fails and suggests `wip`, rather than matching nothing.

= Dependencies and counts

`ready` matches an open task whose dependencies are all in a
#link(u("/statuses/#the-default-legend"))[terminal] status, and whose
#link(u("/wait-on-upstream/"))[`waits-on` checkers] all say the wait is over.
`blocked` matches an open task with at least one dependency that isn't,
including one no loaded task answers to, or with a checker that says it still
waits or can't tell. A task in a terminal status is neither, and `terminal`
matches it. `mt ls` lists ready tasks first.

The `why` column lists what each blocked task waits on:

#terminal("filters-blocked")

`count(expr)` is how many loaded tasks match `expr`. It's a number, so it
appears in a comparison. It counts the whole loaded set, whatever the rest of
the filter selects, so a rule such as a
#link(u("/rules/"))[work-in-progress cap] can compare it on every task.
`count()` works where `mt` loads the whole set: `mt ls`, `mt rm`, views,
`[[tasks.rules]]` and `[[on.flip]]`. A `required` filter can't use it, and
`mt` refuses one that does when it loads the config.

`topo` is a task's level in the dependency graph. A task with no dependency
in the loaded set is at level 0, and any other task sits one level above its
deepest dependency. `topo` ignores status, so ascending `topo` is an order in
which every task comes after what it depends on. Unrelated tasks share a
level.

`blocks` counts the open tasks that wait on this one, directly or through
other tasks. A priority framework scores each task on its own, so a cheap
decision that five tasks wait on can score low. `blocks` shows what finishing
it unblocks. A finished task blocks nothing, so `blocks` falls as work
closes.

A dependency is a `depends-on` edge, or an edge of a
#link(u("/links/"))[custom kind] marked `blocking = true`. `ready`, `blocked`,
`topo` and `blocks` all read them. An edge to a task that isn't loaded counts
as unmet for `blocked`, and `topo` and `blocks` ignore it. `mt check` refuses
a dependency cycle, and neither rank loops on one.

#commands("filters-graph")

= Changed files

`changed("states", "glob", ...)` counts the distinct paths git reports
changed, so it appears in a comparison, such as
`changed("staged+worktree", "src/**") <= estimate.src-files`.
#link(u("/estimates/"))[Estimates] shows it in a rule. A rename counts once, at
its new path. A glob written `"@name"` stands for the entries of the
`[dicts.name]` set, which must declare `case = "exact"`, so a rule and a
commit rule can share one path list.

#table(
  columns: 2,
  table.header([State], [Counts a path that]),
  [`staged`], [differs in the index from `HEAD`.],
  [`worktree`], [differs in the working tree from the index, or is untracked.],
  [`staged+worktree`], [is in either. A path in both counts once.],
)

Globs match paths from the repository root, and `**` crosses folders. With
no glob, every path counts. A glob starting with `!` excludes paths another
glob admits. A renamed file counts at its new path.

= Commits that touched a file

Before you edit a file, you may want to know which tasks shaped it.
`touches("path")` matches the tasks named by the
#link(u("/commit-rules/"))[`[commits] trailer`] of any commit that changed the
path. The path is relative to the repository root:

#terminal("filters-touches")

A trailer may name a task by #link(u("/concepts/#identity"))[stem]
(`Refs: parser`) or by path (`Refs: tasks/parser.typ`). A task `mt mv` renamed
still matches its old stem, through the renames git records. A stem no loaded
task answers to belongs to a retired task. It matches nothing, and
`mt ls --json` lists it under `retired`.

The first `touches()` in a command reads the history with one `git log`. `mt`
caches the result in `.mindtape/cache/` until `HEAD` or the trailer key
changes. A command that doesn't call `touches()` runs no git.

`changed()` and `touches()` need a git repository. Outside one, a filter typed
on the command line fails and prints git's reason, and a rule that reads git
is reported once as skipped. Loading a config never runs git. Git runs
read-only (`--no-optional-locks`) and at most once per command.

= Sort and limit

`--sort` takes `field`, `field:asc` or `field:desc`. Repeat it to add keys,
most significant first. Every field in the table sorts except
#unsorted-fields.map(raw).join([, ], last: [ and ]). An `x-<ns>` list or dict sorts as a missing
value, last. `ready` sorts too: ready, then blocked, then terminal. Any other
direction, such as `priority:up`, is an error. `[ls] default-sort` takes the
same specs.

`topo` and `blocks` produce ties, so a second `--sort` decides within each
level. `-n` cuts the list after the filter and the sort, so it keeps the top
of the order you asked for:

#commands("filters-sort")

= Without a filter

A bare `mt ls` tries the project's
#link(u("/cli/ls/#conditional-rules"))[`[[ls.rules]]`] first. If none matches,
it applies the default view: the view `[ls] default-view` names, or
`[views.default]`. The shipped default view is
`!terminal && (miss(start) || start <= now())`, the open tasks whose start
date has passed or that have none. An empty filter, `mt ls ''`, matches every
task. A bare `mt rm` applies `[rm] default-filter`, which is `terminal`.

The same language decides which tasks must have a field. See
#link(u("/rules/"))[Required fields].

#callout("note")[
  `mt` parses a filter and walks the parse tree itself. Nothing in a filter
  runs as code.
]
