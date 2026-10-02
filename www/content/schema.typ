#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u

#let meta = (
  route: "/schema/",
  label: "Canonical schema",
  title: "Canonical schema",
  summary: "The task JSON schema that `mt ls --json` prints and `mt import` reads, for `jq` adapters to code against.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

One shape, two directions: `mt ls --json` serializes every task into the
canonical schema; `mt import` reads that same shape back and writes native
`.typ` files. Being one contract, `mt ls --json | mt import` round-trips every
field the schema carries — the field contract below names the one core task
field it does not — and a jq program that targets this schema works as both an
import adapter's *output* and something to hand-inspect from a live export.

#callout("note")[
  This page is the schema contract. `CanonicalTask`
  (`crates/mindtape/src/canonical.rs`) is the Rust type that implements it.
]

= Field contract

Every field is either #strong[authored] (round-trips through `mt import` onto
a task) or #strong[computed] (read-only, present for inspection, ignored on
import):

#table(
  columns: (auto, auto, 1fr),
  table.header([Field], [Kind], [Notes]),
  [`id`], [computed], [Defaults to the file stem.],
  [`file`],
  [computed],
  [Project-root-relative path — the value the `path` filter field compares
    and the one `--format csv` carries too, not the working-directory-relative
    path the table's `file` column and `--format path` display.],

  [`title`], [authored], [],
  [`status`], [authored], [`{name, date, note}`, `null` when unset.],
  [`priority`],
  [authored],
  [Framework object or bare scalar/label — see below.],

  [`difficulty`],
  [authored],
  [Number, label string, or a framework's name as fallback.],

  [`kind`], [authored], [],
  [`assignee`], [authored], [`null` when unset.],
  [`estimate`], [authored], [Opaque `unit -> quantity` map.],
  [`links`], [authored], [`[{task, kind, note}]`.],
  [`tags`], [authored], [Authored tags only.],
  [`derived_tags`],
  [computed],
  [Folder-derived virtual tags; import reads `tags` only, never this.],

  [`why`],
  [computed],
  [`[{stem, status, kind}]` — the unfinished direct blocking-kind targets a
    listed task waits on; empty unless it is blocked, and always empty from
    `mt eval`, which sees no task set.],

  [`children`],
  [computed],
  [Count of tasks whose `parent` edge names this one; `0` from `mt eval`.],

  [`progress`],
  [computed],
  [Terminal children over all children, `0..1`; `null` with no children.],

  [`due`], [authored], [`YYYY-MM-DD`.],
  [`start`], [authored], [`YYYY-MM-DD`.],
  [`reviewed`], [authored], [`YYYY-MM-DD`.],
  [`body`], [authored], [Raw Typst source after `#show: task.with(...)`.],
  [`recur`],
  [authored],
  [`{rule, anchor, until, count, materialize}`, `null` when the task does not
    recur — see #link(u("/recurring/"))[Recurring tasks] for the field
    itself.],

  [`extras`], [authored], [Opaque `x-<ns>` consumer extension slots.],
)

= Priority parity

`priority` emits its #strong[authored] form, not just a resolved score, so
`mt import` can rebuild the exact `priority: ice(...)` call it came from:

- A framework value becomes an object — `score` is a read-side computed extra
  (`config.tasks.priority.resolve`), `null` when unresolvable (e.g. no project
  config in scope). The importer reads `framework`/`inputs` and ignores `score`:

```json
{"priority": {"framework": "ice", "inputs": {"impact": 8, "confidence": 0.7, "ease": 6}, "score": 3.0}}
```

- A scalar or label value stays a #strong[bare number or string] — not an
  `{"value": ...}` wrapper, matching `mt ls --json`'s historical shape:

```json
{"priority": 8}
{"priority": "high"}
```

This is the one place the schema is shape-polymorphic. `difficulty` mirrors the
non-framework arm only: a framework difficulty is structurally possible but
never authored in practice, and falls back to its framework name as a label
string (`{"difficulty": "ice"}`).

= Links and body

Each `links:` edge is one object; `note` is `null` when the edge kind does not
require one:

```json
{"links": [{"task": "design-schema.typ", "kind": "depends-on", "note": null}]}
```

`body` is the raw Typst source that follows `#show: task.with(...)` — free-form
prose, headings, anything. `mt import` writes it back verbatim below the
emitted metadata call, so a `== Summary` section (or any other structure a
project's #link(u("/rules/"))[body] rules expect) survives a
round-trip.

= A full record

The whole table in one record — not typed here but sliced out of a live
`mt ls --json` against the `learn-piano` fixture, so the key set is the
binary's own and the page cannot describe a field it stopped emitting:

#raw(read("../generated/ls-task.json"), lang: "json", block: true)

Every key is present whether the task authored it or not: `kind`,
`difficulty`, `assignee` and `reviewed` read `null` because this one sets none
of them, and `extras` is `{}` because it carries no `x-<ns>` slot. Absent and
empty are the same read, so a consumer tests a value rather than a key's
existence. #link(u("/agents/"))[For AI agents] quotes the same capture beside
the `mt ls --json` invocation that produced it.

A record on its own carries no `schema_version`: the marker versions the
*document* shape a listing comes wrapped in, not each task inside it.

= The envelope

`mt ls --json` wraps its records in a document-level envelope. The version,
record count and `applied` attribution below are read from a real capture
against the `learn-piano` fixture, so none of them can drift:

// Derived from the generated capture rather than hand-typed, but summarised
// rather than inlined whole: the full transcript lives on the agents page,
// where the audience wants every field. No line count here — it is generated,
// so any number written down goes stale on the next capture.
#let envelope = json("../generated/ls-json.txt")
#let applied = envelope.applied
#let jstr(v) = if v == none { "null" } else { "\"" + v + "\"" }
#let jarr(vs) = "[" + vs.map(jstr).join(", ") + "]"
#raw(
  "{\n"
    + "  \"schema_version\": "
    + str(envelope.schema_version)
    + ",\n"
    + "  \"tasks\": [ … "
    + str(envelope.tasks.len())
    + " records, each shaped as above … ],\n"
    + "  \"applied\": {\n"
    + "    \"rule\": "
    + jstr(applied.rule)
    + ",\n"
    + "    \"superseded\": "
    + repr(applied.superseded)
    + ",\n"
    + "    \"facets\": {\n"
    + "      \"filter\": {\"from\": "
    + jstr(applied.facets.filter.from)
    + ", \"value\": "
    + jstr(applied.facets.filter.value)
    + "},\n"
    + "      \"sort\": {\"from\": "
    + jstr(applied.facets.sort.from)
    + ", \"value\": "
    + jarr(applied.facets.sort.value)
    + "},\n"
    + "      \"columns\": {\"from\": "
    + jstr(applied.facets.columns.from)
    + ", \"value\": "
    + jarr(applied.facets.columns.value)
    + "}\n"
    + "    }\n"
    + "  },\n"
    + "  \"truncated\": null,\n"
    + "  \"retired\": []\n"
    + "}",
  lang: "json",
  block: true,
)

#link(u("/agents/"))[For AI agents] shows the same envelope captured in full.

`schema_version` bumps only when the envelope or record shape changes in a way
that could break `mt import` or an external `jq` consumer.

`applied` reports which #link(u("/cli/"))[`[[ls.rules]]`] entry (if any) matched
the listing, and where each facet ended up. `rule` is the matched entry's label,
`null` when none matched. Each facet — `filter`, `sort`, `columns` — is
`{from, value}`: `from` names the source that won it (a view name, a rule label,
`manual` for a facet given on the command line, or `match-all` for the built-in
default) and `value` is the effective filter expression, sort spec, or column
list. `superseded` is `true` when the matched rule supplied at least one facet
and every one of them was overridden later, so a rule that lost is still named
rather than silently dropped. #link(u("/cli/"))[The CLI reference] renders the
same attribution for a human eye, as the `applied:` trailer under a table.

`truncated` is what `-n` / `--limit` cut, plus `--skip` dropped: `null` on a
complete listing, else `{"shown": 10, "total": 47, "next_skip": 10}` — the
rows you got against the rows the filter matched, so a bounded query answers
"did I see everything?" from the same parse. `next_skip` is `shown` added to
whatever `--skip` this run already applied, so paging forward is `mt ls
--skip <next_skip>` with everything else unchanged, never `shown` added to a
`--skip` tracked by hand. The key is always present, never omitted, so
reading it is one lookup rather than a test for existence.

`retired` lists every stem a #link(u("/filters/"))[`touches()`] call met in a
commit trailer that no loaded task answers to, even through a rename — the
tasks `mt rm` retired, sorted. It is `[]` when no filter called `touches()`,
and always present.

`mt import` reads `schema_version` permissively: absent — a bare JSON array,
or an envelope without the field, such as hand-written or `jq`-produced input
— is accepted; present and no newer than the version this binary understands
is accepted; a version newer than this binary supports is rejected outright,
naming both versions, rather than risk silently mis-parsing a shape it has
never seen.

#callout("tip")[
  See #link(u("/import-export/"))[Import & export] for the `mt import` flags and
  copy-pasteable recipes that produce and consume this schema.
]
