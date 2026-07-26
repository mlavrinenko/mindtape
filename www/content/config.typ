#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u
#import "../utils/config-ref.typ": config-gate, config-sections-table

#show: page.with(
  title: "Configuration",
  active: "/config/",
  summary: "How .mindtape/config.toml overlays a defaults floor, the four-layer merge, and the sections you can set.",
)

`.mindtape/config.toml` overlays a shipped defaults floor. Omit a section to
inherit the floor's sane defaults, declare it to override, or set it empty (e.g.
`status = []`) to leave that aspect unconstrained.

= The four layers

Config is deep-merged in precedence order, lowest first:

+ the embedded floor (shipped with `mt`);
+ XDG global defaults — `~/.config/mindtape/defaults.toml`;
+ every `.mindtape/defaults.toml` walking up from the project root (farthest
  ancestor first, nearest wins);
+ the project's own `.mindtape/config.toml` (highest precedence).

Inspect the result:

```sh
mt config show            # effective config as re-parseable TOML
mt config show --origin   # which config layers contributed, and from where
mt config show --defaults # just the embedded floor
mt config show --json     # effective config as JSON
```

= Sections

Every section below is reflected straight from `mt config schema` — the same
JSON Schema `mt install` writes to disk, generated from the config structs
themselves. A row's purpose column is that property's own doc comment, so a
newly added config key cannot silently go undocumented: the build fails first.
Deep-dive sections further down this page (linked from the prose above and
below) cover the sections whose shape needs more than one line.

// The command tree pattern (cli.typ / cli-ref.typ) reflects clap's own
// definition; this mirrors it for config: `config-sections` is presentation
// (row order, TOML-syntax labels) and config-gate is the anti-drift check —
// every schema property must be covered here, or the build panics.
#let schema = json("../generated/config-schema.json")
#let config-sections = (
  ("tasks.roots", "[tasks] roots"),
  ("tasks.status", "[[tasks.status]]"),
  ("tasks.priority", "[tasks.priority]"),
  ("tasks.difficulty", "[tasks.difficulty]"),
  ("tasks.estimate", "[tasks.estimate]"),
  ("tasks.recur", "[tasks.recur]"),
  ("tasks.naming", "[tasks.naming]"),
  ("tasks.id", "[tasks.id]"),
  ("tasks.fields", "[tasks.fields.<name>]"),
  ("tasks.body.headers", "[tasks.body.headers]"),
  ("tasks.derive-tags", "[tasks.derive-tags]"),
  ("tasks.tags", "[tasks.tags]"),
  ("tasks.limits", "[[tasks.limits]]"),
  ("tasks.comments", "[tasks.comments]"),
  ("tasks.links", "[[tasks.links.kinds]]"),
  ("ls", "[ls]"),
  ("rm", "[rm]"),
  ("views", "[views.<name>]"),
  ("on", "[[on.flip]]"),
  ("templates", "[templates]"),
  ("add", "[add]"),
  ("go", "[go]"),
  ("format", "[format]"),
  ("project", "[project]"),
  ("typst", "[typst]"),
)
#config-gate(schema, config-sections)
#config-sections-table(schema, config-sections)

#callout("note")[
  #link(u("/derived-tags/"))[Derived tags], #link(u("/priority/"))[Priority],
  and #link(u("/filters/"))[Filter language] cover `[tasks.roots]` /
  `[tasks.derive-tags]`, `[tasks.priority]` / `[tasks.difficulty]`, and
  `[views]` / `[ls]` / `[rm]` in full; the #link(u("/cli/"))[CLI reference]
  covers `[[ls.rules]]`.
]

= Example

A compact project config — a status legend, a bounded priority scale, and allowed
estimate units:

```toml
[tasks]
roots = ["tasks"]

[[tasks.status]]
value = "wip"
glyph = "🔨"
color = "yellow"
[[tasks.status]]
value = "done"
glyph = "✅"
color = "green"
terminal = true

[tasks.priority]
kind = "int"
min  = 0
max  = 9
required = "!terminal"

[tasks.estimate]
units = ["minutes", "hours", "days", "weeks"]

# The default `mt ls` listing: a view, so it can carry sort and columns too.
# `[ls] default-view = "name"` points it elsewhere; `default` is the fallback.
[views.default]
filter = "status != \"done\""
```

