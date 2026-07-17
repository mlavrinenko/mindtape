#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, callout
#import "../utils/site.typ": u

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

#table(
  columns: 2,
  table.header([Section], [Purpose]),
  [`[tasks] roots`], [Directories holding tasks. Table-form `[[tasks.roots]]` adds a display name and folder-tag policy — see #link(u("/derived-tags/"))[Derived tags].],
  [`[[tasks.status]]`], [The status legend, in lifecycle order: `value`, optional `glyph` / `color`, `terminal`, and optional `to` transition edges.],
  [`[tasks.priority]`, `[tasks.difficulty]`], [Numeric scales, bands, and frameworks. See #link(u("/priority/"))[Priority].],
  [`[tasks.estimate]`], [Allowed estimate `units` (free labels, never converted).],
  [`[tasks.recur]`], [Recurring-tasks policy floor: `enabled`, allowed `units` / `anchors`, `default-anchor`, `materialize`, the `rrule` feature flag, `max-advance`. See below.],
  [`[tasks.tags]`], [`allowed` — the tag allow-list, checked against authored and derived tags.],
  [`[ls]`, `[rm]`], [Default filters, columns, and ordering. See #link(u("/filters/"))[Filter language].],
  [`[views.<name>]`], [Named filter + sort + columns bundles, referenced with `@name`. See #link(u("/filters/"))[Filter language].],
  [`[[ls.rules]]`], [Ordered auto-activation rules for bare `mt ls`. See below and #link(u("/cli/"))[CLI reference].],
  [`[templates]`], [Template `dirs` and the `default` template. See #link(u("/templates/"))[Templates & slots].],
  [`[add]`, `[go]`], [Editor, interactive mode, and header for `mt add` / `mt go`.],
  [`[format]`], [`enabled` (default `true`): run typstyle over the Typst `mt` writes, so surgical edits and slot fills land in one canonical shape. Set `false` for raw byte-splice writes (minimal diffs).],
)

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
required = "non-terminal"

[tasks.estimate]
units = ["minutes", "hours", "days", "weeks"]

# The default `mt ls` listing: a view, so it can carry sort and columns too.
# `[ls] default-view = "name"` points it elsewhere; `default` is the fallback.
[views.default]
filter = "status != \"done\""
```

#terminal(read("../generated/config-origin.txt"))

#callout("note")[
  `mt init` writes a slim scaffold that inherits the floor; `mt init --full`
  writes every section, spelled out and commented.
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

Views are validated at config load: each `filter` must parse, `sort` and
`columns` entries must be legal, every view reference must resolve, and the
reference graph must be acyclic — a cycle is a config error.

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

Rules are validated at config load: `condition` must parse and reference only
environment predicates (a task field name is a load-time error), each
`import-views` entry must resolve to a declared view with legal facet
selectors, and a manual override must be valid — same checks as `[views]`.

= Editor schema

`mt install` writes a JSON Schema for `.mindtape/config.toml` next to the app
config (`~/.config/mindtape/config.schema.json`), generated from the config
types so it never drifts. `mt init` scaffolds a config whose first line is a
`#:schema` comment directive pointing at it:

```toml
#:schema file:///home/you/.config/mindtape/config.schema.json
[tasks]
roots = ["tasks"]
```

A schema-aware TOML editor (tombi, Taplo) reads the directive to offer
completion, hover docs, and validation while you edit. `mt init --schema
URL|PATH` sets a different target — e.g. the published URL for a portable,
checked-in config, or a path relative to the config file — and `--no-schema`
(on `init` or `install`) opts out. The directive is a plain TOML comment, so
`mt` itself ignores it.

= Environment variables

Besides `-w`/`--worktree` (or `$MINDTAPE_WORKTREE`, see #link(u("/agents/"))[For
AI agents]), `mt` honours `$MINDTAPE_NOW` for a pinnable "today": every date
`mt` stamps or reads — status dates, `now()`/`stale()` in filters, `mt recur`
advances — resolves once per process from `$MINDTAPE_NOW` (`YYYY-MM-DD` or an
RFC 3339 timestamp; only the date part is used), else the system clock (UTC). A
malformed value is a hard error naming the variable — never a silent
fall-through.
