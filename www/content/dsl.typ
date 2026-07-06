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
  depends-on: (link(\"design-schema.typ\"),),
  relates-to: (link(\"write-docs.typ\"),),
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
  [`depends-on`], [Task links whose targets gate readiness. See #link(u("/filters/"))[ready / blocked].],
  [`relates-to`, `supersedes`, `duplicates`], [Non-gating task links, same `link("stem.typ")` form.],
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

Links reference another task by its filename stem, wrapped in `link()`:

```typ
depends-on: (link("design-schema.typ"),),
relates-to: (link("write-docs.typ"), link("audit.typ")),
```

Because identity is the filename, a moved or deleted target breaks the link —
`mt check` reports the dangling reference and any dependency cycle.

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
