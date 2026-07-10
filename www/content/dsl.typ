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

#mt-import("#show: task.with(
  title: \"Wire OAuth login\",
  status: wip(2026, 7, 6)[behind a feature flag],
  priority: ice(impact: 8, confidence: 0.9, ease: 6),
  difficulty: 4,
  estimate: (days: 1, hours: 2),
  kind: \"feature\",
  due: datetime(year: 2026, month: 7, day: 20),
  start: datetime(year: 2026, month: 7, day: 8),
  tags: (\"backend\", \"auth\"),
  links: depends-on(\"design-schema.typ\") + related(\"write-docs.typ\")[docs companion],
  x-speconaut: (spec: \"auth/login\"),
)

== Summary
The body is free-form Typst below the metadata.")

= Fields

#table(
  columns: 2,
  table.header([Field], [Value]),
  [`title`], [Display title. The only field the built-in template requires.],
  [`status`], [A legend value dated today or explicitly: `wip(2026, 7, 6)`. A trailing block is a note: `wip(2026, 7, 6)[behind a flag]`.],
  [`priority`], [A number on the scale, or a framework call. See #link(u("/priority/"))[Priority].],
  [`difficulty`], [The sibling scale to priority, same shape.],
  [`estimate`], [An opaque `unit: quantity` dict, e.g. `(days: 1, hours: 2)`. Units are never converted or compared — only the allowed set is checked.],
  [`kind`], [A config-defined enum value (e.g. `"feature"`, `"bug"`).],
  [`due`, `start`], [Dates via `datetime(year: …, month: …, day: …)`.],
  [`tags`], [A tuple of strings: `("backend", "auth")`.],
  [`links`], [Typed edges to other tasks — a sum of helper calls, e.g. `depends-on("a.typ") + related("b.typ")[why]`. See below.],
  [`recur`], [A recurrence rule + policy: `every(weeks: 1)`. See below.],
  [`x-<ns>`], [Opaque extension slot for tools built on MindTape. Carried untouched; the consumer validates it.],
)

= Status shortcuts

Each value in your status legend is a constructor. Call it with a date, and
optionally a note block:

```typ
status: proposed(2026, 7, 5)
status: wip(2026, 7, 6)[behind a feature flag]
status: done(2026, 7, 20)
```

A terminal status (like `done`) relaxes fields marked `required = "non-terminal"`.
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
hatch; expansion is a later feature, gated behind `[tasks.recur] rrule` (see
#link(u("/config/"))[Configuration]) until it ships. A task never advances
itself — only an explicit `mt recur` spawns or mutates the next occurrence
(see #link(u("/cli/"))[CLI reference]); `mt done` on a recurring task is a
pure close and never touches `recur`.

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
