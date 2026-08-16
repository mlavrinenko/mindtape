#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u
#import "../utils/config-ref.typ": config-gate, config-sections-table

#let meta = (
  route: "/config/",
  label: "Configuration",
  title: "Configuration",
  summary: "How .mindtape/config.toml overlays a defaults floor through a four-layer merge, and the sections it can set.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`.mindtape/config.toml` overlays a shipped defaults floor. Omit a section to
inherit the floor's defaults, declare it to override, or set it empty (e.g.
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
Sections whose shape needs more than one line get a deep dive further down.

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
  ("tasks.assignee", "[tasks.assignee]"),
  ("tasks.body.headers", "[tasks.body.headers]"),
  ("tasks.derive-tags", "[tasks.derive-tags]"),
  ("tasks.tags", "[tasks.tags]"),
  ("tasks.limits", "[[tasks.limits]]"),
  ("tasks.comments", "[tasks.comments]"),
  ("tasks.links", "[[tasks.links.kinds]]"),
  ("actors", "[[actors]]"),
  ("roles", "[[roles]]"),
  ("ls", "[ls]"),
  ("rm", "[rm]"),
  ("retire", "[retire]"),
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
  covers `[[ls.rules]]`, the opt-in `file` column, and `[ls] hyperlinks` /
  `hyperlink-target`.
]

= Example

A compact project config:

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

`required` gates `[tasks.priority]`, `[tasks.difficulty]`, `[tasks.estimate]`,
`[tasks.assignee]` and each `[tasks.fields.<name>]`. It takes `true`, `false`,
or a #link(u("/filters/"))[filter expression] — the field is required for exactly
the tasks that expression matches, evaluated per task by `mt check`:

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

= Assignees

`assignee:` names the actor currently working a task — a plain string,
orthogonal to status: a blocked task keeps its assignee, and reassigning is
editing the field. It is not ownership. `[tasks.assignee]` bounds the
vocabulary:

```toml
[tasks.assignee]
allowed  = ["ada", "lin"]
required = "!terminal"
```

`allowed` is optional — absent or empty accepts any string. `required` behaves
like every other core field: `true`, `false`, or a filter expression, as above.
A required assignee is spliced into the add template as a slot backed by
`$MINDTAPE_PRINCIPAL`: under a harness that sets the variable, `mt add` files
silently; unset, the slot prompts interactively.

= Actors and roles

`[[actors]]` and `[[roles]]` form the project's actor registry: who works
tasks, and under which hats. An actor is a person or a language model; roles
are identity-only — a role carries no tooling or prompt payload, any tooling a
role should enable lives in the reader's harness config, keyed by role name by
convention:

```toml
[[roles]]
name = "reviewer"

[[actors]]
name   = "ada"
kind   = "human"
roles  = ["reviewer"]

[[actors]]
name  = "gpt"
kind  = "llm"
roles = ["author"]
```

`kind` is one of `human` / `llm`. `mt check` validates the registry: actor and
role names are unique and non-empty, and an actor may only hold declared
roles. When `[tasks.assignee] allowed` is set, every allowed name must be a
declared actor — the registry is the vocabulary the assignee field points
into, so a free-text misspelling can never slip through config. In filters,
`assignee == "ada"` compares the actor name and `role == "reviewer"` resolves
the assignee's roles from the registry at read time:

```text
mt ls 'role == "reviewer"'        # tasks currently held by a reviewer
mt ls 'assignee == "gpt"'         # tasks assigned to the model
```

= Status transitions

Add a `to` list to a status entry to declare which statuses may follow it — the
legend becomes a state machine instead of a flat enum. It is opt-in: with no `to`
on any entry the legend stays any-to-any.

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
defaults to `required`). Unset, an entry stays `optional`:

```toml
[[tasks.status]]
value = "blocked"
note = "required"   # `mt flip blocked task -m "why"` — omit -m and it is refused
```

