#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, mt-import, pkg
#import "../utils/site.typ": u

#show: page.with(
  title: "Task DSL",
  active: "/dsl/",
  summary: "Every field a task file can declare, and what the prelude versus the Rust layer enforce.",
)

A task file imports the prelude and applies `task.with`. Here is the full shape,
with the version-synced import:

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

#table(
  columns: 2,
  table.header([Field], [Value]),
  [`title`], [Display title. The only field the built-in template requires.],
  [`status`],
  [A legend value dated today or explicitly: `wip(2026, 7, 6)`. A trailing block is a note: `wip(2026, 7, 6)[behind a flag]`.],

  [`priority`],
  [A number on the scale, or a framework call. See #link(u("/priority/"))[Priority].],

  [`difficulty`], [The sibling scale to priority, same shape.],
  [`estimate`],
  [An opaque `unit: quantity` dict, e.g. `(days: 1, hours: 2)`. Units are never converted or compared — only the allowed set is checked.],

  [`kind`], [A config-defined enum value (e.g. `"feature"`, `"bug"`).],
  [`due`, `start`], [Dates via `datetime(year: …, month: …, day: …)`.],
  [`tags`], [A tuple of strings: `("backend", "auth")`.],
  [`links`],
  [Typed edges to other tasks — a sum of helper calls, e.g. `depends-on("a.typ") + related("b.typ")[why]`. See below.],

  [`recur`], [A recurrence rule + policy: `every(weeks: 1)`. See below.],
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
`x-<ns>` naming discipline. Everything semantic — the status legend, numeric
scales and classification, priority frameworks, the `kind` enum, required-unless-
terminal rules, bounds, and link resolution — is enforced by the Rust layer
against your #link(u("/config/"))[config], at `mt check` time.

#callout("note")[
  `x-<ns>` fields are yours to define. MindTape validates the core fields above
  and passes your extension fields through — see #link(u("/concepts/"))[the
    extension seam].
]

= Editor support

Every core field is a declared parameter of `task`, so a Typst language server
(#link("https://myriad-dreamin.github.io/tinymist/")[tinymist]) completes the
field names inside `task.with(...)` and shows the whole signature — each field
with its default — as signature help. A misspelled field is not silently kept:
it falls through to the extension sink and fails the file with
#raw("unknown task slot `titel`; tool extensions must use an x-<ns> slot").
