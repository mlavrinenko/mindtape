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

A project keeps its config in `.mindtape/config.toml` at its root. Every key
is optional, and a key the project leaves out takes its value from the config
floor built into `mt`. `mt config show` prints the merged config as TOML, and
`--origin` lists the files it came from (see #link(u("/cli/"))[`mt config`]).

= Layers

`mt` merges four layers, from lowest precedence to highest:

+ the floor built into `mt`;
+ `mindtape/defaults.toml` in your config directory, such as
  `~/.config/mindtape/defaults.toml` on Linux;
+ each `.mindtape/defaults.toml` from the project root up to your home
  directory, the farthest first, so the nearest one wins;
+ the project's `.mindtape/config.toml`.

Tables merge key by key, so a project that sets `[ls] hyperlinks` keeps the
floor's `[ls.columns]`. A scalar or an array replaces the lower layer's value
whole, so a project's `[[tasks.status]]` replaces the floor's legend. An empty
array removes the floor's value, so `[tasks] status = []` leaves the status
field unconstrained.

The four rule arrays are the exception. `[[tasks.rules]]`,
`[[tasks.body.headers.rules]]`, `[[tasks.body.content.rules]]` and
`[[commits.rules]]` collect the entries of every layer and of every rule pack
a layer adopts. An entry replaces a lower layer's entry with the same `name`
in its place, so a project can change one inherited rule and keep the rest.
Two entries with the same `name` in one layer's array are a config error that
names both. `extends` and `disable`, which adopt a rule pack and drop a rule,
are on #link(u("/rule-packs/"))[Rule packs].

= Environment variables

#table(
  columns: 2,
  table.header([Variable], [Effect]),
  [`MINDTAPE_CONFIG_HOME`],
  [Replaces the config directory: where `mt` reads the second layer, and where `mt install` writes `config.schema.json`, `presets/` and `worktrees.toml`.],

  [`MINDTAPE_DATA_HOME`],
  [Replaces the data directory: where `mt install` writes the `@local/mindtape` package, and where task files resolve it from.],

  [`MINDTAPE_NOW`],
  [Sets today's date, which `mt` uses for status dates, for `now()` and `stale()` in filters, and in `mt recur`. Takes `YYYY-MM-DD` or an RFC 3339 timestamp, whose date part is used as written.],

  [`MINDTAPE_WORKTREE`],
  [Selects the project, like `-w`; see #link(u("/agents/"))[For AI agents].],

  [`MINDTAPE_PRINCIPAL`],
  [Fills a required assignee in `mt add`; see Actors below.],
)

The two directory variables work on every platform. The platform's own lookup
reads `XDG_CONFIG_HOME` and `XDG_DATA_HOME` on Linux only, so a test or a
sandbox that must not see your own setup sets these instead. Set but empty,
each one means there is no such directory. `mt` then reads no global layer, or
resolves no global package. #just.check-examples() runs that way, so each
example project reads the prelude beside it.

With `MINDTAPE_NOW` unset or empty, today is the system clock's local date,
using `TZ` on Unix and the system time zone on Windows. A local date keeps a
task filed at 01:00 on the day it was filed. A date in a task file carries no
time zone. A `MINDTAPE_NOW` that is not a date is an error naming the
variable.

= Local Typst packages

A task file imports the DSL as `@local/mindtape`, and Typst resolves a
`@local` package from one directory per machine, the one `mt install` fills.
Every checkout on that machine then reads whichever `lib/` was installed last.
A `[[typst.local-packages]]` entry resolves the package from a path in the
project instead:

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
file or an `x-<ns>` helper imports. A project whose `lib/typst.toml` declares the package
`mindtape` gets that directory with no entry, which is how this repository
resolves its own prelude.

`mt install` still writes the machine directory. Your editor's Typst language
server, `typst compile` and every project without an entry read it.

= Editor schema

`mt install` writes a JSON Schema for `.mindtape/config.toml` into the config
directory. `mt init` starts the config with a `#:schema` line that points at
the published copy, so an editor with tombi or Taplo completes keys and checks
values as you type. `mt` reads that line as a TOML comment.
#link(u("/cli/"))[`mt init`]'s `--schema` and `--no-schema` flags change it.

= Presets

`mt init --preset <name>` writes a preset as the project's config. A preset
may carry an `[overlay]` table, which `mt init` merges over the rest of the
preset. Tables merge key by key, and any other value in the overlay replaces
the preset's. The written config is one file without comments and without
`[overlay]`.

= Sections

The table lists every top-level section with the description
`mt config schema` prints for it. The docs build fails when the schema has a
section the table leaves out.

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

These sections have a page of their own:

- `[[tasks.status]]`: #link(u("/statuses/"))[Statuses].
- `[[tasks.rules]]`, `[tasks.body.headers]`, `[tasks.body.content]`,
  `[tasks.comments]` and every `required` key: #link(u("/rules/"))[Rules].
- `[[tasks.links.kinds]]`: #link(u("/links/"))[Links].
- `[tasks] roots` and `[tasks.derive-tags]`:
  #link(u("/derived-tags/"))[Derived tags].
- `[tasks.priority]` and `[tasks.difficulty]`:
  #link(u("/priority/"))[Priority].
- `[tasks.estimate]`: #link(u("/estimates/"))[Estimates].
- `[tasks.recur]`: #link(u("/recurring/"))[Recurring tasks].
- `[views.<name>]` and `[dicts.<name>]`:
  #link(u("/named-sets/"))[Views & dictionaries].
- `[[on.flip]]`: #link(u("/hooks/"))[Flip hooks].
- `[templates]` and `[add]`: #link(u("/templates/"))[Templates & slots].
- `extends` and `disable`: #link(u("/rule-packs/"))[Rule packs].
- `[commits]`: #link(u("/commit-rules/"))[Commit rules].
- `[ls]` and `[rm]` filters: #link(u("/filters/"))[Filter language].
- `[[ls.rules]]`, `[ls.columns.<name>]` and `[ls] hyperlinks`:
  #link(u("/cli/"))[CLI reference].

The two sections below have no other page.

= Actors

`[[actors]]` lists who works tasks, people and language models, and
`[[roles]]` lists the roles they can hold. A role is only a name. A harness
that gives a role tools or instructions keeps them in its own config, keyed by
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
names the actor working it. When `[tasks.assignee] allowed` is set, each name
in it must be a declared actor. In a filter, `assignee == "ada"` compares the
name, and `role == "reviewer"` reads the assignee's roles from this registry
(see #link(u("/filters/"))[Filter language]).

When an assignee is required, `mt add` asks for it in a slot that
`MINDTAPE_PRINCIPAL` fills, so a harness that sets the variable files tasks
without a prompt. With the variable unset, an interactive `mt add` prompts,
and a non-interactive one fails naming the slot.

= Task file names

`[tasks.naming] stem` is the template `mt add` and `mt recur` use to name a new
task file, such as `stem = "{slug}.task-{yy}{mm}{dd}"`. Unset, `mt add` names
the file `{slug}` and `mt recur` names a spawned occurrence `{slug}-{date}`.

#table(
  columns: 2,
  table.header([Macro], [Expands to]),
  [`{slug}`], [The title, slugified.],
  [`{date}`],
  [Today's date, or the occurrence's date for `mt recur`, as `YYYY-MM-DD`.],

  [`{date:FMT}`], [The same date in another format.],
  [`{yy}`, `{mm}`, `{dd}`], [Two-digit year, month and day.],
  [`{n}`, `{n:0N}`],
  [The occurrence number, bare or zero-padded to `N` digits.],
)

`[tasks.naming] dedup` decides what `mt add` does when the file already
exists. `fail` stops before writing. `skip` writes nothing, exits 0 and
reports the existing path. `add-suffix` appends `-2`, `-3` and so on until the
name is free. Unset, `dedup` is `fail` when `stem` is unset and `add-suffix`
when it is set. `mt add --dedup` overrides it for one run.
