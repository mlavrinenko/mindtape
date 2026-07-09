#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, callout
#import "../utils/site.typ": u

#show: page.with(
  title: "CLI reference",
  active: "/cli/",
  summary: "Every mt command, grouped. Deep topics link out to their own pages.",
)

`mt` is the whole surface. Commands are quiet by default and return honest exit
codes.

= Files

`mt add` files a new task from a template — positional titles, `--set`, JSON, or
editor mode. Full detail on #link(u("/templates/"))[Templates & slots].

`mt go <ref>` opens a task in your editor (`[go] editor` → `$VISUAL` → `$EDITOR`).
`mt eval <file>` evaluates a single task file and prints it (`--json` for machine
output).

= Listing

`mt ls [filter]` lists tasks under the configured roots, applying
`[ls] default-filter` unless you pass `--all`. It accepts a
#link(u("/filters/"))[filter expression], sorts, and limits:

#terminal("$ mt ls
#   STATUS    READY    PRIORITY  DUE         TITLE
1e  wip       ready    high      2026-07-20  Wire OAuth login
2g  proposed  blocked  normal                Write onboarding docs")

```sh
mt ls --all                     # ignore the default filter
mt ls 'has_tag("backend")'      # filter expression
mt ls ready --sort priority:asc -n 10   # predicate + sort + limit
mt ls --json                    # --json is shorthand for --format json
mt ls --format csv              # table (default), json, or csv
mt ls -i                        # interactive: type a #, Enter opens it
```

Empty columns auto-hide; tune them with `[ls.columns]` and `columns-order`.

= Status

`mt flip <status> <refs…>` sets a status dated today. Every legend value is also
a shortcut verb, and flips batch:

#terminal("$ mt done wire-oauth-login
flipped wire-oauth-login (Wire OAuth login): wip → done")

```sh
mt flip blocked 1b 2c -m "waiting on infra"   # batch + note
mt flip --list                                # show the verbs
mt flip done 1b --force                       # bypass the transition graph
mt review <ref>                               # stamp reviewed: with today's date
```

When the legend declares `to` transition edges (see #link(u("/config/"))[Config]),
`mt flip` rejects a move that is not an allowed edge and lists the valid next
states; `--force` overrides. Tasks are referenced by slug, list handle (`1b`), or
path.

= Validate and inspect config

`mt check` validates every task: enum values, required fields, numeric bounds,
link resolution, and dependency cycles.

#terminal("$ mt check
ok: 3 task(s) valid")

`mt config show` prints the effective merged config as re-parseable TOML:

```sh
mt config show --origin     # which config layers contributed, and from where
mt config show --defaults   # just the embedded floor
mt config show --json       # effective config as JSON
```

= Retire

`mt rm` retires tasks: it mints a `swh:1:rev` tombstone, rewrites inbound
`link()`s, and does not commit. Its built-in default filter is `terminal`.

```sh
mt rm implement-auth              # by ref
mt rm 'status == "cancelled"'     # by filter — every match
mt rm terminal --dry-run          # preview without writing
```

= Setup

`mt install` installs the `@local/mindtape` prelude and seeds the XDG global
defaults. `mt init` writes `.mindtape/config.toml` (`--full` for the expanded
form). See #link(u("/quickstart/"))[Quickstart].

= Global flags

#table(
  columns: 2,
  table.header([Flag], [Effect]),
  [`--color <auto\|always\|never>`], [Colour output. `auto` (default) colours when stdout is a TTY and `NO_COLOR` is unset.],
  [`-v` / `-vv` / `-vvv`], [Increase verbosity: info, debug, trace.],
  [`--format <table\|json\|csv>`], [Output format where a command supports it; `--json` is shorthand for `--format json`.],
)

#callout("note")[
  Scripting or driving `mt` from an agent? #link(u("/agents/"))[For AI agents]
  collects the JSON surfaces, exit codes, and env inputs in one place.
]
