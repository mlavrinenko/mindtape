#import "../utils/page.typ": page
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices
#import "../utils/dsl-field-notes.typ": dsl-field-notes

#let meta = (
  route: "/dsl/",
  label: "Task DSL",
  title: "Task DSL",
  summary: "Every field a task file can set, how to write a status, and what the body can hold.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The sketch is a slice of examples/dsl/, which `just check-examples`
// validates. The field table and the status list come from
// www/generated/dsl-fields.json, which www/generate-facts.sh reads off
// lib/prelude.typ.
#let sketch = checked-slices(json("../generated/dsl.json"), "dsl", ("task",))
#let facts = json("../generated/dsl-fields.json")

A task file imports the prelude and passes its fields to `task.with(...)`.
This one, from `examples/dsl/`, sets most of them:

#raw(sketch.task.code, lang: "typ", block: true)

#link(u("/concepts/"))[Concepts] explains how `mt` reads these files, names
them and resolves their links.

= Fields

Each field is a named parameter of `task`, so a Typst language server such as
#link("https://myriad-dreamin.github.io/tinymist/")[tinymist] completes the
names and shows each default. A misspelled field fails the file with an
`unknown task slot` error that names it.

A value is data. The helpers such as `ice(...)` and `every(...)` only build a
dictionary, and `mt` does the scoring and the date arithmetic.

#table(
  columns: 2,
  table.header([Field], [Value]),
  ..facts.fields.map(f => (raw(f.name), dsl-field-notes.at(f.name))).flatten(),
  // `x-<ns>` is the sink for extra fields, not a parameter, so the fact
  // cannot list it.
  [`x-<ns>`],
  [Data for a tool built on MindTape, which `mt` passes through unchecked. See #link(u("/concepts/"))[Concepts].],
)

A date field takes `on(2026, 7, 20)`, the prelude's short form of
`datetime(year: 2026, month: 7, day: 20)`.

= Writing a status

The prelude has a constructor for each of these statuses:
#facts.statuses.map(raw).join(", ").

Each takes the year, month and day the status was set, then an optional note
as a trailing content block, as `wip` does in the sketch above. A status in
your legend without a constructor is written with `status(...)`, which
#link(u("/statuses/"))[The status legend] shows. `mt flip` and the status
commands such as `mt wip` write this field for you.

= Links and recurrence

`links:` holds the task's typed edges to other tasks, and
#link(u("/links/"))[Task links] covers their kinds. `recur:` makes the task
repeat, and #link(u("/recurring/"))[Recurring tasks] covers its rules.

= Task bodies

Everything below `task.with(...)` is Typst markup, so `#` followed by a name
starts a Typst expression there. Prose such as `tag it #urgent` fails the
file. Every command then warns `unknown variable: urgent` and leaves the task
out, and `mt check` fails. Write a literal hash as `\#`, or put the text in
backticks, where `#` is plain text.
