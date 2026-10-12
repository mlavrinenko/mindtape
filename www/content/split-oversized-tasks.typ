#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": own-config, recipe-config

#let meta = (
  route: "/split-oversized-tasks/",
  label: "Split oversized tasks",
  title: "Split oversized tasks",
  summary: "Cap how many source files a task may touch when it takes judgment, so a big design gets split before it's filed.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

An agent files "Sign in with Google and GitHub" as one task. The change touches
the config, the routes, the auth module and a migration. The diff is too big to
review, and when a test fails, you can't tell which part broke it. A task that
touches a few files gives you a diff you can read, and a failing test points at
that diff.

A rename is different. It touches every file that names the type, but it's one
decision, and splitting it only copies that decision into several tasks. So
the cap gets tighter as the task gets harder.

= Config

#recipe-config("split-oversized-tasks")

= Behaviour

The sign-in task takes judgment and needs 7 source files, so `mt add` refuses
it. The rename needs 12, and it's filed:

#terminal("split-oversized-tasks")

The refused `mt add` exits #exit-code("split-oversized-tasks") and writes no
file. An agent reads the rule's message and files two smaller tasks instead.

= Deep dive

`difficulty` rates how much judgment a task takes, from 0 for a mechanical edit
to 9 for an open question. The template asks for it and fills in 3 when
nobody answers, so a task filed without a rating gets the cap. #link(u("/priority/#difficulty"))[Priority]
covers the scale.

The `judgment-stays-small` rule reads the estimate as a number. Its `when`
picks the open tasks rated 3 and up, so a closed task keeps its estimate and a
mechanical one has no cap. Want a ladder? Add a rule for each rung, such as
`difficulty >= 6` with a cap of 2. With `severity = "error"`, every command
that writes a task refuses one that breaks a rule. If you edit a task file by
hand, `mt check` fails on it instead. #link(u("/rules/"))[Rules] covers the
rule keys, and #link(u("/filters/"))[Filter language] covers the expressions.

`[tasks.estimate]` allows one unit, `src-files`, and requires an estimate on
every open task. `!terminal` picks the open tasks. A terminal status, such as
`done`, ends a task. #link(u("/statuses/"))[Statuses] says which statuses are
terminal, and #link(u("/estimates/"))[Estimates] lists the other estimate
keys.

A rule can also compare the estimate with the files git reports changed, so
`mt check` warns you when the work outgrows what the task declared.
#link(u("/estimates/"))[Estimates] shows that budget.

This repo caps its own tasks the same way in #own-config.
