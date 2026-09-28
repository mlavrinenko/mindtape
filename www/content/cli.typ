#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u
#import "../utils/cli-ref.typ": cli-gate, cli-group, render-globals

#let meta = (
  route: "/cli/",
  label: "CLI",
  title: "CLI reference",
  summary: "Every mt command, grouped, with generated transcripts and pointers to the pages that own each deep topic.",
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

A task's identity is its filename stem, so renaming the file is renaming the
task: `mt mv` moves it and retargets every inbound `links:` edge onto the new
stem in the same validated pass, leaving `mt check` with no dangling edge to
report. The `title:` is a separate field and a separate edit.

#cli-group(cli, cli-groups, "Files")

= Fields

`mt set <task> <field>=<value>` is the generic field writer: one field per
call, validated before the write, so a value the project refuses leaves the
file byte-identical. It covers the fields whose Typst literal is one
unambiguous value; the ones with a command of their own — `status`, `links:`,
`reviewed:`, `recur:`, and the stem — are refused naming that command, because
each of those carries more than a value (a date, a kind and a note, a
retargeted graph).

#cli-group(cli, cli-groups, "Fields")

= Listing

`mt ls` is the read side: a #link(u("/filters/"))[filter expression], a sort,
and a limit, with an `@name` #link(u("/named-sets/"))[view] standing in for all
three. Omitting the filter tries the project's `[[ls.rules]]`, then the default
view; an empty filter (`mt ls ''`) matches everything —
#link(u("/filters/"))[Filters] owns that resolution. This page carries the
stacking: a filter argument replaces only the `filter` facet, leaving the
winning rule's or view's sort and columns in place, `--sort` and `--columns`
each take over their own facet, and a bare `@view` argument overrides every
facet that view actually sets. `@default` splices the default view's
contribution — nothing, in a project that leaves it alone, and "nothing" is how
the built-in column set and sort are asked for — so a narrowed listing still
reads through the ordinary table:

```
mt ls 'assignee == "ada"' --columns @default --sort @default
```

#cli-group(cli, cli-groups, "Listing")

`-n` cuts after the sort; `--skip` drops rows before `-n` cuts, so the two
compose into pages: `mt ls -n 20` then `mt ls -n 20 --skip 20` is page two.
A cut table names both what it hid and the exact next command — filter and
flags reproduced, `--skip` advanced past what this page showed, never
restated from an existing `--skip` on the line:

```
truncated: 20 of 143 — next: mt ls -n 20 --skip 20
```

`--json` carries the same fact structurally: a `truncated.next_skip` key
alongside `.shown`/`.total`, `null` on a complete listing —
#link(u("/schema/"))[the schema] owns the envelope shape,
#link(u("/agents/"))[for AI agents] the paging recipe.

Empty columns auto-hide, each on its own `[ls.columns.<name>] when` — one
boolean expression, the same #link(u("/filters/"))[condition language]
`[[ls.rules]]` takes, resolved against the rows it would render. It is the
whole answer to "does this column render": no list, so no ordering and no
precedence to know, and a column with no `when` never renders. A scalar
replaces wholesale across config layers, so `when = 'false'` on one line turns
a column off.

