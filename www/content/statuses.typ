#import "../utils/page.typ": page
#import "../utils/ui.typ": diagram, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../utils/config-ref.typ": config-keys-table

#let meta = (
  route: "/statuses/",
  label: "Statuses",
  title: "The status legend",
  summary: "The `[[tasks.status]]` legend: legal statuses, the moves between them, required notes and the `unblocked` guard.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Every block below comes from examples/statuses/, the project
// `just check-examples` validates: the legend is a slice of its config, the
// two status lines are slices of its tasks, and each refusal is a real
// `mt flip` run against a copy of it (www/generate-config-runs.sh). The same
// script draws the legend's moves from the fixture's `mt config show --json`.
#let schema = json("../generated/config-schema.json")
#let legend = checked-slices(
  json("../generated/statuses-config.json"),
  "statuses-config",
  ("open", "on-hold", "closed"),
)
#let lines = checked-slices(
  json("../generated/statuses.json"),
  "statuses",
  ("custom-status", "failed-note"),
)

`[[tasks.status]]` lists a project's legal statuses in lifecycle order.
`mt check` reports a task whose status is not in the list. Each value is also
a verb, so `mt wip <task>` runs `mt flip wip <task>`.

= The default legend

A project that declares no `[[tasks.status]]` gets this legend from the
embedded config floor:

#let floor = json("../generated/floor-legend.json")
#table(
  columns: 4,
  table.header([Value], [Glyph], [Colour], [Terminal]),
  ..floor
    .map(e => (
      raw(e.value),
      e.at("glyph", default: ""),
      e.at("color", default: ""),
      if e.at("terminal", default: false) { [yes] } else { [] },
    ))
    .flatten(),
)

A terminal status ends a task. A field required with `!terminal` stops being
required there. `mt rm` with no filter retires every task in a terminal
status. Declaring
`[[tasks.status]]` replaces the whole list, because an array replaces the
lower layer's value (see #link(u("/config/"))[Configuration]). An empty list,
`[tasks] status = []`, removes the legend, and then any status is legal.

= Writing a status

A task's `status:` holds a value, the date it was set, and an optional note.
The prelude has a constructor for common values such as `wip` and `done`,
which takes the note as a trailing block. Any other value is written with
`status(...)`, which takes the note as its `note` argument:

#raw(
  render-slices(lines, (
    ("custom-status", "a value the prelude has no constructor for"),
    ("failed-note", "a prelude constructor with its note"),
  )),
  lang: "typ",
  block: true,
)

`mt flip` writes this line, so a task file rarely needs it typed. The
#link(u("/dsl/"))[Task DSL] covers the rest of the field.

= Entry keys

#config-keys-table(schema, "tasks.status")

The legend below sets `to`, `note` and `unblocked`, which the rest of this page
explains:

#raw(
  render-slices(
    legend,
    (
      ("open", "work starts in wip only once nothing it depends on is open"),
      ("on-hold", "pausing work demands a reason"),
      ("closed", "done reopens to wip; cancelled is a sink"),
    ),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

= Transitions

`to` lists the statuses a task may move to from this one. These are the moves
the whole legend of `examples/statuses/` allows. A terminal status has a
double border:

#diagram("statuses-moves", [The moves `to` allows in `examples/statuses/`])

With no `to` on any entry, a task may move from any status to any other. Once
one entry declares `to`, `mt flip` refuses a move the list does not name, and prints the moves it
allows:

#terminal("statuses-transition")

`--force` overrides the refusal. Re-stamping a task with the status it already
has is always allowed, and so is the first status of a task that has none. An
entry with an empty `to` is a sink, which a task leaves only with `--force`:
`cancelled` above.

`mt check` reports a task whose status cannot be reached from the initial
status, the first entry that is not terminal. It checks the current status
only, because a task file keeps no history of the statuses before it.

= Required notes

`note = "required"` makes `mt flip` refuse a move into that status without
`-m`/`--note`:

#terminal("statuses-note")

`mt check` reports a task already in that status whose note is missing or
blank. Require a note on the statuses no other check explains, such as
`on-hold` and `failed` above. The default is `optional`. `mt` never refuses a
status note for being present, so `forbidden` has no effect on a status
entry.

= Unblocked statuses

`unblocked = true` makes `mt flip` refuse a move into that status while the
task waits on an unfinished target of a `depends-on` edge, or of a project
kind declared `blocking` (see #link(u("/links/"))[Links]):

#terminal("statuses-unblocked")

`--force` overrides it. `mt check` warns about a task already in such a
status while it is blocked. This is a warning, because `--force` puts a task
there on purpose, and reopening a dependency puts every finished dependant
there. Set `unblocked` on the statuses where work starts or ends, like `wip`
and `done`. Leave it off the statuses blocked work must still reach, like
`cancelled`.

On a status that is also terminal, `unblocked` also requires every child to
be finished: every task whose `parent` edge names this one. A child in a
terminal status counts as finished. `mt flip` names each unfinished child,
`--force` overrides, and `mt check` warns about a parent that is already
closed over an unfinished child. A status that is `unblocked` but not
terminal, like `wip`, lets a parent start while its children are open.
