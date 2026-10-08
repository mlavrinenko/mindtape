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
Here's one from `examples/dsl/` that sets most of them:

#raw(sketch.task.code, lang: "typ", block: true)

#link(u("/task-files/"))[Task files] walks through the Typst in it, line by
line. #link(u("/concepts/"))[Concepts] covers how `mt` reads these files,
names them and resolves their links.

= Fields

Each field is a named parameter of `task`, so a Typst language server such as
#link("https://myriad-dreamin.github.io/tinymist/")[tinymist] completes the
names and shows each default. Misspell a field as `titel` and the file fails
with `unexpected argument: titel`, which the language server marks under the
misspelled name.

A value is plain data. Helpers like `ice(...)` and `every(...)` only build a
dictionary. `mt` does the scoring and the date arithmetic.

#table(
  columns: 2,
  table.header([Field], [Value]),
  ..facts.fields.map(f => (raw(f.name), dsl-field-notes.at(f.name))).flatten(),
  // `x-<ns>` is the sink for extra fields, not a parameter, so the fact
  // cannot list it.
  [`x-<ns>`],
  [Data for a tool built on MindTape, which `mt` passes through, and checks only if the config declares it. See #link(u("/concepts/"))[Concepts].],
)

A date field takes `on(2026, 7, 20)`, the prelude's short form of
`datetime(year: 2026, month: 7, day: 20)`. So do the `start:` and `until:`
arguments of `every(...)`.

= Writing a status

The prelude has a constructor for each of these statuses:
#facts.statuses.map(raw).join(", ").

Each takes the year, month and day the status was set, then an optional note
as a trailing content block, like `wip` in the sketch above. Your project's
#link(u("/statuses/"))[status legend], its list of allowed statuses, can name
a status with no constructor. Write that one with `status(...)`.

You can type the status and its date yourself, or let `mt flip` and the status
commands such as `mt wip` write the field for you.

= Links and recurrence

`links:` holds the task's typed edges to other tasks. See
#link(u("/links/"))[Task links] for the kinds. `recur:` makes the task repeat.
See #link(u("/recurring/"))[Recurring tasks] for the rules.

= Task bodies

Everything below `task.with(...)` is the body, in Typst markup.
#link(u("/task-files/"))[Task files] shows what the markup holds and how to
write a literal `#` in it.
