#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/config-ref.typ": config-keys-table
#import "../../utils/site.typ": u
#import "../../utils/ui.typ": terminal

#show: cli-page.with("ls")

`mt ls` lists your tasks. Give it a #link(u("/filters/"))[filter], a sort and
a limit, or an `@name` #link(u("/named-sets/"))[view] that sets all three. A
bare `mt ls` uses the first matching `[[ls.rules]]` entry, described below, or
else the default view. `mt ls ''` lists every task.

A listing has three facets: filter, sort and columns. An `[[ls.rules]]` entry,
a view and a command-line argument each set only the facets they name. A
filter argument replaces the filter and keeps the sort and columns of the rule
or view underneath. `--sort` and `--columns` each replace their own facet. A
bare `@view` argument replaces each facet that view sets. `@default` stands
for the default view's facets. Those are empty unless your project changes
the default view, so `--columns @default --sort @default` gets you the
built-in columns and sort:

#terminal("ls-default")

#cli-command("ls")

= Paging

`-n` cuts the list after the sort, and `--skip` drops rows before `-n` cuts.
So `mt ls -n 20 --skip 20` is the second page of 20. A cut table says how many
rows it hid and prints the command for the next page, with your filter and
flags kept and `--skip` advanced. It also keeps how you picked the project, an
`@name` or a `-w` path, so you can paste it from any directory:

#terminal("ls-cut")

`--json` carries the same facts in `truncated`: `shown`, `total` and
`next_skip`, or `null` for a complete listing. #link(u("/schema/"))[Schema]
describes the JSON, and #link(u("/agents/"))[For AI agents] shows how to page
through it.

= Columns

Each column shows when its `[ls.columns.<name>] when` expression holds. The
expression uses the condition language described under Conditional rules
below, and `mt` evaluates it against the rows it's about to print. A column
with no `when` never shows. A #link(u("/config/#layers"))[config layer]
replaces a `when` whole, so `when = 'false'` hides a column.

The shipped expressions: `slug#` and `title` are `true`. `status`, `priority`
and `due` are `nonempty()`. `assignee` is `contested()`, so it shows when the
rows have different assignees. `ready` is `blocked()`. Naming the `assignee`
column doesn't show it. Set `[ls.columns.assignee] when = 'true'` for that.

`why`, `children` and `progress` are `named()`, so they show only when you ask
for them. `why` lists the unfinished blocking targets each task waits on, with
their status. `mt ls blocked --columns title,why` tells you what to unblock
without reading any links, and `--json` carries `why` on every task.
`children` counts a task's `parent` children, and `progress` shows how many of
them are finished, as `3/5`. See #link(u("/links/"))[Parents and progress].

The `slug#` column is the task's stem hash. It's the shortest prefix, at least
three characters long, of a hash of the
#link(u("/concepts/#identity"))[filename stem] that no other task's hash
shares. `mt` hashes every task the project loads, closed ones included, so a
task has the same stem hash in every listing, filtered or not. The hash
changes when `mt mv` gives the task a new stem. It grows a character when a
new task's hash shares its prefix, so `mt` refuses an old hash rather than
pick another task. A hash also grows past any stem or filter word it would
spell, so neither can capture it.

Every command that takes a `<task>` accepts a stem hash, and so does each
`mt link` target. A path or an exact stem wins, and `mt` tries the input as a stem hash only when no stem matches.
`mt rm` also takes filters, so it reads a filter word such as `ready`, a
field, a view or a status as a filter before it tries a stem hash. `mt`
refuses a stem hash that more than one task shares, and the refusal names
each task's stem and hash.

`file` is the task's path, relative to the working directory and in your
platform's #link(u("/cli/#paths"))[separator]. Its cell is
never empty, so `nonempty()` can't hide it. It shows when stdout isn't a
terminal, because a saved stem hash can turn ambiguous once a new task's hash
shares its prefix. On a terminal it shows when you name it, as in
`--columns slug#,title,file`:

#terminal("ls-file-when")