#terminal(read("../generated/config-origin.txt"))

#callout("note")[
  `mt init` writes one of the named presets (`--list-presets` shows them), and
  `--annotate` keeps that preset's explanatory comments. The two are
  independent: `--preset` picks what the config says, `--annotate` picks whether
  it says why.
]

= Required fields

`required` gates `[tasks.priority]`, `[tasks.difficulty]`, `[tasks.estimate]` and
each `[tasks.fields.<name>]`. It takes `true`, `false`, or a
#link(u("/filters/"))[filter expression] — the field is required for exactly the
tasks that expression matches, evaluated per task by `mt check`:

```toml
[tasks.priority]
required = "!terminal"        # until the task reaches a terminal status

[tasks.estimate]
required = "kind == \"bug\""  # only for bugs

[tasks.fields.kind]
required = "@active"          # only inside the `active` view
```

#callout("note")[
  `required = "non-terminal"` is retired. Write `"!terminal"`, which means the
  same thing in the filter language. A malformed expression fails when the config
  loads, naming the field — not once per task.
]

= Status transitions

Add a `to` list to a status entry to declare which statuses may follow it — the
legend becomes a state machine instead of a flat enum. It is opt-in: with no `to`
on any entry the legend stays any-to-any, exactly as before.

```toml
[[tasks.status]]
value = "proposed"
to = ["accepted", "wip", "cancelled"]
[[tasks.status]]
value = "wip"
to = ["done", "failed", "accepted"]
[[tasks.status]]
value = "done"
terminal = true
to = ["wip"]              # a reopen edge; omit `to` to make a state a sink
```

Once any entry declares `to`, `mt flip` rejects a move that is not a listed edge
and prints the allowed next states. A same-status re-stamp and the first stamp on
a status-less task are exempt; `mt flip --force` overrides. `mt check` flags a
task whose status is unreachable from the initial state under the graph — a
snapshot reachability check, not a full path audit (a `status` carries no
history).

= Status notes

Add `note = "required"` to a status entry to demand a written note on every
transition into it — the same `required` / `optional` / `forbidden` rule typed
`links:` edges already use for a kind's note (the built-in `related` kind
defaults to `required`). Unset, an entry stays `optional`, exactly as before
the field existed:

```toml
[[tasks.status]]
value = "blocked"
note = "required"   # `mt flip blocked task -m "why"` — omit -m and it is refused
```

`mt flip` refuses a move into a `required` entry without `-m/--note`, naming the
status and the flag; `mt check` flags a task already sitting there with a
blank or absent note. The rule is per entry, not legend-wide, on purpose: a
blanket requirement on every transition — including `done` — buys a note an
agent will always fill plausibly, not a justification. Reserve it for the
statuses no other check covers (`blocked`, `failed`, a reopen); leave `done`
to a review gate instead. `forbidden` is enforced for a link kind's note but
not for a status entry's — `mt check` never rejects a status note for being
present.

= Link kinds