`file` (the task's source path) is the one cell that is never blank, so the
ordinary auto-hide cannot apply to it — off a terminal it turns on, because the
`#` handle beside it stops being an address there, and on a terminal it turns
on when named (`--columns '#,title,file'`, a view, an `[[ls.rules]]` override,
or `when = 'true'`):

```toml
[ls.columns.file]
when = 'named() || (!tty() && format() == "table")'
```

Its cell is relative to the working directory, a display rather than a
filterable value — the field that compares a task's path is
#link(u("/filters/"))[`path`]. `--format path` renders that same cell;
#link(u("/schema/"))[Schema] owns what the record formats carry instead.

The rest of the floor follows that shape: `#` and `title` are `true`,
`status`, `priority` and `due` are `nonempty()`, `assignee` is `contested()`,
and `ready` is `blocked()`. Naming the `assignee` column does not override its
rule — `[ls.columns.assignee] when = 'true'` does.

`why` is `named()`: it lists what each blocked task waits on — its unfinished
direct blocking-kind targets with their status — and renders only when asked
for. `mt ls blocked --columns title,why` answers what to unblock without
walking any links; `--json` carries the same list as `why` on every task.
`children` and `progress` are `named()` too: a parent's child count and its
finished children over all of them, `3/5` — see
#link(u("/links/"))[Parents and progress].

The `#` column is a handle: an ordinal followed by check characters derived
from the task's id, the ordinal being the task's position in the view
#emph[recomputed on every command]. A handle whose ordinal moved resolves to a
task whose check disagrees, and the command is refused rather than run against
the wrong task; `[handle] check-len` buys more characters, and a handle of any
other length is refused outright rather than matched as a prefix.

`--hyperlinks always` (or `[ls] hyperlinks`) makes the `#` column, and `file`
when shown, clickable in a standalone terminal via an OSC 8 sequence to
`[ls] hyperlink-target` (default `file://{abs}`). `--hyperlinks auto` is a
TTY/non-`dumb`-`TERM` heuristic and the default is `never`, because an
unsupported terminal can render the payload as visible garbage — unlike
colour, which simply does nothing.

`mt ls -i` (`--interactive`) holds the terminal in a key loop over the table:
a `#` ordinal highlights a task (prefix match), `Enter` opens the first
highlighted one via #link(u("/cli/"))[`go`], `Ctrl+Enter` opens every listed
task, `Space` re-evaluates, and `q` / `Ctrl+C` leaves. Table output only, on a
TTY. `mt find TERM…` is the same command with `contains()` sugar.

= Conditional rules

Two keys take an expression in this mini-language: `[[ls.rules]]`'s
`condition`, which picks the listing bundle, and `[ls.columns.<name>]`'s
`when`, which decides one column's visibility; both read it against the same
ambient facts, so an expression means one thing wherever it is written. The
rule shape and its `import-views` composition are on
#link(u("/config/"))[Configuration]. A matched rule composes only what it uses
— it does not inherit `default-view` as a base layer — and what follows is the
language itself.

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
belong in a comparison. A comparison takes a value or a literal on either side
(`weekday() >= 6`, `"review" == env("MODE")`) — more general than filter
expressions, which only ever compare a field on the left. `true`/`false` are
the one exception: they're valid only in boolean position, never as a
comparison operand, since every value-producing predicate here yields a string
or a number and a bool against one is a mistake to reject rather than a case to
define. A task field name in a condition is a load-time config error, since the
condition grammar has no fields at all.

The `applied:` trailer names the source that won each facet, and names a rule
that matched and then lost every facet to the command line rather than letting
it disappear (`applied: manual (filter) · release superseded`):

#terminal(read("../generated/ls.txt"))

= Graph

`mt graph` emits the same dependency edges `ready`/`blocked` read, as Graphviz
DOT or Mermaid, nodes labelled by title and status and filled by readiness.
#link(u("/links/"))[Task links] owns the edge kinds, and
#link(u("/filters/"))[Filters] the dependencies a filter scopes.

#cli-group(cli, cli-groups, "Graph")

= Links

`mt link` is the write side of that graph, and the only command that writes
`links:`. The edge kinds, their target validation, and the adjacency map
`--apply` reads are #link(u("/links/"))[Task links]'s.

#cli-group(cli, cli-groups, "Links")

= Views

Named views, and what an `@name` means in a listing, are defined on
#link(u("/named-sets/"))[Views & dictionaries].

#cli-group(cli, cli-groups, "Views")

= Status

`mt flip <status> <refs…>` sets a status dated today; every legend value is
also a verb, and flips batch. Transition edges and required notes are the
#link(u("/config/"))[status legend]'s, and `mt flip` is the only command that
runs a project's #link(u("/hooks/"))[`[[on.flip]]` hooks]. A landed flip reports
the refreshed handle of every task it moved (`handle <stem>: 1b → 3b`, or
`→ none` when the flip took it out of the view), computed against one view
before the batch and one after.

#cli-group(cli, cli-groups, "Status")

= Recurring tasks

`recur:` advances only through `mt recur`, never as a side effect of a read;
the cadence grammar, the policy floor, and what a spawn leaves behind are
#link(u("/recurring/"))[Recurring tasks]'s.

#cli-group(cli, cli-groups, "Recurring tasks")

= Validate and inspect config

`mt check` runs the whole validation pass — enums, required fields, numeric
bounds, link resolution, dependency cycles — and exits non-zero naming each
violation; #link(u("/concepts/"))[Concepts] owns the two layers that enforce a
task, and #link(u("/config/"))[Configuration] the bounds and rules themselves.
`mt check --commit-msg <file>` checks a commit message instead, from a
`commit-msg` hook, against #link(u("/config/"))[Configuration]'s commit rules.
`mt config show` prints the effective merged config as re-parseable TOML, and
`--origin` names the layers behind it.

#cli-group(cli, cli-groups, "Validate & inspect config")

= Retire

`mt rm` retires tasks: it mints a `swh:1:rev` tombstone and retargets every
inbound `links:` edge onto it, without committing. Without git,
#link(u("/config/"))[`[rm] strategy`] picks how the inbound edges are handled.

#cli-group(cli, cli-groups, "Retire")

= Setup

`mt install` installs the prelude, the XDG defaults, the config schema and the
shell completion hooks;
`mt init` writes the project config from a named preset, `mt preset` manages
those presets, and `mt worktrees` lists the `@name` aliases. Their entries
below carry the flags; #link(u("/quickstart/"))[Quickstart],
#link(u("/config/"))[Configuration] and #link(u("/agents/"))[For AI agents]
own what they write.

#cli-group(cli, cli-groups, "Setup")

#callout("note")[
  Scripting or driving `mt` from an agent? #link(u("/agents/"))[For AI agents]
  collects the JSON surfaces, exit codes, and env inputs in one place.
]
