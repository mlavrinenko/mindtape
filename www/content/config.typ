#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u
#import "../utils/config-ref.typ": config-gate, config-sections-table
#import "../generated/just.typ" as just

#let meta = (
  route: "/config/",
  label: "Configuration",
  title: "Configuration",
  summary: "The .mindtape/config.toml reference: the four-layer merge, rule packs, the section map, and the keys only config defines.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`.mindtape/config.toml` overlays a shipped defaults floor. Omit a section to
inherit the floor's defaults, declare keys to override those, or set an array
section empty (e.g. `status = []`) to leave that aspect unconstrained.

= The four layers

Config deep-merges in precedence order, lowest first:

+ the embedded floor (shipped with `mt`);
+ XDG global defaults — `~/.config/mindtape/defaults.toml`;
+ every `.mindtape/defaults.toml` walking up from the project root (farthest
  ancestor first, nearest wins);
+ the project's own `.mindtape/config.toml` (highest precedence).

Tables merge key-by-key — a project's `[ls] hyperlinks` leaves the floor's
`[ls.columns]` standing — while scalars and arrays replace wholesale, so a
nearer `[[tasks.status]]` legend drops the farther one. The three rule arrays
are the one append: `[[tasks.rules]]`, `[[tasks.body.headers.rules]]` and
`[[tasks.body.content.rules]]` fold together across layers and packs, a
same-named entry replacing the farther one in place. So a rule an ancestor's
`.mindtape/defaults.toml` declares survives a project adding its own, where
the array-replace rule would have dropped it. Replacement by name works only
ACROSS files; two entries sharing a name WITHIN one layer's (or one pack's)
array is a config error naming both.

Inspect the result:

```sh
mt config show            # effective config as re-parseable TOML
mt config show --origin   # which config layers contributed, and from where
mt config show --defaults # just the embedded floor
mt config show --json     # effective config as JSON
```

= Rule packs

`extends` adopts a rule pack, `disable` drops a named rule outright:

```toml
extends = ["mindtape-tasks", "../shared/team-pack.toml"]
disable = ["imperative-title"]
```

What a pack file may contain, how an entry resolves, the full merge order,
and what the shipped `mindtape-tasks` pack enforces:
#link(u("/rule-packs/"))[Rule packs]. `mt config show --origin` names the pack
behind each rule.

= Sections

Every section below is reflected straight from `mt config schema` — the same
JSON Schema `mt install` writes to disk, generated from the config structs
themselves. A row's purpose column is that property's own doc comment, so a
newly added config key cannot silently go undocumented: the build fails first.

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
  ("tasks.body.content", "[tasks.body.content]"),
  ("tasks.derive-tags", "[tasks.derive-tags]"),
  ("tasks.tags", "[tasks.tags]"),
  ("tasks.limits", "[[tasks.limits]]"),
  ("tasks.rules", "[[tasks.rules]]"),
  ("extends", "extends"),
  ("disable", "disable"),
  ("tasks.comments", "[tasks.comments]"),
  ("tasks.links", "[[tasks.links.kinds]]"),
  ("actors", "[[actors]]"),
  ("roles", "[[roles]]"),
  ("ls", "[ls]"),
  ("rm", "[rm]"),
  ("retire", "[retire]"),
  ("handle", "[handle]"),
  ("views", "[views.<name>]"),
  ("dicts", "[dicts.<name>]"),
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

Sections with a page of their own:

- `[tasks.roots]` / `[tasks.derive-tags]` — #link(u("/derived-tags/"))[Derived tags].
- `[tasks.priority]` / `[tasks.difficulty]` — #link(u("/priority/"))[Priority].
- `[tasks.estimate]` bounds and `drift` — #link(u("/estimates/"))[Estimates].
- `[tasks.recur]` — #link(u("/recurring/"))[Recurring tasks].
- `[views.<name>]` / `[dicts.<name>]` — #link(u("/named-sets/"))[Views & dictionaries].
- `[[on.flip]]` — #link(u("/hooks/"))[Flip hooks].
- `[templates]` / `[add]` — #link(u("/templates/"))[Templates & slots].
- `extends` / `disable` — #link(u("/rule-packs/"))[Rule packs].
- `[ls]` / `[rm]` defaults and every filter expression — #link(u("/filters/"))[Filter language].
- `[[ls.rules]]`, the `file` column, `[ls.columns.<name>] when`, `[ls] hyperlinks` / `hyperlink-target` — #link(u("/cli/"))[CLI reference].