`[[tasks.links.kinds]]` layers project-declared edge kinds over the six built-ins
(see #link(u("/dsl/"))[Task DSL]). Each kind sets its cardinality cap (`max`),
note requirement (`note`), whether it gates readiness (`blocking`) or must stay
acyclic (`acyclic`) — and `validate`, the shape its targets take:

```toml
[[tasks.links.kinds]]
name = "discharged-by"
validate = "url"          # the PR or build that closed the task
note = "optional"

[[tasks.links.kinds]]
name = "landed-as"
validate = "git-sha"      # hex, 7–64 chars: abbreviated SHA-1 → full SHA-256

[[tasks.links.kinds]]
name = "ticket"
validate = "regex"        # needs the sibling pattern below
regex = "^ACME-[0-9]+$"
```

#table(
  columns: 2,
  table.header([`validate`], [Accepts]),
  [`local-task-path`],
  [A filename stem resolved against the loaded tasks — the default, and today's behaviour for every kind that omits the key.],

  [`url`], [An RFC 3986 URI: a scheme, then only URI-legal characters.],
  [`git-sha`], [A hexadecimal git object name, 7 to 64 characters.],
  [`uuid`], [A UUID.],
  [`regex`],
  [Whatever the kind's sibling `regex = "..."` matches. The pattern is required — a `regex` kind without one fails config validation rather than accepting everything.],

  [`any`], [Any non-empty string — the deliberate opt-out.],
)

Anything but `local-task-path` declares the kind's targets to be artifacts
outside the backlog. The graph semantics follow from that one field — such an
edge is:

- never resolved to a stem;
- never counted against a cardinality cap;
- never part of a cycle;
- never a readiness blocker, even with `blocking` set (`acyclic` is ignored too).

A malformed target is a `mt check` violation naming the kind, the validator,
and the value. `mt ls` never follows one, and `mt graph` draws it as its own
dashed leaf node rather than a task.

= Stem naming

`[tasks.naming]` templates the filename #link(u("/cli/"))[`mt add`] and
`mt recur` spawns mint for a new task. Unset, a new task gets a bare slug
(`wire-oauth.typ`):

```toml
[tasks.naming]
stem = "{slug}.task-{yy}{mm}{dd}"
```

#table(
  columns: 2,
  table.header([Macro], [Expands to]),
  [`{slug}`], [The title, slugified — the whole template when unset.],
  [`{date}`], [Today's (or the recurrence anchor's) date, ISO `YYYY-MM-DD`.],
  [`{date:FMT}`], [The same date in a custom format token grammar.],
  [`{yy}` / `{mm}` / `{dd}`], [Two-digit year / month / day.],
  [`{n}` / `{n:0N}`],
  [The recurrence occurrence index; bare or zero-padded to `N` digits.],
)

A minted stem never overwrites an existing file: on a collision `mt` appends
`-2`, `-3`, … until a free name turns up.

= Recurring tasks

`[tasks.recur]` is the policy floor a task's `recur:` field (see
#link(u("/dsl/"))[Task DSL]) validates against and `mt recur` (see
#link(u("/cli/"))[CLI reference]) resolves defaults from:

```toml
[tasks.recur]
enabled = true                              # false rejects any authored recur field
units = ["days", "weeks", "months", "years"]
anchors = ["completion", "due"]
default-anchor = "completion"               # used when a task's recur omits anchor
materialize = "spawn"                       # used when a task's recur omits materialize
rrule = false                               # the rrule(...) escape hatch
max-advance = 52                            # runaway cap for `mt recur --advance N`
```

Every field defaults to the value shown, so an unconfigured project already
supports `recur:` out of the box; set `enabled = false` to turn the feature off
project-wide, or narrow `units` / `anchors` to the cadences a project wants to
support.

`rrule = true` alone is not enough: the `rrule("FREQ=...")` escape hatch also
needs `mindtape`/`mt` built with the `rrule` cargo feature (off by default —
it pulls in `chrono-tz` + `regex`, so only a project that actually authors an
`rrule(...)` pays for them). Either gate missing rejects the task with a
precise message; `mt check` additionally grammar-checks the raw rule string
at parse time. See #link(u("/dsl/"))[Task DSL]'s "Recurring tasks" section.

= Work-in-progress limits

`[[tasks.limits]]` caps how many tasks may sit in a matched set at once, and
`mt check` reports a breach. Bounded work-in-progress is what stops a session
ending on an open leaf task.

```toml
[[tasks.limits]]
filter = 'status == "wip" && kind == "umbrella"'
max = 1
severity = "error"

[[tasks.limits]]
filter = 'status == "wip" && kind != "umbrella"'
max = 1
message = "close the open leaf before starting another"
severity = "error"
```

