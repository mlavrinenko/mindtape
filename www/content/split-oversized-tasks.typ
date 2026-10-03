#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/problems.typ": own-config, problem-config

#let meta = (
  route: "/split-oversized-tasks/",
  label: "Split oversized tasks",
  title: "Tasks too big to finish in one change",
  summary: "Cap how many source files a task may touch, so whoever files a big task has to split it first.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

An agent files "Sign in with Google and GitHub" as one task. The change
touches the config, the routes, the auth module and a migration, and the diff
is too big to review. When a test fails, nobody can tell which part broke it. A task that touches a few files gives a diff a
reviewer can read, and a failing test points at that diff.

= The config

#problem-config("split-oversized-tasks")

= What `mt` does

The task below needs 7 source files, and `mt add` refuses it:

#terminal("split-oversized-tasks")

`mt add` exits #exit-code("split-oversized-tasks") and writes no file. An agent
reads the rule's message and files two smaller tasks.

= How it works

`[tasks.estimate]` allows one unit, `src-files`, and requires an estimate on
every open task. #link(u("/estimates/"))[Estimates] lists the other keys.

The `split-big-work` rule reads the estimate as a number. Its `when` limits it
to open tasks, so a closed task keeps its estimate. At `severity = "error"`
every command that writes a task refuses a task that breaks it, and `mt check`
fails on a task file edited by hand. #link(u("/rules/"))[Rules] covers the rule
keys, and #link(u("/filters/"))[Filter language] covers the expressions.

A rule can also compare the estimate with the files git reports changed, so
`mt check` warns when the work outgrows what the task declared. The
#link(u("/estimates/"))[Estimates] page shows that budget.

This repo caps its own tasks the same way in #own-config.