= Body shape: headings and section content

`[tasks.body]` gates the body with two sibling rule families, both selected by
the same `path` through the heading tree. `[tasks.body.headers]` constrains the
heading tree itself — which headings exist, their order, their vocabulary.
`[tasks.body.content]` constrains what is written *under* a heading: a
section's content, everything from the heading down to the next heading of the
same or higher level (nested subsection prose included).

```toml
[[tasks.body.headers.rules]]
name = "has-summary"
path = ["Summary"]
action = "unique"
message = "every task must have exactly one Summary section"

[[tasks.body.content.rules]]
name = "summary-non-empty"
path = ["Summary"]
assert = "non-empty"                # a heading over nothing satisfies nobody
message = "the Summary must not be a placeholder"

[[tasks.body.content.rules]]
path = ["Scope"]
assert = { min-words = 30 }
when = 'kind == "bug"'              # same `when` gating the heading rules have

[[tasks.body.content.rules]]
path = ["AS-IS"]
assert = { max-words = 400 }        # stops padding, not just under-writing

[[tasks.body.content.rules]]
path = ["Watch for"]
assert = { must-match = "\\?" }     # an open-question section states its question

[[tasks.body.content.rules]]
path = [{ match = ".*" }]
assert = { density = { pattern = "—", max = 1, per = "paragraph" } }
message = "more than one em-dash in a paragraph reads as generated prose"
```

A heading path is a list of segments. A bare TOML string is an exact,
case-sensitive, direct-child match; `{ match = "regex", recursive = true }`
matches heading *text* against the regex (recursive = any descendant). With
`when` and `message` on every rule, those two segment shapes are the whole
language — so a section-title vocabulary is an allowlist plus a regex segment:

```toml
[[tasks.body.headers.rules]]
action = "allowlist"
allowed = [{ match = "^(Summary|Scope|Watch for|Decisions)$" }]
```

The six content assertions: `"non-empty"`, `min-words = N`, `max-words = N`,
`must-match = "regex"`, `must-not-match = "regex"`, and `density = { pattern =
"regex", max = N, per = "section" }`. The regexes run against the section's
source text as written (markup included), and words are whitespace-delimited
tokens containing an alphanumeric character — so markup, code blocks, and
nested subsection headings all count toward `min-words` and `max-words`
alike, and a `Scope` section that is all `#raw` reads as non-empty rather
than as a placeholder.

`density` fails when any single unit of the section carries more than `max`
matches of `pattern`, a rate `must-not-match` cannot express since it fails
on the first occurrence. `per` picks the unit: `section` (the default) treats
the whole matched content as one; `paragraph` splits on blank lines, with a
`- `/`+ ` list item opening a new paragraph even without one before it, so a
tight list is one paragraph per item; `sentence` splits each paragraph again
on `.`, `!`, or `?` followed by whitespace or its end. Unlike the word counts
and the other regex asserts, a fenced code block is dropped before any split
runs — a pattern inside a code sample is never counted.

A violation prints at the heading's line, `mt check` style, naming the rule
that fired:

```
truth/tasks/example.typ:11: body.content.rules.summary-non-empty: section `Summary` must not be empty
```

`name` is optional on every `[[tasks.body.headers.rules]]` and
`[[tasks.body.content.rules]]` entry, exactly like `[[tasks.rules]]`'s: it
exists so a reader — or a `disable = ["<name>"]` override — can grep straight
back to the entry that fired. Leave it off and the report names the rule's
position instead, `body.headers.rules[0]`. Two entries in the same array
sharing a name is a config error naming both. Every write command checks both
families before it writes, the same pass that enforces `[[tasks.rules]]`.

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
  A malformed expression fails when the config loads, naming the field — not
  once per task.
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
like every other core field. A required assignee is spliced into the add
template as a slot backed by `$MINDTAPE_PRINCIPAL`: under a harness that sets
the variable, `mt add` files silently; unset, the slot prompts interactively.

= Actors and roles

`[[actors]]` and `[[roles]]` form the project's actor registry: who works
tasks, and under which hats. An actor is a person or a language model; a role
is identity-only — it carries no tooling or prompt payload, and any tooling it
should enable lives in the reader's harness config, keyed by role name by
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

