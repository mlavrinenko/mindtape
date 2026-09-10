#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u
#import "../utils/cli-ref.typ": cli-gate, cli-group, render-globals

#let meta = (
  route: "/cli/",
  label: "CLI",
  title: "CLI reference",
  summary: "Every mt command, grouped, with the deep topics linked out to their own pages.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt` is the whole surface. Commands are quiet by default and return honest exit
codes. Every command, flag, and sample transcript below is generated from the
binary at build time — the command tree by reflecting `mt`'s own clap
definition (#link(u("/agents/"))[`mt _cli --json`]), the transcripts by running
against the #link(u("/quickstart/"))[learn-piano example] — so nothing here can
drift from what `mt` actually is.

#terminal(read("../generated/version.txt"))

// The command tree, reflected from the binary. cli-notes.typ enriches it with
// reference-only prose and examples; cli-gate asserts every command is grouped
// and every note attaches to a live node — a new subcommand cannot silently
// vanish from this page.
#let cli = json("../generated/cli.json")
#let cli-groups = (
  ("Files", ("add", "import", "go", "eval")),
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

#cli-group(cli, cli-groups, "Files")

= Listing

`mt ls [filter]` lists tasks under the configured roots, and accepts a
#link(u("/filters/"))[filter expression], sorts, and limits. Which tasks it
lists resolves in order:

- the project's `[[ls.rules]]` first — see Conditional rules, below;
- then the default view: `[ls] default-view`, else `[views.default]`.

A filter argument replaces whichever of those won the `filter` facet — an empty
one (`mt ls ''`) is the match-all filter and lists everything — while the same
rule or view goes on supplying the `sort` and `columns` it set. Narrowing a
listing is not resetting it; `--sort` and `--columns` are how you take over the
presentation, each on its own facet. A bare `@view` argument overrides every
facet that view sets.

#cli-group(cli, cli-groups, "Listing")

Empty columns auto-hide; tune them with `[ls.columns]` and `columns-order`, or
override for one run with `--columns`. A `file` column (the task's source
path) exists but is opt-in on a terminal — its cell is never blank, so the
ordinary auto-hide rule cannot apply to it; name it explicitly (`--columns
'#,title,file'`, a view, an `[[ls.rules]]` columns override, or `[ls.columns.file]
when = 'true'`) to see it. Its cell is relative to the working
directory (what an IDE-integrated terminal ctrl-clicks straight to) — a
display, not a filterable value: the field that compares a task's path is
#link(u("/filters/"))[`path`], and it is root-relative.

That is the table's cell, and `--format path` renders the same one. The
record formats do not: a `file` in `--format csv` is the root-relative path,
byte-identical to the `file` key in
#link(u("/schema/"))[`--format json`] — a record means the same thing
wherever it was produced, so a stored one never has to carry the directory
that produced it.

#emph[Off] a terminal it turns itself on, because the `#` handle beside it
stops being an address there. Captured output outlives the listing that
produced it — it is pasted into a report, a commit message, a chat — and the
handle's ordinal is a position in a view every command recomputes, so a handle
quoted an hour later names whatever now sits at that position. Every other
renderer had already settled this: `--json` carries `id` and `file` and no
handle at all, and `--format csv` substitutes `id` for `#`. The table was the
last surface handing out a positional id with no stable identity beside it.

The mechanism is a line of the embedded floor, not a special case in the
renderer:

```toml
[ls.columns.file]
when = 'named() || (!tty() && format() == "table")'
```

`[ls.columns.<name>] when` is one boolean expression in the same
#link(u("/cli/"))[condition language] `[[ls.rules]]` takes, and it is the
whole answer to "does this column render" — no list, so no ordering and no
precedence to know. Three predicates carry this one: `named()` (did the caller
ask for this column), `tty()` (is stdout a terminal) and `format()` (the active
`--format` name). It names `table` because `csv` resolves the same column set
and is a data contract — a script parsing it must not silently gain a column,
and it already carries `id`.

A `when` is resolved against the rows it would render, so it can read what the
listing says: `nonempty()` (any cell carries text), `contested()` (the rows
disagree), `named()`, and `blocked()` (some listed task is blocked). Those four
are what let every column's visibility live in config rather than in the
renderer, and the floor uses all of them: `#` and `title` are `true`,
`status`, `priority` and `due` are `nonempty()`, `assignee` is `contested()`,
`ready` is `blocked()`, and `file` is the expression above.

So the floor is the complete statement of what `mt ls` shows. A project
overrides one column by writing its own `when` — a scalar replaces wholesale
across config layers, so `when = 'false'` on one line turns a column off — and
a column with no `when` at all never renders.

An `assignee` column auto-hides on a rule of its own: it shows only when the
listed rows disagree about who works them. A listing already narrowed to one
actor (`mt ls @agent`, or any `assignee == "…"` filter) learns nothing from a
column repeating that name down every row, so it does not grow one — while a
mixed listing shows the split without being asked. An unassigned row counts as
its own value, so a half-assigned listing is a mixed one. Naming the column
does not override this, the same way an explicitly listed `priority` stays
hidden when nothing scores one; `[ls.columns.assignee] when = 'true'`
is the override.

The `#` column is a handle: an ordinal followed by check characters derived
from the task's id. The ordinal is the task's position in the view #emph[as
  recomputed on every command], never a token the last listing handed out, so a
