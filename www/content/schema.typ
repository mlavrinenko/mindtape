#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u

#let meta = (
  route: "/schema/",
  label: "Canonical schema",
  title: "Canonical schema",
  summary: "The task-JSON schema mt ls --json and mt import share — the contract a jq adapter codes against.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

One shape, two directions: `mt ls --json` serializes every task into the
canonical schema; `mt import` reads that same shape back and writes native
`.typ` files. Being one contract, `mt ls --json | mt import` round-trips a task
losslessly, and a jq program that targets this schema works as both an import
adapter's *output* and something to hand-inspect from a live export.

#callout("note")[
  This page mirrors the field contract documented on
  `CanonicalTask` in `crates/mindtape/src/canonical.rs` — treat that module doc
  as the source of truth if the two ever drift.
]

= Field contract

Every field is either #strong[authored] (round-trips through `mt import` onto
a task) or #strong[computed] (read-only, present for inspection, ignored on
import):

#table(
  columns: (auto, auto, 1fr),
  table.header([Field], [Kind], [Notes]),
  [`id`], [computed], [Defaults to the file stem.],
  [`file`], [computed], [Project-relative path.],
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

  [`due`], [authored], [`YYYY-MM-DD`.],
  [`start`], [authored], [`YYYY-MM-DD`.],
  [`reviewed`], [authored], [`YYYY-MM-DD`.],
  [`body`], [authored], [Raw Typst source after `#show: task.with(...)`.],
  [`extras`], [authored], [Opaque `x-<ns>` consumer extension slots.],
)

= Priority parity

`priority` emits its #strong[authored] form, not just a resolved score, so
`mt import` can rebuild the exact `priority: ice(...)` call it came from:

- A framework value becomes an object — `score` is a read-side computed extra
  (`config.tasks.priority.resolve`), `null` when unresolvable (e.g. no project
  config in scope). The importer reads `framework`/`inputs` and ignores `score`:

```json
{"priority": {"framework": "ice", "inputs": {"impact": 8, "confidence": 0.7, "ease": 6}, "score": 6.72}}
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
project's #link(u("/config/"))[required-field] rules expect) survives a
round-trip.

= A full record

```json
{
  "id": "wire-oauth",
  "file": "tasks/wire-oauth.typ",
  "title": "Wire OAuth",
  "status": {"name": "wip", "date": "2026-06-14", "note": null},
  "priority": {
    "framework": "ice",
    "inputs": {"impact": 8, "confidence": 0.7, "ease": 6},
    "score": 6.72
  },
  "difficulty": 5,
  "kind": "task",
  "estimate": {"days": 1, "hours": 2},
  "links": [{"task": "design-schema.typ", "kind": "depends-on", "note": null}],
  "tags": ["backend"],
  "derived_tags": ["auth"],
  "due": "2026-07-01",
  "start": null,
  "reviewed": null,
  "body": "== Summary\nWire up the login flow.\n",
  "extras": {"x-speconaut": {"spec": "auth.spec.typ"}}
}
```

A record on its own carries no `schema_version`: the marker versions the
*document* shape a listing comes wrapped in, not each task inside it.

= The envelope

`mt ls --json` wraps its records in a document-level envelope. The version and
record count below are read from a real capture against the `learn-piano`
fixture, so neither can drift:

// Derived from the generated capture rather than hand-typed, but summarised
// rather than inlined whole: the full transcript lives on the agents page,
// where the audience wants every field. No line count here — it is generated,
// so any number written down goes stale on the next capture.
#let envelope = json("../generated/ls-json.txt")
#raw(
  "{\n"
    + "  \"schema_version\": "
    + str(envelope.schema_version)
    + ",\n"
    + "  \"tasks\": [ … "
    + str(envelope.tasks.len())
    + " records, each shaped as above … ],\n"
    + "  \"applied\": { … which rule and view won each facet … }\n"
    + "}",
  lang: "json",
  block: true,
)

#link(u("/agents/"))[For AI agents] shows the same envelope captured in full.

`schema_version` bumps only when the envelope or record shape changes in a way
that could break `mt import` or an external `jq` consumer; `applied` names
which #link(u("/cli/"))[`[[ls.rules]]`] entry (if any) drove the listing.

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
