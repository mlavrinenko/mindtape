#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal, terminal-ansi
#import "../utils/site.typ": u

#let meta = (
  route: "/filters/",
  label: "Filter language",
  title: "Filter language",
  summary: "The expression language `mt ls` and `mt rm` use to select tasks, with its fields, operators and functions.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt ls` and `mt rm` take a filter expression — a boolean query over task fields.
Here, `mt ls '!terminal'` lists every task not yet in a terminal status:

#terminal-ansi(read("../generated/ls-open.ansi"))

= Fields

Comparable fields: `status`, `priority`, `difficulty`, `kind`, `title`, `id`,
`due`, `start`, `reviewed`, `path`, `depth`, `topo`, `blocks`, `children`,
`progress`, `assignee`, `role`. `priority` and `difficulty` compare numerically (`priority >= 7`).

The `estimate` dict takes presence only (`has(estimate)`); one unit of it,
`estimate.<unit>`, compares as a number under any unit name
(`estimate.src-files <= 5`). A unit the task leaves out is absent like any
other field, so `has(estimate.src-files)` tests it. When `[tasks.estimate]
units` is declared, a unit outside it is an unknown field — in a saved
expression, a config error at load. An `x-<ns>` extension slot takes presence
only: `has(x-goals)`.

Names are spelled the way Typst spells them: a `-` between two name characters
belongs to the name, so `x-goals` and `estimate.src-files` are single names. A
`-` anywhere else is a minus — `priority > -1` still compares against a
negative number — and a string literal is never touched.

A field with fixed named parts is addressed as `field.property`. `status`
carries four: `status.name` (the same value bare `status` compares),
`status.date` (the ISO stamp), `status.note` (the note written with the
stamp), and `status.index` (the 0-based position in the configured legend,
numeric — `status.index < 2` is "before the third status"; a status outside
the legend has no index, so `has(status.index)` is false). A property takes
the comparison, `has`/`miss`, and `--sort` its field does, and a
string-valued one takes `contains` and `matches` too — `status.index` is
numeric, so it does not.

`assignee` compares the actor name as a plain string (`assignee == "ada"`).
`role` is resolved from the `[[actors]]` registry at read time: `role ==
"reviewer"` matches a task whose assignee holds that role, and `role !=
"reviewer"` is its exact complement — an unassigned task, or one whose
assignee is not a declared actor, holds no role.

`path` compares the task's project-root-relative source path with `/`
separators on every platform. A filter saved in `.mindtape/config.toml` (a
view, an `[[ls.rules]]` entry) must mean the same thing regardless of where
`mt` runs from, so it stays anchored to the root rather than to whichever
directory issued the command.

`file` is not a filter field at all — filtering on it is an error, not an
empty listing. #link(u("/cli/"))[CLI] owns what the `file` column displays;
#link(u("/schema/"))[Schema] owns the value the record formats carry.

`depth` is the task's nesting under its root — 0 for a task in the root itself,
1 for one in a sub-folder, and so on. It is always numeric, so `depth == 0`
lists the root tasks with no sub-folder (`depth > 0` the nested ones).

`topo` and `blocks` read the dependency graph rather than the task's own
fields — see _Graph position and leverage_ below. Both are always numeric and
never absent: a task with no links carries 0, not nothing.

`children` and `progress` count a task's `parent`-edge children and the
finished share of them, `0..1`; a task with no children has no `progress` —
see #link(u("/links/"))[Parents and progress].

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
#link(u("/rules/"))[work-in-progress caps] rest on. Select absence explicitly
with `miss(field)`.