`file` is for display, and a filter can't read it. The
#link(u("/filters/"))[`path`] field holds the path from the project root.
`--format path` prints the same cell as `file`, and
#link(u("/schema/"))[Schema] describes what the JSON and CSV formats carry.

`--hyperlinks always`, or `[ls] hyperlinks`, turns the `slug#` column, and
`file` when shown, into OSC 8 links to `[ls] hyperlink-target` (default
`file://{abs}`). `--hyperlinks auto` turns them on for a terminal whose `TERM`
isn't `dumb`. The default is `never`, because a terminal without OSC 8 support
can print the link as garbage.

`mt ls -i` keeps the table open and reads keys. Type a stem hash to highlight
the tasks whose hash starts with it. `Enter` opens the first highlighted task
with `mt go`, and `Ctrl+Enter` opens every listed task. `Space` reloads, and
`Esc` or `Ctrl+C` quits. It needs a terminal and the table format.
`mt find TERM…` is `mt ls` with each term turned into a `contains()` filter.

= Conditional rules

With `[[ls.rules]]`, a bare `mt ls` picks its listing from where it runs: the
git branch, whether stdout is a terminal, an environment variable or the day.
`mt ls` uses the first entry whose `condition` holds or that has no
`condition`. It applies the entry's `import-views` from left to right, each
view replacing the facets it sets, and then the entry's own `filter`, `sort`
and `columns`. A matched entry ignores the default view.

#config-keys-table(json("../../generated/config-schema.json"), "ls.rules")

`mt` checks each entry when it loads the config. `condition` must parse and
may use only the functions below, so a task field there is an error. Each
`import-views` entry must name a declared view, and each override must be
valid.

A `condition` and a column's `when` share one condition language. It has the
operators of the #link(u("/filters/"))[filter language], but it reads facts
about the run instead of task fields:

#table(
  columns: 2,
  table.header([Function], [Value]),
  [`on_branch("glob")`],
  [Whether the current git branch matches the glob. `false` outside a git repository or on a detached `HEAD`.],

  [`tty()`], [Whether stdout is a terminal.],
  [`env("NAME")`], [The environment variable, or `""` when it's unset.],
  [`weekday()`], [Today's ISO weekday, from `1` (Monday) to `7` (Sunday).],
  [`now()`], [Today's ISO date, the same date the filter language reads.],
  [`format()`],
  [The `--format` value (`table`, `json`, `csv`, `path` or `path0`), or `""` outside a listing.],

  [`nonempty()`], [Whether any cell in this column has text.],
  [`contested()`],
  [Whether the rows have different values in this column. A blank cell counts as a value, and one row is never contested.],

  [`named()`],
  [Whether the caller named this column in `--columns`, a view's `columns` or an `[[ls.rules]]` override.],

  [`blocked()`],
  [Whether any listed task is blocked. The filter field `blocked` is about one task.],

  [`true`, `false`], [Constants.],
)

`nonempty()`, `contested()`, `named()` and `blocked()` describe the column
being decided. In an `[[ls.rules]]` condition no column is being decided, so
they're `false` there, and a rule can't pick its columns from `named()`.

`on_branch()`, `tty()`, the four column functions, `true` and `false` are
booleans, written bare (`!tty()`). `env()`, `weekday()`, `now()` and
`format()` are values, so they go in a comparison. Either side of a
comparison may be a value or a literal, as in `weekday() >= 6` or
`"review" == env("MODE")`. You can't compare `true` or `false`.

The line under a table says where the rows came from and how many of the
project's tasks it shows, as in `default view · 3 of 12 tasks`. When a rule
sets the sort, its name follows, as in `inbox view · release sort · 3 of 12 tasks`.
A filter you typed reads `your filter · 1 of 4 tasks`, and `mt ls --json`
says where each facet came from. In `examples/api-service` no rule matches, so
a bare `mt ls` applies the default view:

#terminal("ls")

A task file that fails to load gets a warning naming it, and the line under
the table counts it last, as in `default view · 1 of 2 tasks · 1 broken`.
`mt ls` still lists the rest and exits 0. `mt check` fails on a broken file.