`mt flip` refuses a move into a `required` entry without `-m/--note`, naming the
status and the flag; `mt check` flags a task already sitting there with a
blank or absent note. The rule is per entry, not legend-wide: reserve it for
the statuses no other check covers (`blocked`, `failed`, a reopen), and leave
`done` to a review gate. `forbidden` is enforced for a link kind's note but
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

Whatever mints the stem — a bare slug, a rendered `stem` template, or an
explicit `mt add --name` — one policy decides what happens on a collision at
the destination: `[tasks.naming] dedup`.

```toml
[tasks.naming]
dedup = "skip"   # fail | skip | add-suffix
```

#table(
  columns: 2,
  table.header([Value], [Behaviour]),
  [`fail`], [Bail before writing.],
  [`skip`], [Exit 0, write nothing, report the existing path.],
  [`add-suffix`], [Mint `-2`, `-3`, … until a free stem turns up.],
)

Left unset, `dedup`'s default follows `stem`: a bare stem (no `[tasks.naming]
stem` configured) defaults to `fail`, a templated stem to `add-suffix`. Set
`dedup` explicitly — in config or via `--dedup` — and it applies the same
regardless of how the stem was produced.

`mt add --dedup <mode>` overrides the default for one run; `mt add --format
json` reports which outcome fired (`wrote`, `skipped`, `suffixed`) alongside
the path — see #link(u("/cli/"))[`mt add`].

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
supports `recur:`. Set `enabled = false` to turn the feature off project-wide,
or narrow `units` / `anchors` to the cadences a project supports.

`rrule = true` alone is not enough: the `rrule("FREQ=...")` escape hatch also
needs `mindtape`/`mt` built with the `rrule` cargo feature (off by default —
it pulls in `chrono-tz` + `regex`, so only a project that actually authors an
`rrule(...)` pays for them). Either gate missing rejects the task with a
precise message; `mt check` additionally grammar-checks the raw rule string
at parse time. See #link(u("/dsl/"))[Task DSL]'s "Recurring tasks" section.

= Work-in-progress limits

`[[tasks.limits]]` caps how many tasks may sit in a matched set at once, and
`mt check` reports a breach.

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
`severity` is `"warn"` (the default) or `"error"`. A breach is a state of the
backlog rather than a malformed artifact — it clears by finishing work, not by
editing a file — hence the warn default. `"error"` fails `mt check`, for a
project wanting a hard stop (a pre-commit hook, CI).

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
be split before the work starts, not after. Terminal tasks are exempt.

A file count is the estimate `mt check` can actually verify. `[tasks.estimate.drift]`
compares the git working tree against the budget the in-flight tasks declare and
warns — never fails — when the tree has outgrown it:

```toml
[tasks.estimate.drift]
unit          = "files"              # estimate unit read as the file budget
in-flight     = 'status == "wip"'    # which tasks the diff is attributed to
line-multiple = 200                  # secondary line ceiling per declared file; 0 disables
message       = "rescope before going further"   # replaces the default advice
exclude       = ["docs/generated/**", "*.lock"]  # paths the file count skips
```

`exclude` names paths the file count never counts — generated output nobody
hand-edits, so touching it should not spend part of a budget meant for the
source that produced it. Each pattern is matched against the path git reports,
repo-root-relative (the root `mt` runs against, not the task root), with the
same glob syntax as `[tasks.roots] derive-tags`: `*` matches within one path
segment, `**` crosses directory boundaries. It only filters the file count —
the line total stays unfiltered, already documented below as a generous signal
that tolerates exactly this kind of distortion.

The working tree carries one diff and nothing in it says which file belongs to
which task, so attribution is by count — against the one in-flight task's
budget, or against the sum when several are open:

```
warning: working tree touches 14 file(s), over the 2 declared by `alpha` — split the remaining work into a new task and finish the part in hand
warning: working tree touches 16 file(s), over the 6 declared in total by 3 in-flight tasks — split the remaining work into a new task and finish the part in hand
  in flight: alpha, beta, gamma
