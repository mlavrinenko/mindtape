#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/concepts/",
  label: "Concepts",
  title: "Concepts",
  summary: "How MindTape stores each task as a Typst file, reads the files on every command, and names and links tasks.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The task file is a copy of examples/api-service/tasks/add-redis-client.typ,
// and the transcripts are runs against a copy of examples/links/. Both come
// from www/generate-model.sh.

= A task is a file

Each task is one `.typ` file. The file imports the MindTape prelude and passes
the task's fields to `task.with(...)` in a show rule. Everything below that
line is the task's body, written in Typst markup. This is a task from
`examples/api-service/`:

#raw(read("../generated/concepts-task.typ"), lang: "typ", block: true)

`mt` reads every `.typ` file under the roots that `[tasks] roots` names. A
root can be a backlog folder or the source tree itself, so a task can sit next
to the code it changes. The #link(u("/dsl/"))[Task DSL] lists every field.

= Every command reads the files

`mt` runs no daemon and keeps no database or index. Each command evaluates the
task files under the roots with the Typst compiler, then applies the project's
config to what they hold. An edit counts as soon as the file is saved, whether
you, a script, an agent or `mt` made it. Git versions the backlog with the
code, and a branch carries its own tasks.

A cache in `.mindtape/cache/`, keyed by a hash of each file's bytes, lets `mt`
skip evaluating a file that did not change.

= Identity

A task's id is its filename stem, the file name without `.typ`, so
`tasks/add-redis-client.typ` is `add-redis-client`. Every command that takes a task takes its id, as in
`mt done add-redis-client`. Setting the `id:` field gives a task another id.
`mt check` reports two tasks with the same id.

= Links name files

A `links:` target is the path from the linking file to the target file. A
task in the same folder is `"other.typ"`, and a task one folder over is
`"../api/other.typ"`. `id:` plays no part in it. `mt check` reports a target
that names no task file, so renaming a file by hand breaks every link to it:

#terminal("concepts-mv-by-hand")

`mt mv` renames the file and rewrites each link to it. It keeps the file in
its folder, because a folder can set the task's
#link(u("/derived-tags/"))[derived tags]:

#terminal("concepts-mv")

#link(u("/links/"))[Task links] covers the link kinds and `mt link`, which
writes the path for you.

= Extension fields

A field named `x-<ns>` holds a project's own field, such as `x-team`, or data
for a tool built on MindTape, such as `x-speconaut`. `mt ls --json` passes it
unchanged under `extras`. `mt` checks it only when the config declares it in
`[tasks.fields.x-<ns>]`, whose `values` and `required` hold it like `kind`. Its
`values-when` filter limits the check to the tasks it matches, such as
`"!terminal"` to leave closed tasks alone. A declared field
takes one string, or with `multiple = true` a list of strings, and `values`
holds every element. `mt add` then offers a required one as a
#link(u("/templates/"))[slot]. A #link(u("/filters/"))[filter] reads its value,
declared or not. Any other field that `task`
does not declare fails the file with an `unexpected argument` error that
names it.

= Where a mistake is caught

The prelude refuses only what it would otherwise lose before `mt` reads the
file. That is an undeclared field, and a helper argument it cannot store, such
as a status note that is not a content block. Typst fails the file on these.

`mt check` checks everything else against the project's config: the
#link(u("/statuses/"))[status legend], the priority and difficulty scales, `kind`, required fields, links,
recurrence and the type of every value. It names the field and the position of
a wrong value, such as `tags[1]`.

= Configuration

`.mindtape/config.toml` holds only what a project changes from the defaults
built into `mt`. #link(u("/config/"))[Configuration] covers the config files
`mt` merges, in which order, and every section.

#callout("note")[
  The #link(u("/quickstart/"))[Quickstart] walks from install to a first
  finished task. The #link(u("/cli/"))[CLI reference] lists every command.
]
