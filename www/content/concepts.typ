#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": mindtape, u

#let meta = (
  route: "/concepts/",
  label: "Concepts",
  title: "Concepts",
  summary: "How "
    + mindtape
    + " stores each task as a Typst file, reads the files on every command, and names and links tasks.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The task file is a copy of examples/api-service/tasks/add-redis-client.typ,
// and the transcripts are runs against a copy of examples/links/. Both come
// from www/generate-model.sh.

= A task is a file

Each task is one `.typ` file. The file imports the #mindtape prelude and passes
the task's fields to `task.with(...)` in a show rule. Everything below that
line is the task's body, in Typst markup. Here's a task from
`examples/api-service/`:

#raw(read("../generated/concepts-task.typ"), lang: "typ", block: true)

`mt` reads every `.typ` file under the roots you list in `[tasks] roots`. A
root can be a backlog folder or the source tree itself, so a task can sit next
to the code it changes. The #link(u("/dsl/"))[Task DSL] lists every field.

= Every command reads the files

There's no daemon, database or index. Each `mt` command evaluates the task
files under the roots with the Typst compiler, then applies the project's
config to what they hold. So an edit counts as soon as the file is saved,
whether you made it in your editor or a script, an agent or `mt` made it. Git
versions the backlog with the code, and each branch carries its own tasks.

To keep this fast, `mt` caches each file's result in `.mindtape/cache/`, keyed
by a hash of the file's bytes, and skips a file that hasn't changed.

= Identity

A task's id is its filename stem, the file name without `.typ`, so
`tasks/add-redis-client.typ` is `add-redis-client`. Every command that takes a
task takes its id, as in `mt done add-redis-client`. Set the `id:` field to
give a task a different id. `mt check` reports two tasks with the same id.

= Links name files

A `links:` target is the path from the linking file to the target file. A task
in the same folder is `"other.typ"`, and a task one folder over is
`"../api/other.typ"`. `id:` plays no part in it. `mt check` reports a target
that names no task file, so if you rename a file by hand, every link to it
breaks:

#terminal("concepts-mv-by-hand")

`mt mv` renames the file and rewrites each link to it. It keeps the file in its
folder, because a folder can set the task's
#link(u("/derived-tags/"))[derived tags]:

#terminal("concepts-mv")

#link(u("/links/"))[Task links] covers the link kinds and `mt link`, which
writes the path for you.

= Extension fields

A field named `x-<ns>` holds a field of your own, such as `x-team`, or data
for a tool built on #mindtape, such as `x-speconaut`. `mt ls --json` passes it
through unchanged under `extras`.

`mt` checks an `x-` field only if you declare it in `[tasks.fields.x-<ns>]`.
There, `values` and `required` work the same as they do for `kind`. A
`values-when` filter limits the check to the tasks it matches, such as
`"!terminal"` to leave closed tasks alone. A declared field takes one string,
or a list of strings with `multiple = true`, and `values` covers every
element. `mt add` offers a required one as a
#link(u("/templates/"))[slot]. A #link(u("/filters/"))[filter] can read the
value whether you declared the field or not.

Any other field that `task` doesn't declare fails the file with an
`unexpected argument` error that names it.

= Where a mistake is caught

The prelude refuses only what it would lose before `mt` reads the file: a
field it doesn't declare, and a helper argument it can't store, such as a
status note that isn't a content block. Typst fails the file on these.

`mt check` checks everything else against the project's config: the
#link(u("/statuses/"))[status legend], the priority and difficulty scales,
`kind`, required fields, links, recurrence and the type of every value. It
names the field and where the wrong value sits, such as `tags[1]`.

= Configuration

`.mindtape/config.toml` holds only what you change from the defaults built
into `mt`. #link(u("/config/"))[Configuration] covers which config files `mt`
merges, in what order, and every section.

#callout("note")[
  The #link(u("/quickstart/"))[Quickstart] goes from install to your first
  finished task. The #link(u("/cli/"))[CLI reference] lists every command.
]
