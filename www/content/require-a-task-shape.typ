#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": own-config, pack-excerpt, recipe-config

#let meta = (
  route: "/require-a-task-shape/",
  label: "Require a task shape",
  title: "Require a task shape",
  summary: "Refuse a task that doesn't say how to reproduce the problem, what done looks like and which files change.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A task says "Paginate the audit log" and nothing else. Whoever picks it up has
to find the bug again, guess what done means and hunt for the files. Guess
wrong and you build the wrong thing.

= Config

#recipe-config("require-a-task-shape", after: pack-excerpt((
  "problem-is-numbered-steps",
  "to-be-has-green-bullets",
  "changes-name-files",
)))

= Behavior

`mt add` refuses a task with sections left out. Fill them in, write the
Problem as one sentence instead of steps, and `mt add` refuses it again:

#terminal("require-a-task-shape")

Neither refusal writes a file. A missing slot exits
#exit-code("require-a-task-shape", step: 0), and a broken rule exits
#exit-code("require-a-task-shape", step: 1).

= Deep dive

The project's templates ask for each section as a slot, a value that `mt add`
fills in. `mt add` refuses a fill that leaves a slot out, and the pack's rules
refuse one left empty. `mt add --describe` lists the slots, each with a prompt
and an example. #link(u("/templates/"))[Templates & slots] covers both.

`extends` adopts the `mindtape-tasks` #link(u("/rule-packs/"))[rule pack], a
set of rules that ships with `mt`. Its rules require Problem, TO-BE, Changes
and Out of scope, in that order, plus one more section for some kinds, like
Environment for a `fix`. The Problem must be numbered steps, with 🔴
on the step that shows the bug. Each TO-BE bullet opens with 🟢, and each
Changes bullet names a file. If you edit a task by hand, `mt check` runs the
same rules on it. #link(u("/rule-packs/"))[Rule packs] lists every rule in the
pack and shows how to drop one, and #link(u("/rules/"))[Rules] covers body
rules.

This repo extends the same pack in #own-config.