`kind` is `human` or `llm`. `mt check` validates the registry: actor and role
names are unique and non-empty, and an actor may only hold declared roles. When
`[tasks.assignee] allowed` is set, every allowed name must be a declared actor
— the registry is the vocabulary the assignee field points into, so a
free-text misspelling can never slip through config. In filters,
`assignee == "ada"` compares the actor name and `role == "reviewer"` resolves
the assignee's roles from the registry at read time — see
#link(u("/filters/"))[Filter language].

= Status transitions

Add a `to` list to a status entry to declare which statuses may follow it — the
legend becomes a state machine instead of a flat enum. It is opt-in: with no
`to` on any entry the legend stays any-to-any.

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
dashed leaf node rather than a task. Writing edges and the `edge()` helper:
#link(u("/links/"))[Links].

= Stem naming

`[tasks.naming]` templates the filename `mt add` and `mt recur` spawns mint for
a new task. Unset, `mt add` gets a bare slug (`wire-oauth.typ`) and a spawn
gets `{slug}-{date}`:

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
or narrow `units` / `anchors` to the cadences a project supports. The
cadences, the policy that rides along with them, and the two gates `rrule`
opens are on #link(u("/recurring/"))[Recurring tasks].

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

= Config-declared rules

Every constraint above is one MindTape knows by name — a status legend, a
numeric scale, a tag allow-list, a heading path. `[[tasks.rules]]` is the
general form. It constrains no field in particular, because it names the
field itself, in one of two shapes: `assert`, an expression in the
#link(u("/filters/"))[filter language] that must hold for every task the rule
selects, or `check`, a Typst function the project already has (below).

```toml
[[tasks.rules]]
name = "imperative-title"
assert = 'in_dict("verbs", word(title, 1))'
when = '!terminal'
message = "start the title with an imperative verb"
severity = "error"
```

Exactly one of `assert`/`check` is required. `when` gates which tasks are
asked — absent means every task — and `message` replaces the default advice,
which names the `assert` expression itself (a `check` rule ignores `message`:
each string its function returns is already the specific thing to fix).
`severity` is `"warn"` (the default) or `"error"`.

Rules are evaluated inside the validation pass, not in `mt check` alone. Every
write command validates before it writes, so `mt add`, `mt flip` (and every
legend verb over it), `mt link`, `mt import`, `mt recur` and `mt review`
enforce a rule for free: an `"error"` breach refuses the write before anything
lands, leaving nothing to repair. A convention that only fails hours later at
check time is a convention nobody keeps.

A breach reports at the task's `task.with(…)` line, prefixed by its severity:

```
error: tasks/loader-refactoring.typ:3: rules.imperative-title: start the title with an imperative verb
```

`warn` prints that line and the command still succeeds; `error` prints it and
fails. A rule with no `name` reports as `rules[0]` — its index in
`[[tasks.rules]]`, which is what finds it again in the config.

An `assert` expression (and `when`, either shape) is dry-run once at config
load against a probe task, exactly as `required` and `[[tasks.limits]]` are,
so a rule naming an unknown field, view or dictionary is one config error
rather than one violation per artifact. A `check` rule's Typst module is
checked the same way, at load. Shipping a whole array of rules to other
projects is #link(u("/rule-packs/"))[a rule pack].

== Checking a field with a Typst function

`assert` reaches a task's own fields through the filter language, which has no
name for an `x-<ns>` extension field and no way into a nested dictionary. A
`check` rule reaches for a Typst function instead — the project's own code,
where a structure it invented already lives:

```toml
[[tasks.rules]]
name = "goals"
when = "!terminal"
check = { typst = "truth/lib.typ", function = "check-goals", field = "x-goals" }
severity = "error"
```

`typst` is a path to a Typst module, relative to the project root (the
directory holding `.mindtape/`); `function` is the name it exports. `field`
names the value the function receives — `none` when the task omits it, so
requiring a field needs no second rule. Without `field` the function receives
the whole task, in the shape `mt eval --json` prints. A worked end-to-end
example is #link(u("/self-validating-tasks/"))[Self-validating tasks].

