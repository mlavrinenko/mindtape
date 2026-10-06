#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/import-export/",
  label: "Import & export",
  title: "Import & export",
  summary: "How to bring tasks in from another tracker with `mt import` and `jq`, and take them out with `mt ls '' --json`.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt import` reads tasks as JSON in the #link(u("/schema/"))[canonical schema]
and writes one `.typ` file for each. `mt ls '' --json` prints every task in the
same schema. A `jq` program turns another tracker's export into that schema, or
turns the schema into another tracker's format. The recipes below run in the
docs build, so each output on this page is what the command printed.

= Import

`mt import` reads a file, or standard input when you give none. It checks each
record against this project's config: the
#link(u("/statuses/"))[status legend], the priority scale,
the `kind` values and the allowed tags. It checks the whole batch as one set,
so records that link to each other resolve. Then it writes the records that
pass. A record that fails is reported and not written, and the rest are still
written.

The report on standard error names each record, `filed` or `skipped`, with the
reason for a skip. It also lists each change it made to fit the config:
`mapped`, `defaulted`, `dropped` or `violated`. `mt import` writes the records
that pass, prints the report, then exits 1 when it skipped any record, so a
script can tell a partial import from a whole one.

`--map old=new` renames a status before the check, for a source whose names
differ from your legend. `--to <dir>` writes the batch into a subfolder of the
first #link(u("/concepts/#a-task-is-a-file"))[task root], and `--into <dir>` into a directory, in place of each record's
`file`. `--dry-run` checks the batch and writes nothing: its
report says `would file` for each record that would be written and ends with
`dry run: nothing written`. The
#link(u("/cli/import/"))[`mt import` page] lists every flag.

The adapters in `contrib/jq/` read the tracker's own export. They do not read
your config. They emit the status names of the legend `mt init` writes by
default: `proposed`, `done` and `cancelled`. For a project with another legend,
add `--map` to `mt import`, such as `--map proposed=todo`.

== From Taskwarrior

`task export > taskwarrior.json` saves Taskwarrior's tasks.
`contrib/jq/taskwarrior.jq` reshapes them:

#terminal("import-taskwarrior")

A `pending` task becomes `proposed`, `completed` becomes `done` and `deleted`
becomes `cancelled`. Priority `H`, `M` and `L` become 8, 5 and 2. The
`project` and the tags become tags. The description becomes the title, and
also the body under a `Summary` heading. The UUID, urgency, annotations and
dependencies go into the `x-tw` field.

== From GitHub Issues

`gh issue list --state all --json number,title,body,state,labels,assignees,createdAt,url > issues.json`
saves a repository's issues. `gh` fetches 30 unless you pass `--limit`.
`contrib/jq/github.jq` reshapes them:

#terminal("import-github")

An open issue becomes `proposed` and a closed one `done`. Labels become tags,
and the creation date becomes `start`. The issue number, URL and assignees go
into the `x-gh` field. `--to github` writes the batch to `tasks/github/`, and
in a project that derives tags from folders each task gets a `github` tag.

== From todo.txt

todo.txt is one task per line, so a first `jq` pass splits the file into a
JSON array of lines. `contrib/jq/todo.txt.jq` reshapes the array:

#terminal("import-todotxt")

A line that starts with `x ` becomes `done`, and any other line `proposed`.
Priority `(A)`, `(B)` and `(C)` become 8, 5 and 2. A `+project` word becomes a
tag and a `due:` value the due date. The title keeps `@context` words and any
other `key:value` word. The `x-todotxt` field holds them too, with the
priority letter. Adjust the patterns in the file to the parts of the format
your file uses.

= Round trip

An export imports back whole. `mt import` names each file from the record's
`id`, so a link to a task still resolves, and a record with no `id`, as the jq
adapters emit, is named from its title as `mt add` names it. With neither
`--into` nor `--to`, a record lands in the folder its `file` names when that
folder is under a task root, so an export of several folders imports back as
the same folders. A name taken anywhere in the project gets a `-2` suffix, the
report says `alpha: filed as alpha-2`, and a link to it does not follow the
rename.

This exports `examples/round-trip/`, whose four tasks use every field the
import writes:

#terminal("round-trip-export")

The empty filter `''` lists every task. `mt ls` without it applies the
project's default #link(u("/named-sets/#views"))[view], which in a new project hides finished tasks, so their
records would be missing from the file.

A new project, made by `mt init --no-input`, takes the file back:

#terminal("round-trip-import")

The docs build compares the two projects' `mt ls '' --json` output and fails
if any field differs.

= Export

MindTape has no export command. Each task is a `.typ` file in your repository,
so git and `cp` already move them. For another tracker's format, pipe
`mt ls '' --json` through `jq`.

== To todo.txt

#terminal("export-todotxt")

The recipe drops cancelled tasks, which todo.txt has no mark for. A finished
task starts with `x`, and a priority of 7 or more becomes `(A)`.

== To CSV

`mt ls --format csv` prints the listing's columns:

#terminal("export-csv")

For other columns, build the rows in `jq` and format them with `@csv`:

#terminal("export-csv-jq")

A priority from a framework such as `ice` is an object, which `@csv` refuses,
so the recipe takes its `score`.

== To Taskwarrior

#terminal("export-taskwarrior")

The array has the fields and date format `task export` prints. Save it to a
file and run `task import` on it, a step the docs build does not run because it
has no Taskwarrior.

#callout("tip")[
  The #link(u("/schema/"))[canonical schema] lists every field a recipe can
  read, and which of them `mt import` reads back.
]
