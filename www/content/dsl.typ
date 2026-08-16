#import "../utils/page.typ": page
#import "../utils/ui.typ": mt-import
#import "../utils/site.typ": u
#import "../utils/dsl-field-notes.typ": dsl-field-notes

#metadata((
  route: "/dsl/",
  label: "Task DSL",
  title: "Task DSL",
  summary: "Every field a task file can declare, and what the prelude versus the Rust layer enforce.",
)) <page-meta>

#show: page.with(active: "/dsl/")

A task file imports the prelude and applies `task.with`. The full shape:

#mt-import(
  "#show: task.with(
  title: \"Für Elise — polish for recital\",
  status: wip(2026, 7, 6)[C-section run still rushes],
  priority: ice(impact: 8, confidence: 0.9, ease: 6),
  difficulty: 4,
  estimate: (days: 1, hours: 2),
  kind: \"practice\",
  due: datetime(year: 2026, month: 7, day: 20),
  start: datetime(year: 2026, month: 7, day: 8),
  tags: (\"classical\", \"recital\"),
  links: depends-on(\"major-scales-two-octaves.typ\") + related(\"clair-de-lune-practice.typ\")[same recital],
  x-metronome: (target-bpm: 120),
)

== Summary
The body is free-form Typst below the metadata.",
)

= Fields

// The field list itself comes from www/generated/dsl-fields.json — read back
// off lib/prelude.typ's own `task` signature via its `<mindtape>` marker, not
// mirrored by hand. `x-<ns>` isn't a core field (it's the extension sink, not
// a named parameter), so it stays a hand-added row below the derived ones.
// `just check-doc-facts` fails the build if a field here has no matching
// prose in dsl-field-notes.typ.
#let dsl-fields = json("../generated/dsl-fields.json").fields

#table(
  columns: 2,
  table.header([Field], [Value]),
  ..dsl-fields.map(f => (raw(f.name), dsl-field-notes.at(f.name))).flatten(),
  [`x-<ns>`],
  [Opaque extension slot for tools built on MindTape. Carried untouched; the consumer validates it.],
)

= Status shortcuts

Each value in your status legend is a constructor. Call it with a date, and
optionally a note block:

```typ
status: proposed(2026, 7, 5)
status: wip(2026, 7, 6)[behind a feature flag]
status: done(2026, 7, 20)
```

A terminal status (like `done`) relaxes fields marked `required = "!terminal"`.
The default legend and its glyphs live in #link(u("/config/"))[Configuration].

= Task links

One field, `links:`, holds every typed edge to another task. Each built-in
helper — `depends-on`, `parent`, `recurred-from`, `supersedes`, `duplicates`,
`related` — is variadic over plain filename-stem strings (never wrapped in
`link()`), and returns the edge array `links:` expects directly:

```typ
links: depends-on("design-schema.typ", "audit.typ"),
```

A trailing content block is the note, uniform with `status`; combine several
kinds with `+`:

```typ
links: depends-on("design-schema.typ") + related("write-docs.typ")[docs companion],
```

Targets and the note are told apart by type, so a helper rejects an argument
that is neither. `link()` returns content and content is the note, so
`depends-on(link("design-schema.typ"))` builds no edge at all — and instead of
yielding an empty edge array it fails the file with
`` `depends-on()` needs at least one target ``, plus why the target vanished.
A second note block, a named argument and a non-string target fail the same
way, each naming the helper.

`related`'s note is required — the escape hatch for "connected, but not a
dependency" needs a reason. `edge(kind, ..targets)[note]` builds an edge of any
project-declared custom kind (`[[tasks.links.kinds]]`); `depends-on` and
`parent` are the only kinds checked for cycles, and only `depends-on` gates
readiness by default. Because identity is the filename, a moved or deleted
target breaks the link — `mt check` reports the dangling reference and any
cycle.

A custom kind can point *outside* the backlog instead — the commit, pull request
or build that discharges the task — by declaring a `validate` other than the
default `local-task-path` (see #link(u("/config/"))[Configuration]). Its targets
are checked for shape, never resolved to a stem, and never join the task graph:

```typ
links: edge("discharged-by", "https://github.com/acme/app/pull/42"),
```

= Recurring tasks

`every()` shapes one period unit — no date math in Typst, the same discipline
as the priority frameworks — plus the policy that rides along with it:

```typ
recur: every(weeks: 1)
recur: every(days: 3)
recur: every(weeks: 1, on: (mon, wed, fri))
recur: every(
  weeks: 1,
  anchor: "completion",   // "completion" (default) or "due"
  until: on(2027, 1, 1),  // series end
  count: 10,              // remaining occurrences
  materialize: "spawn",   // "spawn" (default) or "mutate"
)
```

`rrule("FREQ=MONTHLY;BYDAY=1MO")` shapes the same policy dict for an escape
hatch — calendar-complex cadences `every()` cannot express: "first Monday of
the month" (`FREQ=MONTHLY;BYDAY=1MO`), "last day of month"
(`BYMONTHDAY=-1`), "every weekday" (`BYDAY=MO,TU,WE,TH,FR`). It is expanded
by the `rrule` crate (github.com/fmeringdal/rust-rrule), behind mindtape's own
`rrule` cargo feature *and* `[tasks.recur] rrule` (see
#link(u("/config/"))[Configuration]) — either gate missing rejects an
authored `rrule(...)` with a precise message, and the common `every()` path
stays free of the extra dependencies (`chrono-tz`, `regex`) either way.

`rrule(...)` is date-anchored only: it implies `anchor: "due"` and rejects
`anchor: "completion"` outright (RFC 5545 has no completion-relative
primitive). DTSTART is seeded from the task's `due`; the next occurrence is
the rule's first instance strictly after it. `mt check` validates the raw
string at parse time and rejects a malformed one, quoting it in the error
(e.g. `invalid rrule "FREQ=NOPE": ...`).

A task never advances itself — only an explicit `mt recur` spawns or mutates
the next occurrence (see #link(u("/cli/"))[CLI reference]); `mt done` on a
recurring task is a pure close and never touches `recur`.

= What enforces what

The prelude enforces #strong[structure] only: one task per file, and the
`x-<ns>` naming discipline — those fields are yours to define, and MindTape
passes them through (see #link(u("/concepts/"))[the extension seam]).
Everything semantic — the status legend, numeric scales and classification,
priority frameworks, the `kind` enum, required-unless-terminal rules, bounds,
and link resolution — is enforced by the Rust layer against your
#link(u("/config/"))[config], at `mt check` time.

= Editor support

Every core field is a declared parameter of `task`, so a Typst language server
(#link("https://myriad-dreamin.github.io/tinymist/")[tinymist]) completes the
field names inside `task.with(...)` and shows the whole signature — each field
with its default — as signature help. A misspelled field is not silently kept:
it falls through to the extension sink and fails the file with
#raw("unknown task slot `titel`; tool extensions must use an x-<ns> slot").
