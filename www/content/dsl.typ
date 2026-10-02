#import "../utils/page.typ": page
#import "../utils/ui.typ": mt-import
#import "../utils/site.typ": u
#import "../utils/dsl-field-notes.typ": dsl-field-notes

#let meta = (
  route: "/dsl/",
  label: "Task DSL",
  title: "Task DSL",
  summary: "Every field a task file can set, the status shortcuts, and what the body can hold.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

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

Every core field is a declared parameter of `task`, so a Typst language server
(#link("https://myriad-dreamin.github.io/tinymist/")[tinymist]) completes the
names inside `task.with(...)` and shows the whole signature, each field with
its default. A misspelled field is not silently kept: it falls through to the
extension sink and fails the file, naming the field. Values are shapes, never
computed results — date arithmetic and scoring happen in `mt`, not in the file.

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
The default legend and its glyphs live in #link(u("/statuses/"))[Statuses].

= Links and recurrence

`links:` holds every typed edge between tasks, and `recur:` turns a task into a
series. Both are fields like the others; both are deep enough for a page of
their own. #link(u("/links/"))[Task links] carries the six built-in kinds, the
project-declared ones, and the CLI writer; #link(u("/recurring/"))[Recurring
  tasks] carries the cadences, the policy floor, and the `rrule(...)` escape
hatch.

= Task bodies

Everything below `task.with(...)` is real Typst markup — headings, lists, code
blocks, whatever you write. That includes `#`, which in Typst opens an inline
expression rather than printing a hash. So prose like `Channel #N carries it`
is read as a reference to a variable `N`, and the file fails to load:

```text
warning: tasks/probe.typ: eval error: unknown variable: N
```

Write a literal hash as `\#`, or put the run of prose in a code block:

```typ
== Summary

Channel \#N carries it, per issue \#412.

`#[allow(dead_code)]` is fine inside backticks — no escape needed.
```

Bodies are deliberately left as real Typst rather than escaped for you, since
escaping would break the markup that already works.

How the two layers divide the field table — what the prelude refuses before it
is written, and what the Rust layer refuses at `mt check` — is on
#link(u("/concepts/"))[Concepts].
