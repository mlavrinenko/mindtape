#import "../utils/page.typ": page
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../utils/config-ref.typ": config-gate, config-sections-table
#import "../generated/just.typ" as just

#let meta = (
  route: "/config/",
  label: "Configuration",
  title: "Configuration",
  summary: "Where `mt` reads its config, how the four layers merge, and which page explains each config section.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Both TOML blocks are slices of checked fixtures, examples/statuses/ and
// examples/rules/, which `just check-examples` validates.
#let slices = checked-slices(
  json("../generated/config-config.json"),
  "config-config",
  ("local-packages", "actors"),
)

A project's config lives in `.mindtape/config.toml` at its root. Every key is
optional. Any key you leave out takes its value from the floor, the defaults
built into `mt`. To see what `mt` ends up with, `mt config show` prints the
merged config as TOML, and `--origin` lists the files it came from (see
#link(u("/cli/config/"))[`mt config`]).

= Layers

`mt` merges four layers, from lowest precedence to highest:

+ the floor built into `mt`;
+ `mindtape/defaults.toml` in your config directory, such as
  `~/.config/mindtape/defaults.toml` on Linux;
+ each `.mindtape/defaults.toml` from the project root up to your home
  directory, the farthest first, so the nearest one wins;
+ the project's `.mindtape/config.toml`.

Tables merge key by key, so if you set `[ls] hyperlinks`, you keep the
floor's `[ls.columns]`. A scalar or an array replaces the lower layer's value
whole, so your `[[tasks.status]]` replaces the floor's
#link(u("/statuses/"))[status legend]. An empty array removes the floor's
value, so `[tasks] status = []` leaves the status field unconstrained.

The four rule arrays work differently. `[[tasks.rules]]`,
`[[tasks.body.headers.rules]]`, `[[tasks.body.content.rules]]` and
`[[commits.rules]]` collect the entries of every layer and of every
#link(u("/packs/"))[rule pack] a layer adopts. An entry replaces a lower
layer's entry with the same `name`, in its place, so you can change one
inherited rule and keep the rest. Two entries with the same `name` in one
layer's array are a config error that names both. `extends` and `disable`,
which adopt a rule pack and drop a rule, are on
#link(u("/packs/"))[Rule packs].

Two keys name a program for `mt` to run: `[go] editor` and `[add] editor`.
They wait for #link(u("/cli/allow/"))[`mt allow`] like a checker or a flip
`run` does, so a clone can't start a program before you have read it. Until
then `mt` opens `$VISUAL` instead and warns once with the held command. An
editor in your own `mindtape/defaults.toml` needs no approval. An ancestor's
`.mindtape/defaults.toml` does, since it can live inside a clone.

Write a rule's `message` as the next step: an imperative in lower case, with no
trailing period or em dash, such as `move Environment after Problem`. `mt check`
prints it after the rule's name, so it should tell the author what to do, not
what is wrong.

= Environment variables

#table(
  columns: 2,
  table.header([Variable], [Effect]),
  [`MINDTAPE_CONFIG_HOME`],
  [Replaces the config directory: where `mt` reads the second layer and where `mt install` writes `config.schema.json` and `at.schema.json`. You write `presets/` and `at.toml` there yourself, and `mt` reads them. #link(u("/read-many-projects/"))[Read many projects] covers `at.toml`, and #link(u("/cli/board/"))[`mt board`] lists its board keys.],

  [`MINDTAPE_DATA_HOME`],
  [Replaces the data directory. `mt install` writes the `@local/mindtape` package under its `typst/packages`, and task files resolve it from there.],

  [`TYPST_PACKAGE_PATH`],
  [Moves the package directory Typst reads, for `mt` too, unless `MINDTAPE_DATA_HOME` is set. See Local Typst packages below.],

  [`MINDTAPE_NOW`],
  [Sets today's date, which `mt` uses for status dates, for `now()` and `stale()` in filters, and in `mt recur`. Takes `YYYY-MM-DD` or an RFC 3339 timestamp, whose date part is used as written.],

  [`MINDTAPE_WORKTREE`],
  [Selects the project, like `-w`. See #link(u("/agents/"))[For AI agents].],

  [`MINDTAPE_PRINCIPAL`],
  [Fills a required assignee in `mt add`. See Actors below.],
)

The `MINDTAPE_` directory variables work on every platform. The platform's
own lookup reads `XDG_CONFIG_HOME` and `XDG_DATA_HOME` on Linux only, so for a
test or a sandbox that must not see your own setup, set these instead. Set but
empty, each one means there's no such directory. `mt` then reads no global
layer, or resolves no global package. #just.check-examples() runs that way, so
each example project reads the prelude beside it.

With `MINDTAPE_NOW` unset or empty, today is the system clock's local date,
using `TZ` on Unix and the system time zone on Windows. A local date keeps a
task filed at 01:00 on the day it was filed. A date in a task file carries no
time zone. A `MINDTAPE_NOW` that isn't a date is an error naming the
variable.

= Local Typst packages

A task file imports the DSL as `@local/mindtape`, and Typst resolves a
`@local` package from one directory per machine. `mt install` fills that
directory, and `mt`, `typst` and tinymist all read it.

#table(
  columns: 2,
  table.header([Platform], [Directory]),
  [Linux],
  [`$XDG_DATA_HOME/typst/packages`, else `~/.local/share/typst/packages`],

  [macOS], [`~/Library/Application Support/typst/packages`],
  [Windows], [`%APPDATA%\typst\packages`],
)

A set `TYPST_PACKAGE_PATH` replaces it on every platform, for all three
tools. `MINDTAPE_DATA_HOME` replaces it for `mt` alone, which suits a test
but hides the package from your editor.

So every checkout on that machine reads whichever `lib/` was installed last.
To pin a project to its own copy, add a `[[typst.local-packages]]` entry. It
resolves the package from a path in the project instead:

#raw(
  render-slices(
    slices,
    (("local-packages", "the prelude two directories up"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

`path` is relative to the project root and may point outside it. An entry
wins over the machine directory, and it works for any `@local/<name>` a task
file or an `x-<ns>` helper imports. If the project's `lib/typst.toml` declares
the package `mindtape`, `mt` uses that directory with no entry. That's how
this repository resolves its own prelude.

`mt install` still writes the machine directory. Your editor's Typst language
server, `typst compile` and every project without an entry read it.

= Editor schema

`mt install` writes a JSON Schema for `.mindtape/config.toml` into the config
directory. `mt init` starts the config with a `#:schema` line that points at
the published copy, so an editor with tombi or Taplo completes keys and checks
values as you type. `mt` reads that line as a TOML comment.
#link(u("/cli/init/"))[`mt init`]'s `--schema` and `--no-schema` flags change
it.

= Presets

`mt init --preset <name>` writes a preset as the project's config. A preset
may carry an `[overlay]` table, which `mt init` merges over the rest of the
preset. Tables merge key by key, and any other value in the overlay replaces
the preset's. You get one file, with no comments and no `[overlay]`.

= Sections

Here's every top-level section, with the description `mt config schema`
prints for it. The docs build fails if the schema has a section the table
leaves out.

// config-sections orders the rows and labels each in TOML syntax; the
// descriptions come from the schema. config-gate panics when a schema
// property is missing here or a row names no property.
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
  ("tasks.rules", "[[tasks.rules]]"),
  ("tasks.waits", "[[tasks.waits]]"),
  ("commits", "[commits]"),
  ("extends", "extends"),
  ("disable", "disable"),
  ("tasks.comments", "[tasks.comments]"),
  ("tasks.links", "[[tasks.links.kinds]]"),
  ("actors", "[[actors]]"),
  ("roles", "[[roles]]"),
  ("ls", "[ls]"),
  ("rm", "[rm]"),
  ("retire", "[retire]"),
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

These sections have a page of their own:

- `[[tasks.status]]`: #link(u("/statuses/"))[Statuses].
- `[[tasks.rules]]`, `[tasks.body.headers]`, `[tasks.body.content]`,
  `[tasks.comments]` and every `required` key: #link(u("/rules/"))[Rules].
- `[[tasks.links.kinds]]`: #link(u("/links/"))[Task links].
- `[tasks] roots` and `[tasks.derive-tags]`:
  #link(u("/derived-tags/"))[Derived tags].
- `[tasks.priority]` and `[tasks.difficulty]`:
  #link(u("/priority/"))[Priority].
- `[tasks.estimate]` and `[tasks.estimate.queries]`:
  #link(u("/estimates/"))[Estimates].
- `[tasks.recur]`: #link(u("/recurring/"))[Recurring tasks].
- `[views.<name>]` and `[dicts.<name>]`:
  #link(u("/named-sets/"))[Views & dictionaries].
- `[[on.flip]]`: #link(u("/hooks/"))[Flip hooks].
- `[[tasks.waits]]`: #link(u("/wait-on-upstream/"))[Wait on upstream].
- `[templates]` and `[add]`: #link(u("/templates/"))[Templates & slots].
- `extends` and `disable`: #link(u("/packs/"))[Rule packs].
- `[commits]`: #link(u("/commit-rules/"))[Commit rules].
- `[ls]` and `[rm]` filters: #link(u("/filters/"))[Filter language].
- `[[ls.rules]]`, `[ls.columns.<name>]` and `[ls] hyperlinks`:
  #link(u("/cli/ls/"))[`mt ls`].

The sections below have no other page.

= Actors

`[[actors]]` lists who works on tasks, people and language models, and
`[[roles]]` lists the roles they can hold. A role is only a name. If a harness
gives a role tools or instructions, it keeps them in its own config, keyed by
the role's name.

#raw(
  render-slices(
    slices,
    (("actors", "one reviewer, one model"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

`kind` is `human` or `llm`. Actor names and role names must be unique and
not empty, and an actor may hold only declared roles. A task's `assignee:`
names the actor working on it. If `[tasks.assignee] allowed` is set, each name
in it must be a declared actor. In a filter, `assignee == "ada"` compares the
name, and `role == "reviewer"` reads the assignee's roles from this list (see
#link(u("/filters/"))[Filter language]).

When an assignee is required, `mt add` asks for it in a
#link(u("/templates/"))[slot] that `MINDTAPE_PRINCIPAL` fills, so a harness
that sets the variable files tasks without a prompt. With the variable unset,
an interactive `mt add` prompts, and a non-interactive one fails naming the
slot.

= Task file names

`[tasks.naming] stem` is the pattern `mt add` and `mt recur` use to name a
new task file, such as `stem = "{slug}.task-{yy}{mm}{dd}"`. If it's unset,
`mt add` names the file `{slug}` and `mt recur` names a spawned occurrence
`{slug}-{date}`.

#table(
  columns: 2,
  table.header([Macro], [Expands to]),
  [`{slug}`], [The title, slugified.],
  [`{date}`],
  [Today's date, or the occurrence's date for `mt recur`, as `YYYY-MM-DD`.],

  [`{date:FMT}`],
  [The same date in the format `FMT`, built from the tokens below. Other characters pass through, so `{date:YYYY}` expands to `2026` and `{date:%Y}` to `%Y`.],

  [`{yy}`, `{mm}`, `{dd}`], [Two-digit year, month and day.],
  [`{n}`, `{n:0N}`],
  [The occurrence number, bare or zero-padded to `N` digits.],
)

A `{date:FMT}` format reads these tokens, longest first:

#table(
  columns: 2,
  table.header([Token], [Expands to]),
  ..json("../generated/date-tokens.json")
    .tokens
    .map(t => (
      raw(t.token),
      [#upper(t.expands_to.first())#t.expands_to.slice(1).],
    ))
    .flatten(),
)

`[tasks.naming] dedup` decides what `mt add` does if the file already exists.
`fail` stops before writing. `skip` writes nothing, exits 0 and reports the
existing path. `add-suffix` appends `-2`, `-3` and so on until the name is
free. If `dedup` is unset, it's `fail` when `stem` is unset and `add-suffix`
when it's set. `mt add --dedup` overrides it for one run.

= Extension fields

`mt` checks an `x-<ns>` field only if you declare it in
`[tasks.fields.x-<ns>]`. There, `values` and `required` work the same as they
do for `kind`. A `values-when` filter limits the check to the
tasks it matches, such as `"!terminal"` to leave closed tasks alone. A declared
field takes one string, or a list of strings with `multiple = true`, and
`values` covers every element. `mt add` offers a required one as a
#link(u("/templates/"))[slot]. A #link(u("/filters/"))[filter] can read the
value whether you declared the field or not.
