#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u
#import "../utils/cli-ref.typ": cli-gate, cli-group, render-globals
#import "../utils/config-ref.typ": config-keys-table

#let meta = (
  route: "/cli/",
  label: "CLI",
  title: "CLI reference",
  summary: "Every `mt` command by group, with generated transcripts and links to the pages that cover each topic in depth.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt` is the whole interface. A command prints nothing it does not have to,
and its exit code says whether it did what was asked. The build generates
this page from the binary: the commands and flags come from `mt`'s own
argument parser (#link(u("/agents/"))[`mt _cli --json`]). Each example is a
command the build ran in a terminal, most of them in the
`examples/api-service` project, a backend team's backlog. The page prints the
command from that run.

#terminal("version")

// The command tree, read from the binary. cli-notes.typ adds prose and
// examples. cli-gate fails the build when a command has no group or a note
// names no command, so a new subcommand cannot go missing from this page.
#let cli = json("../generated/cli.json")
#let cli-groups = (
  ("Files", ("add", "import", "mv", "go", "eval")),
  ("Fields", ("set",)),
  ("Listing", ("ls", "find")),
  ("Graph", ("graph",)),
  ("Links", ("link",)),
  ("Views", ("views",)),
  ("Status", ("flip", "review")),
  ("Recurring tasks", ("recur",)),
  ("Validate & inspect config", ("check", "config")),
  ("Retire", ("rm",)),
  ("Setup", ("init", "install", "worktrees", "preset")),
)
#cli-gate(cli, cli-groups)

Every command accepts the same global flags:

#render-globals(cli)

= Files

A task's id is its file stem, so renaming the file renames the task. `mt mv`
renames it and rewrites every `links:` edge that points at it in the same
checked write, so `mt check` finds no broken edge afterwards. The `title:`
field is separate, and `mt set` changes it.

#cli-group(cli, cli-groups, "Files")

= Fields

`mt set <task> <field>=<value>` writes one field per call and checks the task
before it writes, so a value the project refuses leaves the file unchanged.
It writes the fields whose Typst value is one plain value. `status`, `links`,
`reviewed` and `recur` each have a command of their own, and `mt set` refuses
them and names that command. It also refuses `estimate`, which is a dict of
units, and `id`, which `mt mv` changes.

#cli-group(cli, cli-groups, "Fields")

= Listing

`mt ls` takes a #link(u("/filters/"))[filter], a sort and a limit, and an
`@name` #link(u("/named-sets/"))[view] can supply all three. A bare `mt ls`
uses the first matching `[[ls.rules]]` entry, described below, or else the
default view. `mt ls ''` lists every task.

The listing has three facets: filter, sort and columns. Each layer sets only
the facets it names. A filter argument replaces the filter and keeps the sort
and columns of the rule or view underneath. `--sort` and `--columns` replace
their own facet. A bare `@view` argument replaces each facet that view sets.
`@default` stands for the default view's facets, which are empty in a project
that leaves the default view alone, so `--columns @default --sort @default`
asks for the built-in columns and sort:

#terminal("ls-default")

#cli-group(cli, cli-groups, "Listing")

`-n` cuts the list after the sort, and `--skip` drops rows before `-n` cuts,
so `mt ls -n 20 --skip 20` is the second page of 20. A cut table says how
many rows it hid and prints the command for the next page, with the filter
and flags kept and `--skip` advanced:

#terminal("ls-cut")

`--json` carries the same facts in `truncated`: `shown`, `total` and
`next_skip`, or `null` for a complete listing. #link(u("/schema/"))[Schema] describes
the JSON, and #link(u("/agents/"))[For AI agents] shows how to page through
it.

= Columns

Each column shows when its `[ls.columns.<name>] when` expression holds. The
expression uses the condition language described under Conditional rules
below, and `mt` evaluates it
against the rows it is about to print. A column with no `when` never shows. A
config layer replaces a `when` whole, so `when = 'false'` hides a column.

The shipped expressions: `slug#` and `title` are `true`. `status`, `priority`
and `due` are `nonempty()`. `assignee` is `contested()`, so it shows when the rows
have different assignees. `ready` is `blocked()`. Naming the `assignee` column
does not show it; `[ls.columns.assignee] when = 'true'` does.

`why`, `children` and `progress` are `named()`, so they show only when asked
for. `why` lists the unfinished blocking targets each task waits on, with
their status. `mt ls blocked --columns title,why` answers what to unblock
without reading any links, and `--json` carries `why` on every task.
`children` counts a task's `parent` children and `progress` shows the finished
ones, as `3/5`; see #link(u("/links/"))[Parents and progress].

`file` is the task's path, relative to the working directory. Its cell is
never empty, so `nonempty()` cannot hide it. It shows when stdout is not a
terminal, because a saved stem hash can turn ambiguous once a new task's hash
shares its prefix. On a terminal it shows when named, as in
`--columns slug#,title,file`:

#terminal("ls-file-when")

`file` is for display, and a filter cannot read it. The
#link(u("/filters/"))[`path`] field holds the path from the project root.
`--format path` prints the same cell as `file`, and
#link(u("/schema/"))[Schema] describes what the JSON and CSV formats carry.

The `slug#` column is the task's stem hash: the shortest prefix, at least
three characters, of a hash of the filename stem that no other task's hash
shares. `mt` hashes every task the project loads, closed ones included, so a
task has the same stem hash in every listing, filtered or not. It changes
when `mt mv` gives the task a new stem, and it grows a character when a new
task's hash shares its prefix, so an old hash is refused rather than naming
another task. Every command that takes a `<task>` accepts a
stem hash. A path or an exact stem wins, and `mt` tries the input as a stem
hash only when no stem matches. A stem hash that more than one task shares is
refused, and the refusal names each task's stem and hash.

