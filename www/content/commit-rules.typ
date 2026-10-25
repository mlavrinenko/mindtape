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

Want every commit to say which task it works on? `mt check --commit-msg
<file>` checks the commit you're writing against the project's `[commits]`
config. Run it from a `commit-msg` hook and git refuses a commit that breaks a
rule. `mt check --commits <range>` runs the same checks over commits already
made. #link(u("/trace-commits-to-tasks/"))[Trace commits to tasks]
walks through the whole setup, from a refused commit to `touches()`.

The examples run in a git copy of `examples/rules/`, whose config holds the
three commit rules below. The rules the `mindtape:tasks` pack ships,
`work-names-a-task`, `no-unsquashed-fixup`, `flip-with-work` and
`named-tasks-finish-at-the-tip`, are
listed on
#link(u("/packs/"))[Rule packs].

#let parts = (
  ("trailer", "the trailer that names tasks"),
  ("commit-scope", "a rule that runs once per commit"),
  ("task-scope", "a rule that runs once per task the commit touches"),
  ("tip-scope", "a rule that runs once per task a range names"),
)

#let slices = checked-slices(
  json("../generated/commit-rules-config.json"),
  "commit-rules-config",
  parts.map(p => p.at(0)),
)

#raw(render-slices(slices, parts, comment: "#"), lang: "toml", block: true)

= The hook

The hook is one line. Git passes the message file as its first argument:

#terminal("commit-hook")

Outside a git repository, `mt check --commit-msg` fails with
`not a git repository`.

= The trailer

`[commits] trailer`, `Refs` by default, names the trailer whose values are
tasks, one #link(u("/concepts/#identity"))[stem] per line. Whatever the rules
say, `mt` refuses a value that's a path, or a stem no task has. Why stems? A
path changes when its task moves to another folder, and a stem is unique in
the project:

#terminal("commit-refs-path")

= Commit rules

`[[commits.rules]]` takes the same keys as `[[tasks.rules]]`: `name`, `when`,
`assert`, `message` and `severity`. `scope = "commit"`, the default, runs the
rule once per commit, over these fields:

#table(
  columns: 2,
  table.header([Field], [Holds]),
  [`subject`], [The message's first line.],
  [`refs`], [The trailer's values, as stems.],
  [`trailers`],
  [Every trailer line of the message, whatever its key, as `Key: value`.
    `matches(trailers, "^Changelog: none$")` finds one.],

  [`files`],
  [Every staged path, relative to the repository's top level. A rename gives
    both its paths. Under `git commit --amend` the paths are diffed against
    the parent of the commit being replaced, so an amend that adds a task's
    flip to its work commit still sees the work.],

  [`tasks_only`],
  [Whether the commit stages at least one file and every staged file is a
    `.typ` file under a `[tasks] roots` entry.],

  [`in_range`],
  [Whether the commit is read from a range: true under
    `mt check --commits`, false under `mt check --commit-msg`.],
)

The operators come from the #link(u("/filters/"))[filter language]. The
functions are `has(field)`, `miss(field)` and `matches(field, "regex")`. Over a
list, `matches` is true if any entry matches, so
`!matches(files, "^services/")` means the commit changes nothing under
`services/`.

`count(files, "glob"…)` is a number: how many of the commit's files the globs
admit, with the same `"!glob"` excludes as `changed()`. It sits on the left of
a comparison, so `count(files, "!docs/**") <= 1` caps the commit at one file
outside `docs/`. A rename counts once, as in `changed()`, while `has` and
`matches` still see both of its paths. `"@name"` reads a `[dicts.name]` path
set the same way `changed()` does. `subject` also compares with a string.

If a commit-scope rule reads a task field, `mt` refuses it when it loads the
config, and lists the fields a commit rule can read.

A breach names the message file and the rule. Here a migration has no
`Refs:` trailer, and then the same commit names its task:

#terminal("commit-scope-breach")

`subject` is also how a rule exempts a commit autosquash will drop. The
`mindtape:tasks` pack's `work-names-a-task` adds
`!matches(subject, "^(fixup|squash|amend)! ")` to its `when`, so a
`git commit --fixup` needs no `Refs:`. The pack's `no-unsquashed-fixup` is
the other half. It reads `in_range`, so the hook lets a fixup through and
`mt check --commits` refuses one that nobody squashed.

Rules merge by `name` across #link(u("/config/#layers"))[config layers] and
#link(u("/packs/"))[rule packs], like `[[tasks.rules]]`. To replace a
pack's rule, declare one with its name. To drop it, use `disable`.

= Task scope

To check what a commit does to a task, such as closing it without any work,
use `scope = "task"`. It runs the rule once for each task the commit changes
or names in its trailer. The rule reads the commit's fields above, and the
task in one of three states:

#table(
  columns: 2,
  table.header([Reads], [Holds]),
  [`head(<expr>)`],
  [The task as of `HEAD`, before the commit, or of `HEAD^` under `git commit --amend`.],

  [`staged(<expr>)`],
  [The task as the index holds it, which the commit records.],

  [`worktree(<expr>)`], [The task as the file on disk holds it.],
  [`named`], [Whether the trailer names the task.],
  [`retires_other`],
  [Whether the commit retires a task other than this one: `HEAD` holds it and
    the index doesn't, as after `mt rm`. Edits to other tasks' files don't
    count.],
)