`filter` and `max` are required; `message` replaces the default advice and
`severity` is `"warn"` (the default) or `"error"`. Warning is the default
because a breach is a state of the backlog rather than a malformed artifact —
it clears by finishing work, not by editing a file. `"error"` fails `mt check`,
which is what a project wanting a hard stop (a pre-commit hook, CI) uses.

No status name is built in: the capped set is a
#link(u("/filters/"))[filter expression], so the same primitive caps a status,
a named view (`filter = '@stuck'`), a priority band or a `kind`. A missing
field is unequal to every literal, so the pair above partitions the whole
backlog — a task carrying no `kind` counts against the second limit by
construction.

A breach prints the count, the cap, the advice, and every task in the matched
set:

```
error: 2 task(s) match `status == "wip" && kind != "umbrella"`, max 1 — close the open leaf before starting another
  - rewrite-loader
  - split-config
```

= Review comments

`[tasks.comments]` polices leftover review notes left in a task's whole source
file — a `//` dropped inside `task.with(...)` is caught exactly like one in the
body:

```toml
[tasks.comments]
policy = "forbid"
allow-pattern = "^TODO:"
```

`policy` is `"allow"` (the default, unconstrained), `"warn"` (reported, `mt
check` still succeeds), or `"forbid"` (reported, and `mt check` fails).
Detection runs on the parse plane, so a `//` inside a string literal or a raw
block is never flagged. `allow-pattern` is an optional regex, matched against
a comment's stripped inner text, that exempts a deliberate form while still
catching stray notes.

A violation reports the file, the line, and the comment text:

```
truth/tasks/example.typ:12: comments: line comment: "no, use in_view() instead"
```

= Estimate bounds and working-tree drift

An estimate is an opaque `unit -> quantity` dict, and `units` is the allow-list
of legal unit names. Two per-unit tables bound the quantities:

```toml
[tasks.estimate]
units = ["files"]

[tasks.estimate.max]
files = 5

[tasks.estimate.min]
files = 1
```

Outside the range is a *validation error*, not a warning: an oversized task must
be split before the work starts, not after. Terminal tasks are exempt — a
finished task cannot be split any more.

A file count is the estimate `mt check` can actually verify. `[tasks.estimate.drift]`
compares the git working tree against the budget the in-flight tasks declare and
warns — never fails — when the tree has outgrown it:

```toml
[tasks.estimate.drift]
unit          = "files"              # estimate unit read as the file budget
in-flight     = 'status == "wip"'    # which tasks the diff is attributed to
line-multiple = 200                  # secondary line ceiling per declared file; 0 disables
message       = "rescope before going further"   # replaces the default advice
```

The working tree carries one diff and nothing in it says which file belongs to
which task, so attribution is by count. One task in flight: compare against its
budget and name it. Several: compare against the sum, and say the figure is a
total.

```
warning: working tree touches 14 file(s), over the 2 declared by `alpha` — split the remaining work into a new task and finish the part in hand
warning: working tree touches 16 file(s), over the 6 declared in total by 3 in-flight tasks — split the remaining work into a new task and finish the part in hand
  in flight: alpha, beta, gamma
```

The message is the whole intervention, so the default names the action rather
than restating the breach. The line total is a secondary soft signal at a
generous multiple of the declared file count, never a budget of its own — line
counts are distorted by generated files, lockfiles and formatting passes.

#callout("note")[
  The check is inert until an in-flight task declares an estimate in `unit`, and
  reads the tree strictly read-only (`git --no-optional-locks … status
  --porcelain`). Outside a git repository, or with git unavailable, it says
  nothing at all — it never guesses, and never fails the command.
]

= Flip hooks

`[[on.flip]]` binds actions to a task changing status. One table, three action
kinds, and the project decides what fires where:

```toml
[[on.flip]]
when = 'into("wip")'
actions = [{ list = "@wip", title = "Already in flight:" }]

[[on.flip]]
when = 'into("done") && !has_tag("chore")'
actions = [
  { show = "closing — the checklist is in CONTRIBUTING.md" },
  { run = "just check", gate = true, message = "the suite has to pass first" },
]
```

