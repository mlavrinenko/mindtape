#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/problems.typ": own-config, problem-config

#let meta = (
  route: "/require-a-task-shape/",
  label: "Require a task shape",
  title: "Tasks too vague to start",
  summary: "Refuse a task that doesn't say how to reproduce the problem, what done looks like and which files change.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A task says "Paginate the audit log" and nothing else. Whoever picks it up,
you or an agent, has to find the bug again, guess what done means and hunt for
the files. An agent that guesses wrong builds the wrong thing and reports
success.

= The config

#problem-config("require-a-task-shape")

= What `mt` does

`mt add` refuses a task with sections left empty. Fill them in, write the
Problem as one sentence instead of steps, and `mt add` refuses it again:

#terminal("require-a-task-shape")

`mt add` exits #exit-code("require-a-task-shape") and writes no file.

= How it works

The project's template asks for each section as a slot, a value that `mt add`
fills in. `mt add` refuses a fill that leaves a slot empty.
`mt add --describe` lists the slots, so an agent can learn the shape before it
writes a task. #link(u("/templates/"))[Templates & slots] covers both.

`extends` adopts the `mindtape-tasks` #link(u("/rule-packs/"))[rule pack], a
set of rules that ships with `mt`. Its rules require Problem, TO-BE, Changes
and Out of scope, in that order. The Problem must be numbered steps, with 🔴
on the step that shows the bug. Each TO-BE bullet opens with 🟢, and each
Changes bullet names a file. If you edit a task by hand, `mt check` runs the
same rules on it. #link(u("/rule-packs/"))[Rule packs] lists every rule in the
pack and shows how to drop one, and #link(u("/rules/"))[Rules] covers body
rules.

This repo extends the same pack in #own-config.
