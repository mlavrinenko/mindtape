#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/commit-rules/",
  label: "Commit rules",
  title: "Commit rules",
  summary: "How `mt check --commit-msg` checks a commit's `Refs` trailer and message against the project's commit rules.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt check --commit-msg <file>` checks the commit being written against the
project's `[commits]` config. A `commit-msg` hook runs it, so git refuses a
commit that breaks a rule. `mt check --commits <range>` runs the same checks
over commits already made.

The examples run in a git copy of `examples/rules/`, whose config holds the two
commit rules below. The rules the `mindtape-tasks` pack ships,
`work-names-a-task` and `flip-with-work`, are listed on
#link(u("/rule-packs/"))[Rule packs].

#let parts = (
  ("trailer", "the trailer that names tasks"),
  ("commit-scope", "a rule that runs once per commit"),
  ("task-scope", "a rule that runs once per task the commit touches"),
)

#let slices = checked-slices(
  json("../generated/commit-rules-config.json"),
  "commit-rules-config",
  parts.map(p => p.at(0)),
)

#raw(render-slices(slices, parts, comment: "#"), lang: "toml", block: true)

= The hook

The hook is one line. git passes the message file as its first argument:

#terminal("commit-hook")

Outside a git repository, `mt check --commit-msg` fails with
`not a git repository`.

= The trailer

`[commits] trailer`, `Refs` by default, names the trailer whose values are
tasks, one stem per line. `mt` refuses a value that is a path, or a stem no
task has, whatever the rules say. A path changes when its task moves to
another folder, and a stem is unique in the project:

#terminal("commit-refs-path")

= Commit rules

`[[commits.rules]]` takes the fields of `[[tasks.rules]]`: `name`, `when`,
`assert`, `message` and `severity`. `scope = "commit"`, the default, runs the
rule once per commit, over these fields:

#table(
  columns: 2,
  table.header([Field], [Holds]),
  [`subject`], [The message's first line.],
  [`refs`], [The trailer's values, as stems.],
  [`files`],
  [Every staged path, relative to the repository's top level. A rename is
    both its paths.],

  [`tasks_only`],
  [Whether the commit stages at least one file and every staged file is a
    `.typ` file under a `[tasks] roots` entry.],
)

The operators are the #link(u("/filters/"))[filter language]'s. The functions
are `has(field)`, `miss(field)` and `matches(field, "regex")`. Over a list,
`matches` is true when any entry matches, so `!matches(files, "^services/")`
means the commit changes nothing under `services/`. `subject` also compares
with a string. `mt` refuses a task field in a commit-scope rule when it loads
the config, and lists the fields a commit rule reads.

A breach names the message file and the rule. Here a migration has no `Refs:`
trailer, and then the same commit names its task:

#terminal("commit-scope-breach")

Rules merge by `name` across config layers and
#link(u("/rule-packs/"))[rule packs], as `[[tasks.rules]]` do. A project
replaces a pack's rule by declaring one with its name, or drops it with
`disable`.

= Task scope

`scope = "task"` runs the rule once for each task the commit changes or names
in its trailer. The rule reads the commit's fields above, and the task in one
of three states:

#table(
  columns: 2,
  table.header([Reads], [Holds]),
  [`head(<expr>)`], [The task as of `HEAD`, before the commit.],
  [`staged(<expr>)`],
  [The task as the index holds it, which the commit records.],

  [`worktree(<expr>)`], [The task as the file on disk holds it.],
  [`named`], [Whether the trailer names the task.],
)

A wrapper takes any #link(u("/filters/"))[filter expression] and evaluates it
against the whole project in that state, so `ready`, `blocked` and `count(...)`
read the tasks as they stand there. Inside a wrapper, `exists` is false when the
state has no file for the task, so `head(exists) && !staged(exists)` is a
deletion. As one side of a comparison, a wrapper is the field's value in that
state, as in `worktree(status) == staged(status)`. `mt` refuses a bare task
field, because the reader could not tell which state it reads.

`into("s")` means `staged(status == "s") && !head(status == "s")`: the commit
moves the task into status `s`.

A breach names the task after the rule. Here `mt done` changed the task file,
but only the code is staged:

#terminal("commit-task-scope")

= Check a range

A hook does not run under `git commit --no-verify`, so CI checks the commits a
push carries. `mt check --commits <range>` checks every commit
`git rev-list` lists for the range, oldest first, with the same trailer check
and rules.

Each commit is read against its first parent, a merge included. `subject` and
`refs` come from its message, and `files` from what it changes against that
parent. `staged(...)` reads the commit's tree and `head(...)` the parent's. A
trailer must name a task the commit's tree holds. A past commit has no working
tree, so `mt` skips a rule that calls `worktree(...)` or `changed(...)` and
says so once. Only a commit with something to report is listed, with its lines
indented below it:

#terminal("commit-range")
