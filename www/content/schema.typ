#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/schema/",
  label: "Canonical schema",
  title: "Canonical schema",
  summary: "The task JSON that `mt ls --json` prints and `mt import` reads, for `jq` programs to code against.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Writing a script or a `jq` program against your tasks? `mt ls --json` prints
each task as one JSON record, and `mt import` reads the same records back and
writes `.typ` files. So a program written against this page works in both
directions. `CanonicalTask` in `crates/mindtape/src/model/canonical.rs` is the
Rust type behind it.

= Fields

A field is #strong[authored] when it holds a value written in the task file.
`mt import` writes each authored field back into the file it creates. A
#strong[computed] field is derived from the file or the project, and
`mt import` ignores it.

#table(
  columns: (auto, auto, 1fr),
  table.header([Field], [Kind], [Notes]),
  [`id`],
  [computed],
  [The file #link(u("/concepts/#identity"))[stem], unless the task sets `id:`. `mt import` names the file as #link(u("/import-export/#round-trip"))[Import & export] says.],

  [`file`],
  [computed],
  [The path from the project root, which the `path` filter field also reads.
    The table's `file` column shows the path from the working directory
    instead.],

  [`title`], [authored], [The only field a record must carry.],
  [`status`], [authored], [`{name, date, note}`, or `null` when unset.],
  [`priority`], [authored], [A number, a label, or a framework object.],
  [`difficulty`],
  [authored],
  [A number or a label. A framework value prints as the framework's name.],

  [`kind`], [authored], [],
  [`assignee`], [authored], [],
  [`estimate`], [authored], [A map from unit to quantity.],
  [`links`],
  [authored],
  [`[{task, kind, note}]`. `task` is the target's file name as the task
    wrote it, and `note` is `null` when the edge has no note.],

  [`waits_on`],
  [authored],
  [The `checker:id` entries the task waits on outside the repo.],

  [`tags`], [authored], [The tags written in the file.],
  [`derived_tags`], [computed], [The tags the task's folders give it.],

  [`why`],
  [computed],
  [`[{stem, status, kind}]`: the unfinished tasks a blocked task waits on.
    A `waits-on` entry the checker hasn't cleared shows up as
    `{stem: entry, status: the checker's line, kind: "waits-on"}`. Empty
    unless the task is blocked, and always empty from `mt eval`, which reads
    one file.],

  [`children`],
  [computed],
  [How many tasks name this one in a `parent` link. `0` from `mt eval`.],

  [`progress`],
  [computed],
  [Finished children over all children, from 0 to 1. `null` with no
    children.],

  [`due`], [authored], [`YYYY-MM-DD`.],
  [`start`], [authored], [`YYYY-MM-DD`.],
  [`reviewed`], [authored], [`YYYY-MM-DD`.],
  [`body`],
  [authored],
  [The Typst source after `#show: task.with(...)`, which `mt import` writes
    back unchanged.],

  [`recur`],
  [authored],
  [`{rule, anchor, until, count, materialize}`, or `null` for a task that
    doesn't #link(u("/recurring/"))[recur].],

  [`extras`], [authored], [Each `x-<ns>` field, keyed by its name.],
)

A record read by `mt import` needs only `title`. A missing key reads as
`null` or empty, so an adapter can leave out every field it has no value for.

Every authored field survives `mt ls '' --json` followed by `mt import`. `id`
and `file` come back the same too, unless the import renames the file.
#link(u("/import-export/#round-trip"))[Import & export] runs that round trip
and says when a name changes.

= A record

Here's one task from `examples/round-trip/`:

#terminal("schema-record")

Every key is present whether the task sets it or not, so your program can test
the value and skip checking whether the key exists.

= Priority

`priority` prints the value as the task wrote it, so `mt import` can write
the same call back. A number or a label prints bare. A framework call such as
`ice(...)` prints as an object of the framework's name and inputs, plus the
`score` the project's config computes from them. `score` is `null` when the
config can't compute it, and `mt import` ignores it. `examples/dsl/` has one
of each:

#terminal("schema-priority")

= The listing

`mt ls --json` wraps the records in one object. Here `jq` replaces the records
with their count:

#terminal("schema-envelope")

`schema_version` changes only when the record or this object changes in a
way that could break `mt import` or a `jq` program. `mt import` accepts input
without it, such as a bare array of records. It refuses a version newer than
its own and names both.

`applied` says which #link(u("/named-sets/"))[view] or
#link(u("/cli/ls/#conditional-rules"))[`[[ls.rules]]` entry] shaped the
listing. `rule` is the label of the matching `[[ls.rules]]` entry, or `null`.
`facets` holds the listing's three #link(u("/cli/ls/"))[facets], `filter`,
`sort` and `columns`, each as `{from, value}`. `value` is what the listing
used, and `from` says where it came from: a view, a rule label, `manual` for a
value given on the command line, or `match-all` for the built-in default.
`superseded` is `true` when the matching rule set at least one of them and the
command line overrode them all.

`truncated` is `null` when the listing shows every match. When `-n` or
`--skip` cut it, `truncated` is `{shown, total, next_skip}`, and
`mt ls --skip <next_skip>` with the same arguments shows the next page.

`retired` lists the stems that a #link(u("/filters/"))[`touches()`] call
found in commit trailers and that no task has, such as tasks `mt rm` removed.
It's `[]` when no filter calls `touches()`.

#callout("tip")[
  #link(u("/import-export/"))[Import & export] has recipes that write and read
  this schema.
]
