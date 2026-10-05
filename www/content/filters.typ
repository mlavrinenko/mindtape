#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, commands, exit-code, terminal, terminal-ansi
#import "../utils/site.typ": u

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

A filter is a boolean expression over task fields. `mt ls` and `mt rm` take
one as an argument, and views, `[[tasks.rules]]` and `[[on.flip]]` hooks take
one in config. The examples on this page ran in `examples/api-service`, a
backend team's backlog:

#terminal-ansi("filters-open")

= Fields

#table(
  columns: 2,
  table.header([Field], [Value]),
  [`status`], [The status name. It sorts by position in the status legend.],
  [`status.name`, `status.date`, `status.note`],
  [The name, the ISO date and the note of the status stamp. `status.name` sorts alphabetically.],

  [`status.index`],
  [The status's 0-based position in the legend, as a number. A status outside the legend has none.],

  [`priority`, `difficulty`],
  [Numbers. A #link(u("/priority/"))[framework] priority reads as its score.],

  [`kind`, `title`, `id`, `assignee`], [Text.],
  [`due`, `start`, `reviewed`], [ISO dates, compared as text: `due < now()`.],
  [`path`],
  [The task file's path from the project root, with `/` on every platform.],

  [`depth`],
  [How many folders deep the file sits under its root: 0 in the root itself.],

  [`role`],
  [Matches when the assignee holds that role in `[[actors]]`. Comparison only, so it does not sort.],

  [`topo`, `blocks`],
  [The task's place in the dependency graph, described below.],

  [`children`, `progress`],
  [The number of #link(u("/links/"))[`parent`] children, and the finished share of them from 0 to 1. A task with no children has no `progress`.],

  [`estimate.<unit>`],
  [One #link(u("/estimates/"))[estimate] unit, as a number. Filters only, so it does not sort.],

  [`estimate`],
  [The estimate dict. Only `has()` and `miss()` read it: `has(estimate, "days")`.],

  [`tags`],
  [The written and derived tags. `has(tags, "frontend")` reads them, and `contains` searches them.],

  [`x-<ns>`],
  [An extension field, whether or not `[tasks.fields]` declares it. Text, a number or a boolean compares like any field. A list equals nothing, so `has(field, value)` reads it. A dict takes only `has()` and `miss()`. A filter in the config that names an undeclared one, while `[tasks.fields]` declares others, prints a warning at load with the nearest declared name.],
)

A field name may contain `-` between two letters or digits, as in
`estimate.src-files` and `x-goals`. Any other `-` is a minus sign.
`[tasks.estimate] units`, when set, lists the only units a filter may name.

`mt ls` shows a `file` column, but `file` is not a field: a filter that names
it fails. Use `path`.

= Operators

`==`, `!=`, `<`, `>`, `<=` and `>=` compare. `&&`, `||` and `!` combine, and
parentheses group:

#terminal("filters-operators")

The left side of a comparison is a field or a count. The right side is a
literal, `now()`, another field or a count. Two numbers compare as numbers.

A field the task does not have is unequal to every value and orders against
none. So `kind != "bug"` matches a task with no `kind`, and `kind == "bug"`
and `kind < "bug"` do not. A `field == "x"` and `field != "x"` pair splits the
backlog in two with nothing left over. `miss(field)` selects the tasks
without the field.

`now()` is today's date. `stale()`, `now()` and the overdue marker read the
same date, which `MINDTAPE_NOW` pins (see
#link(u("/config/"))[Environment variables]).

= Functions

#table(
  columns: 2,
  table.header([Function], [Matches]),
  [`has(field)`, `miss(field)`], [The task has the field, or does not.],
  [`has(field, value)`],
  [The field's list holds the value, its scalar equals it, or its dict has it as a key. `tags` and `estimate` read this way too.],

  [`has_tag("name")`],
  [The task has the tag, written or #link(u("/derived-tags/"))[derived].],

  [`contains("text")`],
  [The text appears in the title, the id or the body, ignoring case. `[ls] contains-fields` changes those fields.],

  [`contains("text", field, ...)`],
  [The text appears in one of the named fields. `note` (every link note) and `tags` are searchable too.],

  [`stale(N)`],
  [The task was not reviewed in the last N days. With no `reviewed` date, its status date counts.],

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

`into("status")` matches when a change moves the task into that status. It works
only in an #link(u("/hooks/"))[`[[on.flip]]`] `when` and in a task-scope
#link(u("/commit-rules/"))[`[[commits.rules]]`] entry, because only there
does a status change exist. The other functions a task-scope commit rule
reads are on that page.

= Text functions

Three functions read text. `word(field, n)` is the n-th word of a text field,
counting from 1, with punctuation trimmed from both ends, so
`"Fix: the slug"` has `Fix` as its first word. It returns text, so it only appears as an
argument. `matches(field, "regex")` matches a Rust `regex` pattern anywhere in
the text. `in_dict("name", text)` matches when the text is in the named
#link(u("/named-sets/"))[dictionary]. Each of `matches()` and `in_dict()`
takes a field or `word(...)` as its text:

#commands("filters-strings")

`api-service` extends the #link(u("/rule-packs/"))[`mindtape-tasks`] pack,
which declares the `verbs` dictionary. A project without the pack uses the
built-in `mindtape-verbs`, or declares its own `[dicts.verbs]`.

`word()` splits on whitespace and knows nothing about Typst markup. A title
that opens with a code span, such as #raw("`mt ls` prints paths"), has `mt`
as its first word, so the last example above lets `matches()` accept it.

A value that is not there, such as a missing field or a word past the end of
the title, does not match. Any other problem is an error, and the message
names it: an unknown dictionary, a pattern `regex` refuses, a number where text is
needed, or a wrong argument count. `mt ls` exits #exit-code("filters-unknown-dict"):

#terminal("filters-unknown-dict")

`mt` also runs each filter saved in config once when it loads the config, so
a view or rule with a bad expression fails by name before any listing.

= Dependencies and counts

`ready` matches an open task whose dependencies are all in a terminal status.
`blocked` matches an open task with at least one dependency that is not,
including one no loaded task answers to. A task in a terminal status is
neither, and `terminal` matches it. `mt ls` lists ready tasks first.

The `why` column lists what each blocked task waits on:

#terminal("filters-blocked")

`count(expr)` is how many loaded tasks match `expr`. It is a number, so it
appears in a comparison. It counts the whole loaded set, whatever the rest
of the filter selects, so a rule such as a
#link(u("/rules/"))[work-in-progress cap] can compare it on every task.
`count()` works where `mt` loads the whole set: `mt ls`, `mt rm`, views,
`[[tasks.rules]]` and `[[on.flip]]`. A `required` filter cannot use it, and
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
#link(u("/links/"))[custom kind] the project marks `blocking = true`. `ready`,
`blocked`, `topo` and `blocks` all read them. An edge to a task that is not
loaded counts as unmet for `blocked` and is ignored by `topo` and `blocks`. `mt check`
refuses a dependency cycle, and neither rank loops on one.

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

`touches("path")` matches the tasks named by the
#link(u("/commit-rules/"))[`[commits] trailer`] of any commit that changed the
path. Ask it before you edit a file, to find the tasks that shaped it. The
path is relative to the repository root:

#terminal("filters-touches")

A trailer may name a task by stem (`Refs: parser`) or by path
(`Refs: tasks/parser.typ`). A task `mt mv` renamed still matches its old stem,
through the renames git records. A stem no loaded task answers to belongs to
a retired task. It matches nothing, and `mt ls --json` lists it under
`retired`.

The first `touches()` in a command reads the history with one `git log`.
`mt` caches the result in `.mindtape/cache/` until `HEAD` or the trailer key
changes, and a command that does not call `touches()` runs no git.

`changed()` and `touches()` need a git repository. Outside one, a filter
typed on the command line fails and prints git's reason, and a rule that
reads git is reported once as skipped. Loading a config never runs git. Git runs read-only (`--no-optional-locks`) and at
most once per command.

= Sort and limit

`--sort` takes `field`, `field:asc` or `field:desc`, and repeats, most
significant first. Every field in the table sorts except `role`,
`estimate.<unit>` and `estimate`. An `x-<ns>` list or dict sorts as a
missing value, last. `ready` sorts too: ready, then
blocked, then terminal. Any other direction, such as `priority:up`, is an error.
`[ls] default-sort` takes the same specs.

`topo` and `blocks` produce ties, so a second `--sort` decides within each
level. `-n` cuts the list after the filter and the sort, so it keeps the top
of the order you asked for:

#commands("filters-sort")

= Without a filter

A bare `mt ls` tries the project's #link(u("/cli/"))[`[[ls.rules]]`] first.
If none matches, it applies the default view: the view `[ls] default-view`
names, or `[views.default]`. The shipped default view is
`!terminal && (miss(start) || start <= now())`, the open tasks whose start
date has passed or that have none. An empty filter, `mt ls ''`, matches every
task. A bare `mt rm` applies `[rm] default-filter`, which is `terminal`.

The same language decides which tasks must have a field; see
#link(u("/rules/"))[Required fields].

#callout("note")[
  `mt` parses a filter and walks the parse tree itself. Nothing in a filter
  runs as code.
]