`--hyperlinks always`, or `[ls] hyperlinks`, makes the `slug#` column, and
`file` when shown, links to `[ls] hyperlink-target` (default `file://{abs}`), using
OSC 8. `--hyperlinks auto` turns them on for a terminal whose `TERM` is not
`dumb`. The default is `never`, because a terminal without OSC 8 support can
print the link as garbage.

`mt ls -i` keeps the table open and reads keys. Typing a stem hash
highlights the tasks whose hash starts with it. `Enter` opens the first
highlighted task with `mt go`, and `Ctrl+Enter` opens every listed task.
`Space` reloads, and `Esc` or `Ctrl+C` quits. It needs a terminal and the table format. `mt find TERM…`
is `mt ls` with each term turned into a `contains()` filter.

= Conditional rules

`[[ls.rules]]` lets a bare `mt ls` choose its listing from where it runs: the
git branch, whether stdout is a terminal, an environment variable or the day.
`mt ls` uses the first entry whose `condition` holds or that has no
`condition`. It applies the entry's `import-views` from left to right, each
view replacing the facets it sets, and then the entry's own `filter`, `sort`
and `columns`. A matched entry ignores the default view.

#config-keys-table(json("../generated/config-schema.json"), "ls.rules")

`mt` checks each entry when it loads the config. `condition` must parse and
may use only the functions below, so a task field there is an error. Each
`import-views` entry must name a declared view, and each override must be
valid.

A `condition` and a column's `when` use the same condition language. It has
the operators of the #link(u("/filters/"))[filter language], but it reads
facts about the run instead of task fields:

#table(
  columns: 2,
  table.header([Function], [Value]),
  [`on_branch("glob")`],
  [Whether the current git branch matches the glob. `false` outside a git repository or on a detached `HEAD`.],

  [`tty()`], [Whether stdout is a terminal.],
  [`env("NAME")`], [The environment variable, or `""` when it is unset.],
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
being decided, so they are `false` in an `[[ls.rules]]` condition, where no
column is being decided. So a rule cannot pick its columns from `named()`.

`on_branch()`, `tty()`, the four column functions and `true` and `false` are
booleans, written bare (`!tty()`). `env()`, `weekday()`, `now()` and
`format()` are values, so they go in a comparison. Either side of a
comparison may be a value or a literal, as in `weekday() >= 6` or
`"review" == env("MODE")`. `true` and `false` cannot be compared.

The `applied:` line under a table names the source of each facet. When the
command line overrides every facet of a matched rule, the line names that
rule as superseded, as in `applied: manual (filter) · release superseded` for
a rule named `release`. In `examples/api-service` no rule matches, so a bare
`mt ls` applies the default view:

#terminal("ls")

= Graph

`mt graph` prints the dependency edges that `ready` and `blocked` read, as
Graphviz DOT or Mermaid. Each node shows a task's title and status and is
coloured by whether the task is ready. #link(u("/links/"))[Task links]
describes the edge kinds.

#cli-group(cli, cli-groups, "Graph")

= Links

`mt link` is the only command that writes `links:`.
#link(u("/links/"))[Task links] describes the edge kinds, how `mt` checks a
target, and the map `--apply` reads.

#cli-group(cli, cli-groups, "Links")

= Views

#link(u("/named-sets/"))[Views & dictionaries] describes views and what
`@name` means in a listing.

#cli-group(cli, cli-groups, "Views")

= Status

`mt flip <status> <refs…>` stamps a status dated today on one or more tasks.
Each status in the legend is also a command, so `mt done x` is
`mt flip done x`. The #link(u("/statuses/"))[status legend] sets which
changes are allowed and which need a note. `mt flip` is the only command that
runs #link(u("/hooks/"))[`[[on.flip]]` hooks].

#cli-group(cli, cli-groups, "Status")

= Recurring tasks

Only `mt recur` advances a `recur:` field. No read command changes it.
#link(u("/recurring/"))[Recurring tasks] describes the schedule format and
what a new occurrence keeps.

#cli-group(cli, cli-groups, "Recurring tasks")

= Validate and inspect config

`mt check` validates every task: field values, required fields, numeric
bounds, links and dependency cycles. It prints each problem and exits
non-zero when there is one. #link(u("/concepts/"))[Concepts] describes the two
layers that check a task, and #link(u("/rules/"))[Rules] the checks a project
adds. `mt check --commit-msg <file>` checks a commit message instead, from a
`commit-msg` hook, against the project's
#link(u("/commit-rules/"))[commit rules]. `mt check --commits <range>` checks
every commit in a range, for CI. `mt config show` prints the merged config as
TOML that `mt` can read back, and `--origin` names the file each value came
from.

#cli-group(cli, cli-groups, "Validate & inspect config")

= Retire

`mt rm` retires tasks. In a git repository it records a `swh:1:rev` id for
each task, points every `links:` edge at it, and deletes the file, without
committing. Outside git, #link(u("/config/"))[`[rm] strategy`] decides what
happens to those edges.

#cli-group(cli, cli-groups, "Retire")

= Setup

`mt install` installs the Typst package, the user defaults file, the config
schema and the shell completion hooks. `mt init` writes a project's config
from a preset, `mt preset` saves and lists presets, and `mt worktrees` lists
the `@name` project aliases. #link(u("/quickstart/"))[Quickstart],
#link(u("/config/"))[Configuration] and #link(u("/agents/"))[For AI agents]
describe the files they write.

#cli-group(cli, cli-groups, "Setup")

#callout("note")[
  #link(u("/agents/"))[For AI agents] collects the JSON output, exit codes
  and environment variables a script or an agent reads.
]