A wrapper takes any #link(u("/filters/"))[filter expression] and evaluates it
against the whole project in that state, so `ready`, `blocked` and
`count(...)` read the tasks as they stand there. Inside a wrapper, `exists` is
false if the state has no file for the task, so
`head(exists) && !staged(exists)` is a deletion. As one side of a comparison,
a wrapper is the field's value in that state, as in
`worktree(status) == staged(status)`. `mt` refuses a bare task field, because
nobody reading the rule could tell which state it means.

`into("s")` means `staged(status == "s") && !head(status == "s")`: the commit
moves the task into status `s`.

The `mindtape:tasks` pack's `flip-with-work` uses `retires_other` so that a
cleanup task closes in the commit that runs `mt rm` on other tasks, though
that commit stages nothing but task files. It also lets a task of
`kind: "milestone"` close alone, because a milestone ends through its leaves
and has no work of its own.

A breach names the task after the rule. Here `mt done` changed the task
file, but only the code is staged:

#terminal("commit-task-scope")

= Tip scope

A branch that works on a task should finish it. `scope = "tip"` runs the rule
once for each task any commit of a range names in its trailer, over the task as
the range's newest commit, the tip, holds it. Only `mt check --commits` runs
it, since `mt check --commit-msg` checks one commit with no range. A tip rule
is an ordinary #link(u("/filters/"))[filter expression], like a
`[[tasks.rules]]` one: `status`, `ready` and `count(...)` read the tip's tasks,
and there are no commit fields or state wrappers. The `mindtape:tasks` pack
ships `named-tasks-finish-at-the-tip`, which asserts `!(status == "wip")`, and
the config above has the same rule as `finish-named-tasks`. A task named in several commits is checked once, and a
breach leads with the tip's id and names the task. Here a branch starts a task
and never finishes it:

#terminal("commit-tip")

A task missing from the tip, because the range deleted or retired it, is
skipped: it has no state to be wrong in. A tip rule that calls `changed(...)`
is skipped like any rule that reads the working tree.

To refuse a branch at merge time, run `mt check --commits main..HEAD` in the
`pre-merge` hook of `wt merge`, or in CI.

= Check a range

A hook doesn't run under `git commit --no-verify`, so check the commits a
push carries in CI too. `mt check --commits <range>` checks every commit
`git rev-list` lists for the range, oldest first, with the same trailer check
and rules. The one difference is `in_range`, which is true here.

Each commit is read against its first parent, a merge included. `subject` and
`refs` come from its message, and `files` from what it changes against that
parent. `staged(...)` reads the commit's tree and `head(...)` the parent's. A
trailer must name a task the commit's tree holds. A past commit has no working
tree, so `mt` skips a rule that calls `worktree(...)` or `changed(...)`, and
says so once. `count(files, …)` reads the commit's own files, so it still
runs. Only commits with something to report are listed, each with its lines
indented below it:

#terminal("commit-range")