```

The line total is a secondary soft signal at a generous multiple of the
declared file count, never a budget of its own: line counts are distorted by
generated files, lockfiles and formatting passes.

#callout("note")[
  The check is inert until an in-flight task declares an estimate in `unit`, and
  reads the tree strictly read-only (`git --no-optional-locks … status
  --porcelain`). Outside a git repository, or with git unavailable, it says
  nothing at all — it never guesses, and never fails the command.
]

= Flip hooks

`[[on.flip]]` binds actions to a task changing status. One table, three action
kinds:

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
the waiver in the status note (`forced past failing gate: "just check"`). A
gate command runs from the project root with the transition in its
environment:

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
one still prints its output.

#callout("warn")[
  *Trust boundary.* An `[[on.flip]]` command is exactly as trusted as a
  `Justfile` recipe or a git hook: it comes from the repository's own config and
  runs through the platform shell — `sh -c`, or `cmd /C` on Windows — with your
  full authority, from the project root. Nothing sandboxes it. Read a project's
  config before running `mt flip` under it, the same way you read a `Makefile`
  before running `make`.
]

A `run` command is shell text, not an argv, and the two shells share no syntax,
so a command written for one platform is not portable to the other. A project
targeting both writes a line both shells accept, or calls a script.

Only `mt flip` runs actions. `mt check`, `mt ls` and every other read command
execute nothing. A malformed hook (an unparseable `when`, an `into(...)` naming
no configured status, an action with no kind) is a config error at load, so it
fails every command equally rather than surprising the first flip that reaches
it.

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
`due`, `title`, `file`) are optional, as is a free-text `description`. Reference a
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
`default-view`.

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
checked-in file: a machine-local path would not survive a clone. `mt init
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
`[overlay]` — the written file is an ordinary config. In the
merge, tables merge key-by-key and a scalar or array replaces the base value
wholesale, so the overlay always wins the key it names. A preset whose whole
body is under `[overlay]` composes to exactly that overlay.

Arrays replace wholesale, so two ordinary (non-overlay) fragments can't safely
declare the same array-typed section — the status legend (`[[tasks.status]]`),
priority `bands`, difficulty anchors, `[[tasks.roots]]`. Declaring it twice is
a hard error naming both sources: a silent pick would be a status legend
nobody chose. Fix it by picking one fragment, or by moving the difference into
`[overlay]` — exempt from the rule because it exists to override.

= Environment variables

Besides `-w`/`--worktree` (or `$MINDTAPE_WORKTREE`, see #link(u("/agents/"))[For
  AI agents]), `mt` honours `$MINDTAPE_NOW` for a pinnable "today": every date
`mt` stamps or reads — status dates, `now()`/`stale()` in filters, `mt recur`
advances — resolves once per process from `$MINDTAPE_NOW` (`YYYY-MM-DD` or an
RFC 3339 timestamp; only the date part is used, verbatim — a trailing offset
shifts nothing), else the system clock read as a *local* calendar date (`$TZ`
on Unix, the platform zone on Windows), never UTC. A malformed value is a hard
error naming the variable — never a silent fall-through.

Local, not UTC, because a task filed at 01:00 belongs to that day: stamping UTC
would date it yesterday for everyone east of Greenwich and skew `now()`,
`stale()` and the overdue marker by up to a day. Dates carry no offset once
written — a `YYYY-MM-DD` in a task file is a plain civil date, compared against
whatever "today" is where the command runs.

Two more move where `mt` keeps per-user state. `$MINDTAPE_CONFIG_HOME` replaces
the config directory — the global `defaults.toml` layer, `config.schema.json`,
`presets/`, `worktrees.toml` — and `$MINDTAPE_DATA_HOME` replaces the data
directory the `@local/mindtape` Typst package installs under. Both override the
platform lookup wholesale, for reading and for what `mt install` writes; set but
empty means "no such directory", which drops the global layer rather than
falling back to the machine's own. They exist because the platform lookup reads
the environment on Unix only (`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `HOME`) and
goes through a known-folder API on Windows that no variable can move — so a
sandboxed run or a test that must not see your personal `mt` setup has one knob
that works everywhere.