`when` is a #link(u("/filters/"))[filter expression] over the task *as it stands
before the write* — so `status == "wip"` names the status being left — plus the
`into("name")` predicate for the status being stamped. An action runs only on a
transition its `when` matches. An empty `when` fires on every flip.

Each action declares exactly one kind:

#table(
  columns: 2,
  table.header([Action], [Effect]),
  [`{ show = "text" }`], [Print `text`.],
  [`{ list = "filter or @view", title = "..." }`],
  [Print the matching tasks as a table under `title`. A bare `@view` also picks up that view's sort. This is how an agent starting work sees what else is in flight — with no materialised index and nothing written to disk.],

  [`{ run = "cmd", gate = false, message = "..." }`],
  [Execute `cmd`. With `gate = true` a non-zero exit *refuses the flip*, prints the command's output, and stops: nothing is written for that task and the actions after it do not run. Without `gate` the output is informational. `message` replaces the default advice on failure.],
)

Actions run in declaration order. `--force` waives a failing gate and records
the waiver in the status note (`forced past failing gate: "just check"`), so an
override is still legible afterwards. A gate command runs from the project root
with the transition in its environment:

#table(
  columns: 2,
  table.header([Variable], [Value]),
  [`MINDTAPE_TASK_FILE`], [Absolute path of the task file being flipped.],
  [`MINDTAPE_TASK_STEM`],
  [The task's identity (its file stem, unless `id:` overrides).],

  [`MINDTAPE_TASK_STATUS`], [The status being stamped.],
)

`show` and `list` output is suppressed under `--quiet` and whenever stdout is
not a terminal, so a scripted flip stays clean. Gates still run, and a failing
one still prints its output — that is the evidence the refusal rests on.

#callout("warn")[
  *Trust boundary.* An `[[on.flip]]` command is exactly as trusted as a
  `Justfile` recipe or a git hook: it comes from the repository's own config and
  runs through `sh -c` with your full authority, from the project root. Nothing
  sandboxes it. Read a project's config before running `mt flip` under it, the
  same way you read a `Makefile` before running `make`.
]

Only `mt flip` runs actions. `mt check`, `mt ls` and every other read command
execute nothing — a read that shells out is a surprise, and `mt check` is what
runs inside editors and commit hooks. A malformed hook (an unparseable `when`,
an `into(...)` naming no configured status, an action with no kind) is a config
error at load, so it fails every command equally rather than surprising the
first flip that reaches it.

= Named views

`[views.<name>]` bundles a filter with an optional sort and column set so
`mt ls` can reference it by name instead of repeating the expression:

```toml
[views.broken]
filter = 'blocked && stale(30)'

[views.summer]
filter = 'has_tag("summer") && !terminal'
sort = ["priority:desc", "due:asc"]
columns = ["#", "status", "priority", "title"]
description = "Summer push"
```

`filter` is required; `sort` (a list of `--sort`-grammar specs) and `columns`
(a subset of the `mt ls` column names — `#`, `status`, `ready`, `priority`,
`due`, `title`) are optional, as is a free-text `description`. Reference a
view with `@name`: bare `@name` applies filter, sort, and columns together;
`@name` inside a larger expression is filter-only, equivalent to
`in_view("name")` — see #link(u("/filters/"))[Filter language] for the
predicate and #link(u("/cli/"))[CLI reference] for `mt ls` / `mt views`.

Views are validated at config load:

- each `filter` must parse;
- `sort` and `columns` entries must be legal;
- every view reference must resolve;
- the reference graph must be acyclic — a cycle is a config error.

= Conditional `ls` rules

`[[ls.rules]]` makes bare `mt ls` adapt to ambient project state: the first
entry (top to bottom) whose `condition` holds — or that omits `condition`
entirely — wins.

```toml
[[ls.rules]]
name = "release"                              # optional; shown in the trailer / JSON
condition = 'on_branch("release/*")'          # filter-syntax over env predicates; absent = always
import-views = ["inbox", "release(sort)"]     # compose left-to-right, per-facet last-wins
filter = '!terminal && due <= "2026-08-01"'   # optional manual override (highest precedence)
```