The right side may also be another field, or a `changed()` count (below):
`estimate.files >= estimate.days` compares two fields, numerically when both
are numbers, and a side the task lacks follows the same rule.

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
  [The named #link(u("/named-sets/"))[view]'s filter matches. `@name` is sugar for it.],

  [`matches(field, "regex")`],
  [The field's text matches the (unanchored, Rust `regex`) pattern.],

  [`in_dict("name", <string>)`],
  [The string is in the named #link(u("/named-sets/"))[`[dicts.<name>]`] set,
    under that dictionary's own `case`. Paired with `word()` below, this is
    what #link(u("/rules/"))[a config-declared rule] turns into an
    enforced convention rather than a query.],
)

= Function arguments

An argument may be a literal, a bare field, or another function's result:
`word()` nests inside `matches()` and `in_dict()`, the two functions that read
a string.

```sh
mt ls 'in_dict("verbs", word(title, 1))'   # titles opening with a known verb
mt ls '!matches(word(title, 1), "ing$")'   # ...and not a gerund
```

`word(field, n)` is the one function that yields a _string_ rather than a
verdict: the n-th whitespace-separated word of a string field, counting from
1, trimmed of everything non-alphanumeric at either end (`"Fix: the slug"` has
`Fix` as its first word). It is an argument, never a term of its own.

`word()` takes no position on Typst markup. A whitespace split is all it does,
so a title opening with a backticked command yields the token inside the ticks
— #raw("`mt ls` should print the path") has `mt` as its first word, which no
verb dictionary carries. Exempting such a title is the rule author's business,
which is what `matches()` is for:

```sh
mt ls 'in_dict("verbs", word(title, 1)) || matches(title, "^`")'
```

Nothing goes missing quietly. A value that is not there — a field the task
does not carry, a word past the end of the title — reads as a non-match, the
same missing-field rule comparisons follow. Everything else is an error naming
itself: a dictionary no `[dicts]` section and no built-in declares, a pattern
`regex` refuses, a numeric field where a string is required, a wrong argument
count. A filter saved in config is dry-run at load, so a view or a rule
reports its own bad expression by name rather than quietly matching nothing.

Built-in dictionaries need no declaration, so a filter references them
directly — #link(u("/named-sets/"))[Views & dictionaries] owns them:

```sh
mt ls 'in_dict("mindtape-verbs", word(title, 1))'
```

Everything date-shaped — `stale(N)`, `now()` in an `[[ls.rules]]` condition,
`due < now()` and the overdue marker — reads one "today", resolved once per
process and pinnable with `$MINDTAPE_NOW`: see
#link(u("/config/"))[Environment variables].

`@name` composes like any predicate:

```sh
mt ls '@backend && !@blocked'   # in the backend view, minus blocked
```

One predicate exists only where a transition does: `into("status")`, in a
#link(u("/hooks/"))[`[[on.flip]]`] hook's `when`.

= Changed files

`changed("<states>", "<glob>"…)` counts the distinct paths git reports
changed. It is a count, not a verdict, so it sits in a comparison — against a
literal or a field:

```toml
[[tasks.rules]]
name     = "src-budget"
when     = 'status == "wip" && has(estimate.src-files)'
assert   = 'changed("staged+worktree", "crates/*/src/**", "!**/*_tests*.rs") <= estimate.src-files'
severity = "warn"
```

#table(
  columns: 2,
  table.header([State], [Counts a path that]),
  [`staged`], [differs in the index from `HEAD`.],
  [`worktree`], [differs in the working tree from the index, or is untracked.],
  [`staged+worktree`], [is in either; a path in both counts once.],
)

Globs match paths relative to the repository root; `**` crosses directories.
A glob names what counts, one starting with `!` excludes even what another
admits, and no glob counts every path. A rename counts its destination.

Git runs read-only (`--no-optional-locks`, so not even the index's stat cache
is refreshed), on demand, and at most once per command. Outside a git
repository a `[[tasks.rules]]` entry reading `changed()` reports once as
skipped — a warning whatever its `severity` — while a filter typed on the
command line fails naming git's reason. The load-time dry run checks the
state names and globs without running git.

= Tasks that shaped a file

`touches("<path>")` matches a task named by the
#link(u("/config/"))[`[commits] trailer`] of any commit that changed `<path>`,
relative to the repository root — which tasks shaped a file, asked before
editing it:

```sh
mt ls 'touches("crates/mindtape/src/ready.rs")'
```

A trailer value resolves by its stem, whether written as a stem (`Refs:
parser`) or as a path (`Refs: tasks/parser.typ`). A task `mt mv` renamed
answers to its old stem through the renames git records under the task roots.
A stem no loaded task answers to, even through a rename, belongs to a retired
task: it matches nothing, and `mt ls --json` lists it in the envelope's
`retired` key, for git archaeology.

The first `touches()` of a command reads the whole history in one `git log`,
and the index is cached in `.mindtape/cache/` until `HEAD` or the trailer key
moves; a command that never calls it never runs git. Outside a git repository
it fails naming git's reason, and a `[[tasks.rules]]` entry reading it is
skipped, as with `changed()`.

= Dependency predicates

`ready` and `blocked` classify a non-terminal task by its `depends-on`
targets: ready when every direct dependency is terminal; blocked when one is
not — including a link naming no loaded task. A terminal task is settled,
neither. `terminal` matches a task in a terminal status. Use them bare or
called (`ready()`):

```sh
mt ls ready          # startable now — deps all done
mt ls blocked        # waiting on something
mt rm terminal       # retire finished work
```

`mt ls` floats ready work to the top by default.

= Counting the set

`count(<expr>)` is how many loaded tasks match `<expr>` — a number, compared
on the left of an operator. The argument is an expression, not a string, so
nothing nests in quotes:

```sh
mt ls 'count(status == "wip") > 1 && status == "wip"'  # everything in flight, when more than one is
```

It ranges over the whole loaded set, never the rows a filter narrows to — the
same rule `ready` follows. Inside `count` every field reads the task being
counted, so the answer is one number per command, whichever task asks:
`count` walks the set once per distinct argument, and a rule asserting it on
every task stays linear. That makes a set-level cap a rule a project writes
rather than a table of its own: see
#link(u("/rules/"))[Work-in-progress caps].

`count` needs the loaded set, so it works where one exists — `mt ls`, `mt rm`,
views, `[[tasks.rules]]`, `[[on.flip]]` hooks — and is an error anywhere
evaluated one task at a time, rather than a silent zero; `required` rejects it
at load.

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
project flags #link(u("/links/"))[`blocking = true`]. An edge naming no loaded
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
settled). A property is sortable with its field. Bare `status` orders by
legend position — the same order as `status.index`, spelled the same way it
was before the property existed — while `status.name` orders alphabetically.
`--sort` takes one or more `field` / `field:asc` / `field:desc` specs,
most-significant first; `[ls] default-sort` in
#link(u("/config/"))[Configuration] takes the same specs for the built-in
ordering. Any other suffix (`priority:up`, a lone `priority:`) is an error
naming the direction — never a silent ascending sort.

```sh
mt ls --sort priority:asc      # or :desc
mt ls --sort status --sort due:desc   # status first, due breaks ties
mt ls --sort status.date:desc  # most recently stamped first
mt ls -n 10                    # limit to 10
```

The graph ranks are what make a sort chain read as an order of work, since both
tie and hand the tie to the next spec:

```sh
mt ls --sort topo --sort priority:desc    # graph order, priority within a layer
mt ls --sort blocks:desc --sort priority:desc  # leverage first, score to break ties
```

`-n` cuts the list *after* the filter and the sort, so it always yields the top
N of the order you asked for rather than whichever tasks the loader reached
first; #link(u("/schema/"))[Schema] and #link(u("/agents/"))[For AI agents] own
how the cut reports itself.

= Default filters

Omitting the filter first tries the project's
#link(u("/cli/"))[`[[ls.rules]]`].
No rule matching falls back to the default view — `[ls] default-view` by name,
else the `[views.default]` view; an empty filter (`''`) skips all of that and
matches everything. The shipped default view filters `!terminal &&
(miss(start) || start <= now())` — open work whose start date, if any, has
arrived. Being a view (not a bare filter) it can carry sort and columns too.
`mt rm` applies `[rm] default-filter` (`terminal`). All three live in
#link(u("/config/"))[Configuration].

The same language also decides which tasks must carry a field — see
#link(u("/rules/"))[Required fields].

#callout("note")[
  Filters are parsed, not evaluated as code: the expression syntax is fixed,
  with no arbitrary call surface.
]
