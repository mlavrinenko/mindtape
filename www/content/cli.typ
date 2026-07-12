#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, terminal-ansi, callout
#import "../utils/site.typ": u

#show: page.with(
  title: "CLI reference",
  active: "/cli/",
  summary: "Every mt command, grouped. Deep topics link out to their own pages.",
)

`mt` is the whole surface. Commands are quiet by default and return honest exit
codes. Every transcript and help block below is captured from the binary at
build time against the #link(u("/quickstart/"))[learn-piano example], so nothing
here can drift from what `mt` actually prints.

#terminal(read("../generated/version.txt"))

`mt --help` lists the commands and the global flags every one accepts (`-w`,
`--color`, `-v`, `--format`):

#terminal(read("../generated/help.txt"))

= Files

`mt add` files a new task from a template — positional titles, `--set`, JSON, or
editor mode. Full detail on #link(u("/templates/"))[Templates & slots].

#terminal(read("../generated/help-add.txt"))

`mt go <ref>` opens a task in your editor (`[go] editor` → `$VISUAL` → `$EDITOR`).
`mt eval <file>` evaluates a single task file and prints it (`--json` for machine
output):

#terminal(read("../generated/eval.txt"))

#terminal(read("../generated/help-go.txt"))

#terminal(read("../generated/help-eval.txt"))

= Listing

`mt ls [filter]` lists tasks under the configured roots. Omitting the filter
(and any `@view`) first tries the project's `[[ls.rules]]` — see Conditional
rules, below — then falls back to the default view (`[ls] default-view`, else
`[views.default]`); an empty filter (`mt ls ''`) skips both and lists
everything. It accepts a #link(u("/filters/"))[filter expression], sorts, and
limits:

#terminal-ansi(read("../generated/ls.ansi"))

```sh
mt ls ''                        # empty filter — lists everything
mt ls 'has_tag("classical")'    # filter expression
mt ls ready --sort priority:asc -n 10   # predicate + sort + limit
mt ls @stuck                    # named view — filter + sort + columns
mt ls --columns status,title    # one-off column override
mt ls --json                    # --json is shorthand for --format json
mt ls --format csv              # table (default), json, or csv
mt ls -i                        # interactive: type a #, Enter opens it
```

Empty columns auto-hide; tune them with `[ls.columns]` and `columns-order`, or
override for one run with `--columns`.

#terminal(read("../generated/help-ls.txt"))

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

#terminal(read("../generated/views.txt"))

#terminal(read("../generated/help-views.txt"))

= Status

`mt flip <status> <refs…>` sets a status dated today. Every legend value is also
a shortcut verb, and flips batch:

#terminal(read("../generated/done.txt"))

```sh
mt flip blocked 1b 2c -m "waiting on infra"   # batch + note
mt flip --list                                # show the verbs
mt flip done 1b --force                       # bypass the transition graph
mt review <ref>                               # stamp reviewed: with today's date
```

When the legend declares `to` transition edges (see #link(u("/config/"))[Config]),
`mt flip` rejects a move that is not an allowed edge and lists the valid next
states; `--force` overrides. Tasks are referenced by slug, list handle (`1b`), or
path. `mt flip --list` prints the shortcut verbs generated from the legend:

#terminal(read("../generated/flip-list.txt"))

#terminal(read("../generated/help-flip.txt"))

= Recurring tasks

A task with a `recur:` field (see #link(u("/dsl/"))[Task DSL]) advances only
via `mt recur` — never as a side effect of a read (`mt ls`/`mt check` spawn or
mutate nothing). Materialize is per-task (`spawn`, the default, or `mutate`;
`[tasks.recur] materialize` sets the project default):

```sh
mt recur hanon-warmup-daily              # spawn a new file, or refresh in place
mt recur hanon-warmup-daily --dry-run    # preview without writing
mt recur hanon-warmup-daily --advance 4  # pre-materialize 4 future spawns (spawn only)
mt recur hanon-warmup-daily --skip       # advance the anchor/count, materialize nothing
mt recur hanon-warmup-daily --force      # spawn again even if a successor already exists
```

`spawn` writes the next occurrence as a new file — its stem minted by
`[tasks.naming]` (or the built-in `{slug}-{date}` fallback), linked back to its
predecessor with a `recurred-from` edge — and refuses a second spawn unless
`--force`. `mutate` resets the same file's status and dates in place instead;
one stem forever, the audit trail is git history. Either way `mt done` on a
recurring task stays a pure close — it never strips `recur` — and prints a
notice: a `spawn` series points back at `mt recur`, a `mutate` series notes
that closing ends it.

#terminal(read("../generated/help-recur.txt"))

= Validate and inspect config

`mt check` validates every task: enum values, required fields, numeric bounds,
link resolution, and dependency cycles. It exits non-zero and names each
violation when one exists — drop it in CI or a pre-commit hook.

#terminal(read("../generated/check.txt"))

#terminal(read("../generated/help-check.txt"))

`mt config show` prints the effective merged config as re-parseable TOML;
`--origin` lists which layers contributed, and from where:

#terminal(read("../generated/config-origin.txt"))

```sh
mt config show              # effective config as re-parseable TOML
mt config show --origin     # which config layers contributed, and from where
mt config show --defaults   # just the embedded floor
mt config show --json       # effective config as JSON
```

#terminal(read("../generated/help-config.txt"))

= Retire

`mt rm` retires tasks: it mints a `swh:1:rev` tombstone, retargets every inbound
`links:` edge (any kind) onto it, and does not commit. Its built-in default
filter is `terminal`.

```sh
mt rm abandoned-rondo-alla-turca  # by ref
mt rm 'status == "cancelled"'     # by filter — every match
mt rm terminal --dry-run          # preview without writing
```

#terminal(read("../generated/help-rm.txt"))

= Setup

`mt install` installs the `@local/mindtape` prelude and seeds the XDG global
defaults. `mt init` writes `.mindtape/config.toml` (`--full` for the expanded
form). See #link(u("/quickstart/"))[Quickstart]. `mt worktrees` lists the
`@name` worktree aliases (see #link(u("/agents/"))[Run from anywhere]).

#terminal(read("../generated/help-init.txt"))

#terminal(read("../generated/help-install.txt"))

#terminal(read("../generated/help-worktrees.txt"))

#callout("note")[
  Scripting or driving `mt` from an agent? #link(u("/agents/"))[For AI agents]
  collects the JSON surfaces, exit codes, and env inputs in one place.
]
