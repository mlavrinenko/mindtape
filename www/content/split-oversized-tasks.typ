#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": own-config, recipe-config

#let meta = (
  route: "/split-oversized-tasks/",
  label: "Split oversized tasks",
  title: "Split oversized tasks",
  summary: "Cap how many source files a task may touch, so a big task gets split before it's filed.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

An agent files "Sign in with Google and GitHub" as one task. The change touches
the config, the routes, the auth module and a migration. The diff is too big to
review, and when a test fails, you can't tell which part broke it. A task that
touches a few files gives you a diff you can read, and a failing test points at
that diff.

= Config

#recipe-config("split-oversized-tasks")

= Behaviour

This task needs 7 source files, so `mt add` refuses it:

#terminal("split-oversized-tasks")

`mt add` exits #exit-code("split-oversized-tasks") and writes no file. An agent
reads the rule's message and files two smaller tasks instead.

= Deep dive

`[tasks.estimate]` allows one unit, `src-files`, and requires an estimate on
every open task. `!terminal` picks the open tasks. A terminal status, such as
`done`, ends a task. #link(u("/statuses/"))[Statuses] says which statuses are
terminal, and #link(u("/estimates/"))[Estimates] lists the other estimate
keys.

The `split-big-work` rule reads the estimate as a number. Its `when` limits it
to open tasks, so a closed task keeps its estimate. With
`severity = "error"`, every command that writes a task refuses one that breaks
the rule. If you edit a task file by hand, `mt check` fails on it instead.
#link(u("/rules/"))[Rules] covers the rule keys, and
#link(u("/filters/"))[Filter language] covers the expressions.

A rule can also compare the estimate with the files git reports changed, so
`mt check` warns you when the work outgrows what the task declared.
#link(u("/estimates/"))[Estimates] shows that budget.

This repo caps its own tasks the same way in #own-config.
