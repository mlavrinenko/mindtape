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
line is the task's body, in Typst markup. A task from
`examples/api-service/`:

#raw(read("../generated/concepts-task.typ"), lang: "typ", block: true)

`mt` reads the task sources under the roots you list in `[[tasks.roots]]`. A
root can be a backlog folder or the source tree itself, so a task can sit next
to the code it changes.
#link(u("/concepts/#which-files-count-as-tasks"))[Which files count as tasks]
says which files under a root qualify. The #link(u("/dsl/"))[Task DSL] lists
every field.

== Which files count as tasks

A task source is a regular `.typ` file under a configured `[[tasks.roots]]`
entry, outside every reserved component and template directory below. `mt`
handles a path as text: it folds `.` and `..` without touching the
filesystem. A root needs no trailing slash, and whether a file is a task
depends only on its resolved path. These rules narrow that base:

- The extension is `.typ`, case-sensitive, so `TASK.TYP` is not a task.
- No path component may be `.mindtape` or `.git`, in any ASCII case. The
  project's metadata directory never contains a task.
- A file under a `[templates] dirs` entry is a template, not a task. A
  #link(u("/templates/"))[template] is a `.typ` file with `slot(...)` calls,
  and `mt check` checks it on its own.
- The entry at the path must be a regular file. A symlink is never a task
  source, even one whose target is a regular `.typ` file. No scan follows a
  directory symlink.

A working-tree config whose root is under `.mindtape` or `.git`, or whose
template directory contains a root or the project root, is refused when it
loads, because it could never yield a task.

In a Git work tree, `mt` asks Git which paths are in the working tree: every
tracked regular file and every untracked file Git does not ignore. That keeps
`mt ls` in step with the commit states. A tracked file stays a task when an
ignore pattern matches it, and an untracked ignored file is not a task. A
tracked file deleted from the working tree is not a task there, though a
commit state still lists it. An ignored task root lists only its tracked
files, and the load warns once that the root's untracked files are invisible.
Unignore the root to load them.

Git's rules are the repository's `.gitignore` files, `.git/info/exclude` and
your global excludes file. The last two are machine-local, so two people can
see different tasks in one clone.

An untracked ignored path is not scanned at all. When an ignore rule starts
matching an untracked task file, that task leaves the listing and its links
resolve as missing, but the file stays where it is: `mt` does not delete it.
A tracked file is unaffected.

A root outside the project is working-tree only. `mt` finds its files for
`mt ls`, `mt check` and `mt fix`, and a commit state cannot list them, so
commit classification reads only paths inside the project.

Outside a Git work tree, or on a machine without `git`, `mt` walks each root
on disk with the same extension, reserved-name and template rules. The walk
consults no ignore rules and does not follow a directory symlink. Any other
Git failure stops the load with an error naming the root.

Four examples.

- A project whose only root is `.` keeps its backlog across the tree. Its template at
  `.mindtape/templates/task.typ` sits under a reserved component and a
  template directory, so the template is not a task, while a real task can sit
  beside the code it changes.
- A tracked `tasks/old.typ` that a `.gitignore` line matches stays a task,
  because Git still tracks it. `git rm --cached tasks/old.typ` drops it from
  the listing without touching the file.
- A task file that becomes a symlink, such as
  `tasks/alias.typ -> ../shared/task.typ`, stops being a source even when the
  target is a regular `.typ` file.
- `mt add` refuses a destination that is itself a symlink.

= Every command reads the files

There's no daemon, database or index. Each `mt` command evaluates the task
files under the roots with the Typst compiler, then applies the project's
config to what they contain. So the next command sees a saved change, whether
it came from your editor, a script, an agent or `mt` itself. Git versions the
backlog with the code, and each branch has its own tasks.

To keep this fast, `mt` caches each file's result in `.mindtape/cache/`, keyed
by a hash of the file's bytes, and skips a file that hasn't changed.

= Identity

A task's id is its filename stem, the file name without `.typ`, so
`tasks/add-redis-client.typ` is `add-redis-client`. Every command that takes a
task takes its id, as in `mt done add-redis-client`. Set the `id:` field to
give a task a different id. `mt check` reports two tasks with the same id.

`mt ls` also shows a short hash for each task in its `SLUG#` column, like
`re8`. Every command takes it in place of the name.

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

An `x-<ns>` field, such as `x-team`, stores your own data, or data for a tool
built on #mindtape, such as `x-speconaut`. `mt ls --json` passes it through
unchanged under `extras`.

To have `mt` check one, declare it in `[tasks.fields.x-<ns>]`.
#link(u("/config/#extension-fields"))[Configuration] covers the rest.

= Where a mistake is caught

Typst stops a file before any rule runs. A misspelled field, or any other
field `task` doesn't declare, fails with `unexpected argument: titel`. A helper
refuses a value it can't hold, like a status note that isn't a content block.

`mt check` checks everything else against the project's config: the
#link(u("/statuses/"))[status legend], the priority and difficulty scales,
`kind`, required fields, links, recurrence and the type of every value. It
names the field and shows where the wrong value is, such as `tags[1]`.

= Configuration

`.mindtape/config.toml` contains only what you change from the defaults built
into `mt`. #link(u("/config/"))[Configuration] covers which config files `mt`
merges, in what order, and every section.

#callout("note")[
  The #link(u("/quickstart/"))[Quickstart] goes from install to your first
  finished task. The #link(u("/cli/"))[CLI reference] lists every command.
]