The function returns an array of strings: an empty array passes, and each
string is one breach, reported exactly like an `assert` breach —
`rules.goals: <string>`, at the rule's own `severity`. A return that is not an
array is one breach naming what came back, and a panic inside the function
(Typst's own `panic()`) is one breach carrying the panic message rather than
an eval error that aborts the project load.

A value reaches the function as a Typst literal built from the field's
harvested value: a content block arrives as its plain text, an empty
dictionary as `(:)`, a one-element array with its trailing comma, a date as a
real `datetime(year:, month:, day:)` value, and a string escaped.

`typst` must exist and export `function` — checked once at config load, each
a separate config error, the same as a malformed `assert`. The function itself
is never called at load: a function that only errors for some inputs must not
fail config load over a value it was never asked to handle.

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

= Conditional `ls` rules

`[[ls.rules]]` lets `mt ls` pick its listing from ambient project state: the
first entry whose `condition` holds — or that omits `condition` entirely —
wins, and its own overrides beat what its `import-views` composed.

```toml
[[ls.rules]]
name = "release"                              # optional; shown in the trailer / JSON
condition = 'on_branch("release/*")'          # absent = always
import-views = ["inbox", "release(sort)"]     # composed left-to-right, per-facet last-wins
filter = '!terminal && due <= "2026-08-01"'   # manual override (highest precedence)
```

- `name` — shown in the trailer and JSON, and the source label for the rule's
  own overrides; falls back to the `condition` text, then to `rule`.
- `condition` — an environment-predicate expression over `on_branch`/`tty`/
  `env`/`weekday`/`now`/`format`; absent matches always.
- `import-views` — ordered `"view"` (all facets) or `"view(facet, ...)"` (a
  subset of `filter`/`sort`/`columns`) entries, composed left to right with
  per-facet last-wins.
- `filter`/`sort`/`columns` — manual overrides, highest precedence; same
  grammar as `[views]`.

A matched rule does not inherit `default-view` as a base layer — it composes
everything it uses. Rules are validated at config load: `condition` must parse
and reference only environment predicates — a task field name is an error —
every `import-views` entry must resolve to a declared view with legal facet
selectors, and a manual override must be valid. The condition language and its
predicates, the built-in column `when` expressions, and how the command line
stacks on top per facet are at #link(u("/cli/"))[the CLI reference].

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
`[overlay]` — the written file is an ordinary config. In the merge, tables
merge key-by-key and a scalar or array replaces the base value wholesale, so
the overlay always wins the key it names. A preset whose whole body is under
`[overlay]` composes to exactly that overlay.

Arrays replace wholesale, so two ordinary (non-overlay) fragments can't safely
declare the same array-typed section — the status legend (`[[tasks.status]]`),
priority `bands`, difficulty anchors, `[[tasks.roots]]`. Declaring it twice is
a hard error naming both sources: a silent pick would be a status legend
nobody chose. Fix it by picking one fragment, or by moving the difference into
`[overlay]` — exempt from the rule because it exists to override.

= Local Typst packages

A task file imports the DSL as `@local/mindtape:0.2.0`, and Typst resolves a
`@local` package from one machine-global directory —
`<data dir>/typst/packages/local/<name>/<version>/`, which `mt install` fills.
One directory, every checkout: a second clone or a git worktree reads whichever
tree installed last, not the `lib/` beside it. Nothing says so, and it stays
invisible until the two preludes differ — at which point a project is validated
against a DSL it does not contain.

`[[typst.local-packages]]` pins resolution to the tree instead:

```toml
[[typst.local-packages]]
name = "mindtape"
path = "../../lib"
```

`path` names the directory holding that package's `typst.toml`, resolved
relative to the project root and free to leave it. An entry wins over both the
machine-global directory and the automatic registration below, for the name it
declares. The seam is not MindTape-specific: any `@local/<name>` a task file or
an `x-<ns>` helper imports can be pointed at an in-tree path this way.

One case needs no entry. A project whose own `<root>/lib` holds a `typst.toml`
naming `mindtape` gets that directory registered automatically — matched on the
manifest's declared name, not on the directory merely existing, so a `lib/`
that is some other package is left alone. This repo resolves its own prelude
that way; the fixtures under `examples/` each carry an explicit entry, since
nothing named `lib` sits beside them.

`mt install` is unaffected either way. It still writes the machine-global
package, which an editor's Typst language server, `typst compile`, and every
project without an entry all read. What an entry buys is a checkout that
validates on a machine which never ran it — MindTape's own #just.check() passes
with no global install, which is how CI runs it.

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

"For reading" is the half that has to be built rather than inherited. Typst
resolves `@local` through that same platform lookup, one level below `mt`, so
`$MINDTAPE_DATA_HOME` moved where `mt install` wrote the package while every
task file kept importing the machine's own — an install and a load naming
different directories. `mt` now hands the resolver the directory it settled on,
so the two agree; set the variable empty and no global package resolves at all,
which is how #just.check-examples() proves each example fixture reads the
prelude beside it rather than the one this machine happens to have.
