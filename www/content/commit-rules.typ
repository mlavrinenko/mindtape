#import "../utils/page.typ": page
#import "../utils/site.typ": u

#let meta = (
  route: "/commit-rules/",
  label: "Commit rules",
  title: "Commit rules",
  summary: "How `mt check --commit-msg` checks a commit's `Refs` trailer and message against the project's commit rules.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`mt check --commit-msg <file>` holds the commit being written to `[commits]`,
from a `commit-msg` hook — one line opts a repository in, wired with whatever
hook tool it already uses:

```sh
mt check --commit-msg "$1"
```

`[commits] trailer` (default `Refs`) is the trailer whose values name tasks,
one stem per line: `Refs: rewrite-loader`. A value that is a path, or a stem
no task carries, is always refused, rules or no rules — a path breaks the day
its task moves folders, and stems are unique project-wide:

```
error: .git/COMMIT_EDITMSG:5: Refs: `tasks/rewrite-loader.typ` is a path; write the stem: `Refs: rewrite-loader`
```

`[[commits.rules]]` takes the `[[tasks.rules]]` shape — `name`, `when`,
`assert`, `message`, `severity` — with `scope = "commit"` (the default): the
rule runs once per commit, over the commit's own fields rather than a task's.

```toml
[commits]
trailer = "Refs"

[[commits.rules]]
name    = "work-names-a-task"
scope   = "commit"
when    = '!tasks_only'
assert  = 'has(refs)'
message = "name the task this commit works on: `Refs: <stem>`"
severity = "error"
```

#table(
  columns: 2,
  [*Field*], [*Holds*],
  [`subject`], [the message's first line],
  [`refs`], [the trailer's values, as stems],
  [`files`],
  [every staged path, relative to the repository's top level; a rename is both its sides],

  [`tasks_only`],
  [the commit stages at least one file, and every one is a `.typ` under a `[tasks] roots` entry],
)

The operators are the #link(u("/filters/"))[filter language]'s; the functions
are `has(field)` / `miss(field)`, and `matches(field, "regex")`, which over a
list asks whether any entry matches — `!matches(files, "^services/")` reads
"touches nothing under `services/`". `subject` also compares against a
literal. A task field is refused at config load, naming the fields a commit
rule reads.

A breach names the message file and the rule:

```
error: .git/COMMIT_EDITMSG: commits.rules.work-names-a-task: name the task this commit works on: `Refs: <stem>`
```

The rules merge by `name` across layers and #link(u("/rule-packs/"))[rule
  packs] like `[[tasks.rules]]`, so a project overrides a shipped one by
redeclaring its name, or drops it with `disable`. Outside a git repository
`--commit-msg` fails with `not a git repository`.

= Task scope

`scope = "task"` runs the rule once per task the commit changes or names in
its trailer. It reads the commit's fields above, and the task through one of
three states of the project:

#table(
  columns: 2,
  [*Reads*], [*Holds*],
  [`head(<expr>)`], [the task as of `HEAD`, before the commit],
  [`staged(<expr>)`], [the task as the index holds it: what the commit records],
  [`worktree(<expr>)`], [the task as the file on disk holds it],
  [`named`], [the trailer names the task],
)

A wrapper takes any #link(u("/filters/"))[filter expression] and evaluates it
against that state's whole project, so `ready`, `blocked` and `count(...)`
read the tasks as they stand there. Inside one, `exists` is false where the
state has no file for the task: `head(exists) && !staged(exists)` is a
deletion. As a comparison side a wrapper is the field's value there:
`worktree(status) != staged(status)`. A bare task field is refused at config
load: which state it reads is the one thing its reader could not tell.

`into("s")` means `staged(status == "s") && !head(status == "s")` — the
commit flips the task into `s`. So a start or a close rides with the work it
claims:

```toml
[[commits.rules]]
name    = "flip-with-work"
scope   = "task"
when    = 'into("wip") || into("done")'
assert  = 'named && (!tasks_only || staged(has_tag("decision")))'
message = "start or close a task in a commit that carries its work"
```

A breach names the task after the rule:

```
error: .git/COMMIT_EDITMSG: commits.rules.flip-with-work: parser: start or close a task in a commit that carries its work
```

`when = 'worktree(status) != staged(status)'` catches a flip left unstaged
beside the work it names.

= Checking a range

A hook is one `--no-verify` away from skipped, so CI checks the commits a push
carries again: `mt check --commits <range>` holds every commit `git rev-list`
lists for the range, oldest first, to the same trailer check and rules.

```sh
mt check --commits origin/main..HEAD
```

Each commit reads against its first parent only, a merge included: `subject`
and `refs` come from its message, `files` from what it changes against that
parent, `staged(...)` from its tree and `head(...)` from the parent's. A
trailer resolves against the tasks the commit holds. A past commit has no
working tree, so a rule calling `worktree(...)` or `changed(...)` is skipped,
and said so once. Only a commit with something to report is listed, its
lines indented beneath it:

```
skip: commits.rules.flip-unstaged: reads the working tree, which a past commit has none of
a1b2c3d task: close the parser
  error: commits.rules.flip-with-work: parser: start or close a task in a commit that carries its work
Error: 1 commit(s) broke commit rules
```
