#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, callout
#import "../utils/site.typ": u
#import "../utils/cli-ref.typ": cli-gate, cli-group, render-globals

#show: page.with(
  title: "CLI reference",
  active: "/cli/",
  summary: "Every mt command, grouped. Deep topics link out to their own pages.",
)

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
  ("Listing", ("ls",)),
  ("Graph", ("graph",)),
  ("Views", ("views",)),
  ("Status", ("flip", "review")),
  ("Recurring tasks", ("recur",)),
  ("Validate & inspect config", ("check", "config")),
  ("Retire", ("rm",)),
  ("Setup", ("init", "install", "worktrees")),
)
#cli-gate(cli, cli-groups)

Every command accepts the same global flags:

#render-globals(cli)

= Files

File a new task, open one in your editor, or evaluate a single file.

#cli-group(cli, cli-groups, "Files")

= Listing

`mt ls [filter]` lists tasks under the configured roots. Omitting the filter
(and any `@view`) first tries the project's `[[ls.rules]]` — see Conditional
rules, below — then falls back to the default view (`[ls] default-view`, else
`[views.default]`); an empty filter (`mt ls ''`) skips both and lists
everything. It accepts a #link(u("/filters/"))[filter expression], sorts, and
limits.

#cli-group(cli, cli-groups, "Listing")

Empty columns auto-hide; tune them with `[ls.columns]` and `columns-order`, or
override for one run with `--columns`.

`mt ls -i` (`--interactive`) holds the terminal in a key loop over the listed
table instead of printing once: type a task's `#`-column ordinal to highlight it
(prefix match), `Enter` opens the first highlighted task in the editor (via
#link(u("/cli/"))[`go`]), `Ctrl+Enter` opens every listed task, `Space`
re-evaluates the project, and `q` / `Ctrl+C` leaves. Table output only, and it
needs a TTY.

= Conditional rules

Ordered `[[ls.rules]]` entries let bare `mt ls` adapt to ambient project state
— see #link(u("/config/"))[Config] for the table shape. The first entry whose
`condition` holds (or that omits `condition` entirely) wins; it composes its
filter, sort, and columns from `import-views` (left to right, per-facet
last-wins) plus an optional manual override (highest precedence). No match
falls back to the default view exactly as before. The dim line under the
table names which source won each facet:

#terminal(read("../generated/ls.txt"))

`condition` is a separate mini-language over ambient environment facts, not
task fields — the same operators and parser as
#link(u("/filters/"))[filters], evaluated against a different domain:

#table(
  columns: 2,
  table.header([Predicate], [Matches]),
  [`on_branch("glob")`], [The current git branch (`.git/HEAD`), glob-matched. `false` off a git repo or on a detached `HEAD`.],
  [`env("NAME")`], [The named environment variable (`""` when unset).],
  [`weekday()`], [Today's ISO weekday, `1` (Monday) through `7` (Sunday).],
  [`now()`], [Today's ISO date — same as the filter-language `now()`.],
)

A comparison takes a value or a literal on either side (`weekday() >= 6`,
`"review" == env("MODE")`) — more general than filter expressions, which only
ever compare a field on the left. A task field name in a condition is a
load-time config error, since the condition grammar has no fields at all.

```toml
[[ls.rules]]
name = "release"                              # shown in the trailer / JSON
condition = 'on_branch("release/*")'          # absent = always matches
import-views = ["inbox", "release(sort)"]     # "view" = all facets, "view(f, ...)" a subset
filter = '!terminal && due <= "2026-08-01"'   # optional manual override, highest precedence
```

= Graph

`mt graph [filter]` emits the task dependency graph — the typed `links:` edges —
as Graphviz DOT (default) or Mermaid (`--format mermaid`). Nodes are tasks
labelled by title and status and filled by readiness (ready, blocked, terminal);
edges are styled per relation kind. The optional
#link(u("/filters/"))[filter expression] scopes the subgraph — edges to tasks
outside it drop — and `--no-terminal` prunes closed tasks. It reuses the same
graph model that computes ready/blocked, so it is a pure read: a diffable view
of the same source, no new store.

#cli-group(cli, cli-groups, "Graph")

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
that closing ends it.

= Validate and inspect config

`mt check` validates every task: enum values, required fields, numeric bounds,
link resolution, and dependency cycles. It exits non-zero and names each
violation when one exists — drop it in CI or a pre-commit hook. `mt config show`
prints the effective merged config as re-parseable TOML; `--origin` lists which
layers contributed, and from where.

#cli-group(cli, cli-groups, "Validate & inspect config")

= Retire

`mt rm` retires tasks: it mints a `swh:1:rev` tombstone, retargets every inbound
`links:` edge (any kind) onto it, and does not commit. Its built-in default
filter is `terminal`.

#cli-group(cli, cli-groups, "Retire")

= Setup

`mt install` installs the `@local/mindtape` prelude, seeds the XDG global
defaults, and writes the config JSON Schema (`config.schema.json`) beside them.
`mt init` writes `.mindtape/config.toml` from a named preset (`--preset NAME`,
`--list-presets`; `--annotate` keeps the reasoning) with a
first-line `#:schema` comment directive pointing at that schema, so a
schema-aware TOML editor (tombi, Taplo) assists filling values; `--schema
URL|PATH` retargets it and `--no-schema` (on either command) opts out. See
#link(u("/quickstart/"))[Quickstart]. `mt worktrees` lists the `@name` worktree
aliases (see #link(u("/agents/"))[Run from anywhere]).

#cli-group(cli, cli-groups, "Setup")

#callout("note")[
  Scripting or driving `mt` from an agent? #link(u("/agents/"))[For AI agents]
  collects the JSON surfaces, exit codes, and env inputs in one place.
]