flip that re-sorts or filters the list renumbers it under anyone still holding
one — which is why `mt flip` reports the refreshed handle of every task it
moved. The check is what catches the rest — a handle whose ordinal moved
resolves to a task whose check disagrees, and the command is refused rather
than run against the wrong task. One character catches most shifts; `[handle]
check-len` buys more at a character apiece, and a handle of any other length is
then refused outright rather than matched as a prefix, since a short handle is
a valid handle for some other task.

`--hyperlinks always` (or `[ls] hyperlinks`) makes both the always-visible `#`
column and, when shown, `file` clickable in a standalone terminal via an OSC 8
hyperlink sequence — the same row's target either way, targeting `[ls]
hyperlink-target` (default `file://{abs}`) — so `#` alone gives a
click-navigable listing without spending a column on a path. There is no way
to ask a terminal whether it supports OSC 8, so `--hyperlinks auto` is a
heuristic (TTY, non-`dumb` `TERM`) and the default is `never`, not `auto` — an
unsupported terminal can render the escape payload as visible garbage, unlike
colour, which just does nothing.

`mt ls -i` (`--interactive`) holds the terminal in a key loop over the listed
table instead of printing once:

- a task's `#`-column ordinal highlights it (prefix match);
- `Enter` opens the first highlighted task in the editor (via
  #link(u("/cli/"))[`go`]), `Ctrl+Enter` opens every listed task;
- `Space` re-evaluates the project;
- `q` / `Ctrl+C` leaves.

Table output only, and it needs a TTY.

`-n N` (`--limit N`) caps the listing at N rows, cutting after the filter and
the sort so it always yields the top N of the order asked for. It truncates
in-process, where `mt ls | head` truncates by closing the read end under a
still-writing `mt` — same rows, one fewer moving part, and it works on a
format that is not line-oriented.

A cut listing says so, in whatever the format has room for: a dim
`truncated: showing 10 of 47` under the table, the same pair in `--json`'s
#link(u("/schema/"))[envelope], and nothing at all in `csv` / `path` /
`path0`, which are record streams a trailing prose line would corrupt for the
program reading them. A limit that hides nothing prints nothing.

`--format path` prints one task file path per line and nothing else — no
header, no colour, no hyperlinks, no `applied:` or `truncated:` trailer — for
composing with
another program: `rg TODO $(mt ls ready --format path)`. `--format path0` is
the same list NUL-delimited instead, for `xargs -0` and paths holding
whitespace: `mt ls 'stale(30)' --format path0 | xargs -0 $EDITOR`. Both render
the path exactly as the table's `file` column does — relative to the working
directory — so the listing and the list you paste into the next command never
disagree. `contrib/fzf-mt-go.sh` wires
`--format path` to a fuzzy picker of your choice (`mt ls -i` stays a plain
key-loop highlighter, not a fuzzy finder).

`mt find TERM…` is sugar over `contains()`: `mt find login bug` is exactly
`mt ls 'contains("login") && contains("bug")'`, run through the same code
path. Every `ls` flag forwards verbatim (`--format`, `--json`, `--sort`,
`--columns`, `-n`, `-i`, `--color`, `--hyperlinks`, `-w`); there is no `--in`
flag; targeting a field stays in the filter language
(`mt ls 'contains("bug", title)'`). The desugared expression is not hidden —
`mt find bug --json` reports `contains("bug")` in the `applied` envelope, and
the table footer names it the same way a manual filter would.

= Conditional rules

Two keys take an expression in this language: `[[ls.rules]]`'s `condition`,
which picks the listing bundle a command line has not already claimed, and
`[ls.columns.<name>]`'s `when`, which decides one column's visibility for any
`mt ls`. Both read the same mini-language against the same ambient facts, so an
expression means one thing wherever it is written.

Ordered `[[ls.rules]]` entries let `mt ls` adapt to ambient project state — see
#link(u("/config/"))[Config] for the table shape. The first entry whose
`condition` holds (or that omits `condition` entirely) wins; it composes its
filter, sort, and columns from `import-views` (left to right, per-facet
last-wins) plus an optional override of its own (highest precedence within the
rule). No match falls back to the default view exactly as before. The dim line
under the table names which source won each facet, and names a rule that
matched and then lost every facet to the command line rather than letting it
disappear (`applied: manual (filter) · release superseded`):

#terminal(read("../generated/ls.txt"))

`condition` is a separate mini-language over ambient environment facts, not
task fields — the same operators and parser as
#link(u("/filters/"))[filters], evaluated against a different domain:

#table(
  columns: 2,
  table.header([Predicate], [Matches]),
  [`on_branch("glob")`],
  [The current git branch (`.git/HEAD`), glob-matched. `false` off a git repo or on a detached `HEAD`.],

  [`tty()`],
  [Whether stdout is a terminal — whether a human is reading this output directly, rather than a capture that will outlive the command.],

  [`env("NAME")`], [The named environment variable (`""` when unset).],
  [`weekday()`], [Today's ISO weekday, `1` (Monday) through `7` (Sunday).],
  [`now()`], [Today's ISO date — same as the filter-language `now()`.],
  [`format()`],
  [The active `--format` name (`table`, `json`, `csv`, `path`, `path0`); `""` outside a rendering command.],

  [`nonempty()`], [Whether any cell in this column carries text.],
  [`contested()`],
  [Whether the listed rows disagree about this column's cell. A blank counts as its own value, so a half-filled column is contested; one row never is.],

  [`named()`],
  [Whether the caller asked for this column by name — `--columns`, a view's `columns`, or an `[[ls.rules]]` columns override — rather than taking the built-in order.],

  [`blocked()`],
  [Whether #emph[any] listed task is blocked on an unmet dependency — a fact about the listing, unlike the #link(u("/filters/"))[filter] field of the same name, which is about one task.],

  [`true`, `false`],
  [Boolean constants, composable like any other boolean (`!false`, `true && !tty()`).],
)

The last four describe the listing rather than the ambient environment, so
they only have an answer while a column is being resolved. In an `[[ls.rules]]`
condition — which picks the whole bundle, no column in sight — all four read
`false`. That is also what keeps `named()` honest: it reports what was
_requested_, and the one place a `columns` override is itself chosen cannot
consult it.

`on_branch()`, `tty()`, the four listing predicates, and the `true`/`false`
literals are boolean and are used bare (`!tty()`); the rest produce values and
belong in a comparison. A
comparison takes a value or a literal on either side (`weekday() >= 6`,
`"review" == env("MODE")`) — more general than filter expressions, which only
ever compare a field on the left. `true`/`false` are the one exception:
they're valid only in boolean position, never as a comparison operand, since
every value-producing predicate here yields a string or a number and a bool
against one is a mistake to reject rather than a case to define. A task field
name in a condition is a load-time config error, since the condition grammar
has no fields at all.

= Graph

`mt graph [filter]` emits the task dependency graph — the typed `links:` edges —
as Graphviz DOT (default) or Mermaid (`--format mermaid`). Nodes are tasks
labelled by title and status and filled by readiness (ready, blocked, terminal);
edges are styled per relation kind. The optional
#link(u("/filters/"))[filter expression] scopes the subgraph — edges to tasks
outside it drop — and `--no-terminal` prunes closed tasks. It reuses the same
graph model that computes ready/blocked, so it is a pure read: no new store.

#cli-group(cli, cli-groups, "Graph")

= Links

`mt link` is the write side of that same graph, and the only CLI writer for
`links:`. `mt link <task> <kind> <target>...` upserts one edge per target;
`mt link --apply <file>` (`-` for stdin) reads an adjacency map — stem, then
kind, then targets — whose shape is documented on
#link(u("/dsl/"))[Task DSL]. Edges dedupe on (kind, target), so either form is
safe to re-run and a map stays safe to re-apply as it grows.

Every edge is checked before a byte is written: the kind must be configured,
the target must resolve, and a kind whose note is `required` is refused
without one, naming the edge. A per-entry failure is reported and the rest
still land, the way `mt rm` and `mt flip` report a batch.

#cli-group(cli, cli-groups, "Links")

= Views

Named views bundle a filter with an optional sort and column set under
`[views.<name>]` — see #link(u("/config/"))[Config] for the table shape.
Reference one with an `@name` sigil:

- `mt ls @stuck` — bundle mode: applies the view's filter, sort, and columns
  together. A facet flag overrides just that facet: `mt ls @stuck --sort
  @this-week` keeps `stuck`'s filter and columns but takes `this-week`'s sort.
- `@name` inside a larger #link(u("/filters/"))[filter expression]
  (`mt ls '@this-week && !@stuck'`) is the filter facet only — sugar for
  `in_view("name")` — and composes with `&&` / `||` / `!` like any predicate.
  Sort and columns are ignored in that position.
- `--sort @view` splices that view's sort chain (repeatable, composes with
  plain `--sort` specs); `--columns @view` takes its column set.

`mt views` lists the named views; `mt views <name>` shows one view's resolved
definition; `--json` on either for scripting.

#cli-group(cli, cli-groups, "Views")

= Status

`mt flip <status> <refs…>` sets a status dated today. Every legend value is also
a shortcut verb, and flips batch. When the legend declares `to` transition edges
(see #link(u("/config/"))[Config]), `mt flip` rejects a move that is not an
allowed edge and lists the valid next states; `--force` overrides. When a
target entry declares `note = "required"`, `mt flip` refuses the move without
`-m/--note` — no `--force` escape for this one. Tasks are referenced by slug,
list handle (`1b`), or path. `mt flip --list` prints the shortcut verbs
generated from the legend.

A landed flip re-sorts the default view, so the batch closes by saying what
that did to the handles you are holding: `handle <stem>: 1b → 3b` per flipped
task whose ordinal moved, computed against one view taken before the batch and
one after — not one per target, so a batch reports the ordinals that were on
your screen rather than ones that existed only mid-batch. A task the flip
filtered out of the view has no ordinal left, and the line says so naming the
stem (`handle <stem>: 1b → none`), which still addresses it. A flip that moved
nothing — a same-status re-stamp, typically — says nothing.

`mt flip` is also the only command that runs a project's
#link(u("/config/"))[`[[on.flip]]` hooks] — printing a message or a task list,
or running a command that may refuse the flip. `--quiet` suppresses the printed
ones (gates still run); `--force` waives a failing gate and records the waiver
in the status note. No read command executes anything.

#cli-group(cli, cli-groups, "Status")

= Recurring tasks

A task with a `recur:` field (see #link(u("/dsl/"))[Task DSL]) advances only
via `mt recur` — never as a side effect of a read (`mt ls`/`mt check` spawn or
mutate nothing). Materialize is per-task (`spawn`, the default, or `mutate`;
`[tasks.recur] materialize` sets the project default).

#cli-group(cli, cli-groups, "Recurring tasks")

`spawn` writes the next occurrence as a new file — its stem minted by
`[tasks.naming]` (or the built-in `{slug}-{date}` fallback), linked back to its
predecessor with a `recurred-from` edge — and refuses a second spawn unless
`--force`. `mutate` resets the same file's status and dates in place instead;
one stem forever, the audit trail is git history. Either way `mt done` on a
recurring task stays a pure close — it never strips `recur` — and prints a
notice: a `spawn` series points back at `mt recur`, a `mutate` series notes
that closing ends it. Either way the occurrence is validated before it is
written, so a rejected one creates no file and refreshes nothing.

`--advance N` lays out `N` future occurrences at once, spaced by the rule
itself: the first honours the task's `anchor`, and each one after it steps off
the occurrence before it rather than asking the anchor again. That difference is
what makes the flag work under `anchor: "completion"` (the default), which
resolves from today and would otherwise plan the same date `N` times over. The
chain is written as one batch or not at all, since every entry names the one
before it.

= Validate and inspect config

Every write command validates before it writes, never after. A new file that
would not pass is not created; a rewrite that would not pass is not applied, so
the file still holds exactly what its author put there. That is why a failed
`mt add`, `mt recur`, `mt flip`, `mt review`, `mt link` or `mt import` asks you
to fix the input and re-run rather than to repair or delete anything. A batch is
checked as one set, so records or edges that reference each other resolve;
`mt import` and `mt link` then land the entries that passed and report the rest,
while `mt recur --advance` refuses its whole chain, whose entries derive from
one another.

`mt check` validates every task: enum values, required fields, numeric bounds
(including the `[tasks.estimate]` per-unit caps), link resolution, and dependency
cycles. It exits non-zero and names each violation when one exists — drop it in
CI or a pre-commit hook. It also warns, without failing, when the git working
tree has outgrown the file budget the in-flight tasks declare (see
#link(u("/config/"))[Configuration]). `mt config show`
prints the effective merged config as re-parseable TOML; `--origin` lists which
layers contributed, and from where.

#cli-group(cli, cli-groups, "Validate & inspect config")

= Retire

`mt rm` retires tasks: it mints a `swh:1:rev` tombstone, retargets every inbound
`links:` edge (any kind) onto it, and does not commit. Its built-in default
filter is `terminal`.

Without a git repository there is no tombstone to mint, so `[rm] strategy`
picks how the inbound edges are handled instead: `reject` (the default)
refuses, listing every surviving referrer; `unlink` strips the edges and
deletes; `retire` keeps the file and flips it to a terminal status — `[retire]`
tunes the status and an optional archive move. A task nothing references is
deleted plainly, git or not.

#cli-group(cli, cli-groups, "Retire")

= Setup

`mt install` installs the `@local/mindtape` prelude, seeds the XDG global
defaults, and writes the config JSON Schema (`config.schema.json`) beside them.
`mt init` writes `.mindtape/config.toml` from a named preset (`--preset NAME`,
`--list-presets`; `--annotate` keeps the reasoning) with a first-line
`#:schema` comment directive pointing at the published schema URL;
`--schema local` uses the installed copy instead, `--schema URL|PATH` any
other target, and `--no-schema` (on either command) opts out — see
#link(u("/config/"))[Editor schema] and #link(u("/quickstart/"))[Quickstart].
`mt worktrees` lists the `@name` worktree aliases (see
#link(u("/agents/"))[Run from anywhere]).

`mt preset save <name>` captures the current project's `.mindtape/config.toml`
and its `templates/` as a reusable user preset under
`~/.config/mindtape/presets/<name>` — a raw copy, never a reverse-engineered
guess at the wizard answers that produced it. `--force` overwrites an
existing user preset; without it the command refuses and names what is already
there. `mt preset ls` lists built-in and user presets together, marking which
user preset shadows a built-in of the same name, and which is the default.

#cli-group(cli, cli-groups, "Setup")

#callout("note")[
  Scripting or driving `mt` from an agent? #link(u("/agents/"))[For AI agents]
  collects the JSON surfaces, exit codes, and env inputs in one place.
]
