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

Someone files "Sign in with Google and GitHub" as one task. The change touches
the config, the routes, the auth module and a migration. The diff is too big to
review, and when a test fails, you can't tell which part broke it. A task that
touches a few files gives you a diff you can read, and a failing test points at
that diff.

A rename is different. It touches every file that names the type, but it's one
decision, and splitting it only copies that decision into several tasks. So
the cap only applies to work that takes judgment.

= Config

The `changes` collector reads the first backticked path on every Changes
bullet, so `derived.changes` is the plan the task already carries.
`judgment-stays-small` counts the paths that fall inside the `source` set:

// docs:hand-typed the `[dicts.source]` block that makes `@source` resolve
#recipe-config(
  "split-oversized-tasks",
  after: "[dicts.source]\ncase = \"exact\"\nvalues = [\"**\"]",
)

= Behavior

The sign-in task takes judgment and its Changes section names 7 source files,
so `mt add` refuses it. The rename names its 12 files through one glob,
`src/**/*.rs`, and is filed: a glob counts once, and mechanical work has no
cap.

#terminal("split-oversized-tasks")

The refused `mt add` exits #exit-code("split-oversized-tasks") and writes no
file.

= Deep dive

`difficulty` rates how much judgment a task takes, from 0 for a mechanical edit
to 9 for an open question. The template asks for it and fills in 3 when
nobody answers, so a task filed without a rating gets the cap. #link(u("/priority/#difficulty"))[Priority]
covers the scale.

The `judgment-stays-small` rule reads
`count_paths(derived.changes, "@source")`: the distinct paths in the Changes
section that fall inside the project's `source` path set. Its `when` picks the
open tasks rated 3 and up, so a closed task is not collected and a mechanical
one has no cap. Want a ladder? Add a rule for each rung, such as
`difficulty >= 6` with a cap of 2. With `severity = "error"`, every command
that writes a task refuses one that breaks a rule. If you edit a task file by
hand, `mt check` fails on it instead. #link(u("/rules/"))[Rules] covers the
rule keys, and #link(u("/filters/"))[Filter language] covers the expressions.

`[tasks.estimate]` allows one unit, `src-files`, and leaves it optional. The
unit stays so a task filed with `estimate.src-files` remains valid, while a
new task declares its plan once, in Changes. Every Changes bullet starts
with one backticked path, and a glob counts once. A terminal status, such as
`done`, ends a task. #link(u("/statuses/"))[Statuses] says which statuses are
terminal, and #link(u("/estimates/"))[Estimates] lists the other estimate
keys.

The budget rule in `mindtape:budget` compares the files git reports changed
with the same count, so `mt check` warns you when the work outgrows the plan.
#link(u("/estimates/"))[Estimates] shows that budget.

This repo caps its own tasks the same way in #own-config.