- `name` — shown in the trailer and JSON; falls back to the `condition` text
  when unset.
- `condition` — a #link(u("/cli/"))[condition-language] expression over
  `on_branch`/`env`/`weekday`/`now`; absent matches always.
- `import-views` — ordered `"view"` (all facets) or `"view(facet, ...)"` (a
  subset of `filter`/`sort`/`columns`) entries, composed left to right with
  per-facet last-wins.
- `filter`/`sort`/`columns` — manual overrides, highest precedence; same
  grammar as `[views]`.

Per facet, precedence is manual > the last `import-views` entry that provides
it > the built-in default. A matched rule does not inherit `default-view` as a
base layer — it composes everything it uses. No rule matching falls back to
`default-view` exactly as before bare `mt ls` had rules at all.

Rules are validated at config load, same checks as `[views]`:

- `condition` must parse and reference only environment predicates — a task
  field name is a load-time error;
- each `import-views` entry must resolve to a declared view with legal facet
  selectors;
- a manual override must be valid.

= Editor schema

`mt install` writes a JSON Schema for `.mindtape/config.toml` next to the app
config (`~/.config/mindtape/config.schema.json`), generated from the config
types so it never drifts. `mt init` scaffolds a config whose first line is a
`#:schema` comment directive:

```toml
#:schema https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/config.schema.json
[tasks]
roots = ["tasks"]
```

A schema-aware TOML editor (tombi, Taplo) reads the directive to offer
completion, hover docs, and validation while you edit.

The default is the published URL because `.mindtape/config.toml` is a
checked-in file: an editor fetches and caches the URL, so it needs no local
install and means the same thing to everyone who clones the repo. `mt init
--schema local` points at the installed copy instead (a `file://` URL under
your XDG config dir) for offline use or a project you never share; `--schema
URL|PATH` sets any other target, including a path relative to the config file;
and `--no-schema` (on `init` or `install`) opts out. The directive is a plain
TOML comment, so `mt` itself ignores it.

= Preset overlays

A preset (`mt init --preset`) is composed at init time from the wizard's
configuration axes. Whatever the axes cannot express — custom link kinds, named
views, body-header rules — goes in a top-level `[overlay]` table the preset may
carry:

```toml
[tasks]
roots = ["tasks"]
# ... axis-shaped sections ...

[overlay.views.mine]
filter = "!terminal"

[[overlay.tasks.links.kinds]]
kind = "spike"
```

`[overlay]` is an init-time-only concept: `mt init` deep-merges it over the
composed base *last*, so it beats every axis fragment, then flattens the
result to one `.mindtape/config.toml`. The loader never sees a literal
`[overlay]` — the written file is an ordinary config, yours to edit. In the
merge, tables merge key-by-key and a scalar or array replaces the base value
wholesale, so the overlay always wins the key it names. A preset whose whole
body is under `[overlay]` composes to exactly that overlay.

Arrays replace wholesale, so two ordinary (non-overlay) fragments can't safely
declare the same array-typed section — the status legend (`[[tasks.status]]`),
priority `bands`, difficulty anchors, `[[tasks.roots]]`. Declaring it twice is
a hard error naming both sources: a silent pick would be a status legend
nobody chose. Fix it by picking one fragment, or by moving the difference into
`[overlay]` — exempt from the rule precisely because it exists to override.

= Environment variables

Besides `-w`/`--worktree` (or `$MINDTAPE_WORKTREE`, see #link(u("/agents/"))[For
  AI agents]), `mt` honours `$MINDTAPE_NOW` for a pinnable "today": every date
`mt` stamps or reads — status dates, `now()`/`stale()` in filters, `mt recur`
advances — resolves once per process from `$MINDTAPE_NOW` (`YYYY-MM-DD` or an
RFC 3339 timestamp; only the date part is used), else the system clock (UTC). A
malformed value is a hard error naming the variable — never a silent
fall-through.
