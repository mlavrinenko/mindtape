#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/schema/",
  label: "Canonical schema",
  title: "Canonical schema",
  summary: "The task records `mt ls --json` prints, what `mt import` reads, and every machine output's schema.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt ls --json` prints each task as one JSON record, and `mt import` reads the
same records back and writes `.typ` files, so a program written against this
page works in both directions. `CanonicalTask` in
`crates/mindtape/src/model/canonical.rs` is the Rust type behind it.

= Fields

A field is #strong[authored] when it has a value written in the task file.
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
  [The path from the project root, with `/` between folders on every
    platform, which the `path` filter field also reads. The table's `file`
    column shows the path from the working directory instead, in your
    platform's #link(u("/cli/#paths"))[separator].],

  [`title`], [authored], [The only field a record must have.],
  [`status`], [authored], [`{name, date, note}`, or `null` when unset.],
  [`priority`], [authored], [A number, a label, or a framework object.],
  [`difficulty`],
  [authored],
  [A number or a label. A framework value prints as the framework's name.],

  [`kind`], [authored], [],
  [`assignee`], [authored], [],
  [`estimate`],
  [authored],
  [A map from unit to quantity. A `count(...)` unit contains the number it
    resolved to.],

  [`estimate_counts`],
  [authored],
  [Each `count(...)` unit as `{count, queries, problem}`. Each query
    lists the files it matched as `files`, and `problem` says why the unit
    didn't resolve, or is `null`. `mt import` writes the unit back as the
    `count(...)` from `count` and `queries`, and ignores the rest.],

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

  [`derived`],
  [computed],
  [Each #link(u("/config/#derived-fields"))[collector]'s result, keyed by
    its name. `mt import` ignores it.],

  [`collector_problems`],
  [computed],
  [Why a collector produced no value, one message per failure. `mt check`
    reports each as a `Collector` violation at `derived.<name>`.],
)

A record read by `mt import` needs only `title`. A missing key reads as
`null` or empty, so an adapter can leave out every field it has no value for.

Every authored field survives `mt ls '' --json` followed by `mt import`. `id`
and `file` come back the same too, unless the import renames the file.
#link(u("/import-export/#round-trip"))[Import & export] runs that round trip
and says when a name changes.

= A record

A task from `examples/round-trip/`:

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

The first key is `$schema`, a URL to this object's schema. `mt schema ls`
prints the same bytes, so a program can check the URL it read. `mt import`
accepts input without `$schema`, such as a bare array of records.

Every object output has `$schema` first, with its payload keys flat
beside it. `mt eval --json` wraps one task record under its own `eval` schema;
the `task` schema describes the bare record `mt ls` puts in `tasks`.

`applied` says which #link(u("/named-sets/"))[view] or
#link(u("/cli/ls/#conditional-rules"))[`[[ls.rules]]` entry] set the
listing. `rule` is the label of the matching `[[ls.rules]]` entry, or `null`.
`facets` contains the listing's three #link(u("/cli/ls/"))[facets], `filter`,
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

= Every output

`mt schema` lists each machine output with its version and the URL its
`$schema` names:

#terminal("schema-list")

The URL ends in `/schema/output/<name>/<version>.schema.json`. The version
changes only when a change could break an earlier consumer. A compatible
addition updates the file in place; an incompatible one adds a new version and
leaves the old file, so a stored URL still resolves after an upgrade. `task`
describes one record.
Every command output has its own file. The odd names map to their command:
`preset-ls` is `mt preset ls --json`, `set-ls` is `mt @set ls --json`,
`set-check` is `mt @name check --json`, `at-project` and `at-set` are the two
`mt at <name> --json` shapes, `describe` is `mt add --describe=json`,
`config-show` is `mt config show --json`, `view` is
`mt explain views <name> --json`, and `dict` is
`mt explain dicts <name> --json`. The rest are named after their command: `ls`,
`add`, `check`, `graph`, `explain`, `rules`, `tags`, `eval`, `board`, `at`,
`packs`, `views` and `dicts`.

A released binary names
`https://raw.githubusercontent.com/mlavrinenko/mindtape/main/schema/output/...`,
so the URL always resolves to the newest published copy. A working-tree build
names its checkout through `MINDTAPE_SCHEMA_BASE`, which the dev shell sets to
`file://$mt_root/schema/output`. `mt install` writes the same files under the
config directory, `~/.config/mindtape/schema/output/...` (or
`$MINDTAPE_CONFIG_HOME/mindtape/...`), so an offline editor can use the
installed copy. Each later install refreshes them, so a newer binary's schema
replaces the installed copy without `--force`.

`mt schema --write` records every current schema under `schema/output/`, and
`mt schema --check` fails, naming each missing or changed file. The test suite
runs that check, and the docs gate fails when a doc or a capture names a schema
URL with no file, so a stale link cannot resolve.

= Many projects

A #link(u("/read-many-projects/"))[set] such as `@work` reads every project it
contains. `mt @work ls --json` prints one object whose first key is `$schema`
and whose `projects` key contains one listing object per project, in set
order. Each listing has the keys above, plus `project`, the name `mt at <name>` shows, and
`error`. `error` is `null` when the project listed, and the message when it
failed, in which case `tasks` is `[]` and `applied` is `null`. Here `jq` cuts
each listing down to its task ids:

#terminal("schema-set-ls")

`mt @work board --json` wraps its rows the same way: `$schema`, then a
`projects` array with one object per project and not per task. Here `jq` cuts
it to that array:

#terminal("schema-set-board")

- `project` is the project's name and `path` its directory.
- `hidden` is `true` for a row the board's `hide` filter leaves out of the
  table. The `projects` array still includes it.
- `cells` maps each column name to its value. A `filter` column contains a
  count, and a `top` column contains `{id, title, file}` for the first task it
  ranks, or `null` when no task matches. A column that fails to run contains
  `{error}` in place of either.
- A project whose config won't load has `project`, `path` and `error`, and no
  `cells` or `hidden`.

#callout("tip")[
  #link(u("/import-export/"))[Import & export] has recipes that write and read
  this schema.
]
