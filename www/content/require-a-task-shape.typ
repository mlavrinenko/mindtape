#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": own-config, pack-excerpt, recipe-config

#let meta = (
  route: "/require-a-task-shape/",
  label: "Require a task shape",
  title: "Require a task shape",
  summary: "Refuse a task that doesn't say where it starts, what done looks like and which files change.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A task says "Paginate the audit log" and nothing else. Whoever picks it up has
to find the bug again, guess what done means and hunt for the files. Guess
wrong and you build the wrong thing.

= Config

#recipe-config("require-a-task-shape", after: pack-excerpt((
  "as-is-is-numbered-steps",
  "to-be-has-green-bullets",
  "changes-name-files",
  "changes-bullets-start-with-a-path",
  "changes-non-empty",
)))

= Behavior

`mt add` refuses a task with sections left out. Fill them in, write `AS-IS` as
one sentence instead of steps, and `mt add` refuses it again:

#terminal("require-a-task-shape")

Neither refusal writes a file. A missing slot exits
#exit-code("require-a-task-shape", step: 0), and a broken rule exits
#exit-code("require-a-task-shape", step: 1).

= Tags

Four tags tune the shape. Run `mt explain tags --for add` to print them with the rule
each one reads:

#table(
  columns: (auto, 1fr),
  table.header([Tag], [What it says]),
  [raw("stateful")],
  [The change keeps state, so the task carries one Rollback section saying how to undo it.],

  [raw("stateless")],
  [The change keeps no state, so the task carries no Rollback.],

  [raw("no-repro")],
  [The `AS-IS` has no runnable repro, so the fenced block is waived.],

  [raw("no-ui-ux")],
  [The `TO-BE` has no screen or output to show, so the fenced example is waived.],
)

The `statefulness` and `examples` parameters turn the tag rule and the two
fenced examples on. Both are off in the example, so a task without one of these
tags still passes.

= Deep dive

The project's templates ask for each section as a slot, a value that `mt add`
fills in. `mt add` refuses a fill that leaves a slot out, and the pack's rules
refuse one left empty. `mt add --describe` lists the slots, each with a prompt
and an example. #link(u("/templates/"))[Templates & slots] covers both.

`extends` adopts the `mindtape:tasks` #link(u("/packs/"))[rule pack], a
set of rules that ships with `mt`. Its rules require `AS-IS`, `TO-BE`, Changes
and Out of scope, in that order. A `fix` also carries Environment, a `feat`
or a `decision` carries Alternatives, a `docs` task carries Source of truth,
and a task tagged `stateful` carries Rollback. `AS-IS` must be numbered steps,
with 🔴 on the step that shows the bug. Each `TO-BE` bullet opens with 🟢, and
each Changes bullet starts with one backticked path. Changes is never empty. A question left in `TO-BE`, Changes or Out of
scope fails, so ask it before you file, while a `?` in code passes. If you edit
a task by hand, `mt check` runs the same rules on it. #link(u("/packs/"))[Packs]
lists every rule in the pack and shows how to drop one, and
#link(u("/rules/"))[Rules] covers body rules.

This repo extends the same pack in #own-config.
