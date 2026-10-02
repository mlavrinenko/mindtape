#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/self-validating-tasks/",
  label: "Self-validating tasks",
  title: "Self-validating tasks",
  summary: "How a project adds its own field to tasks and checks it with a Typst function on every `mt check` and every write.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Your project can give its tasks a field MindTape does not know, and check that
field with a Typst function you write. `mt check` runs the function on every
task, and every command that writes a task runs it before writing. You need no
separate linter, and the rule is a file in the repo, next to the tasks.

Every block below is a slice of `examples/self-validating/`, a project with two
tasks.

#let code = checked-slices(
  json("../generated/self-validating.json"),
  "self-validating",
  ("goals", "check-goals", "ship-onboarding-checklist", "export-report-as-csv"),
)
#let config = checked-slices(
  json("../generated/self-validating-config.json"),
  "self-validating-config",
  ("goals-rule",),
)

= Write the function

The project lists its goals in a Typst module. Each task says how it serves
them in an `x-goals` field. MindTape keeps any field named `x-<name>` as it is
and checks nothing in it.

#raw(
  render-slices(code, (("goals", "the goals every task must address"),)),
  lang: "typ",
  block: true,
)

The check is an ordinary Typst function. It receives the field's value, or
`none` when the task has no such field. It returns one string per problem, and
an empty array when the task passes.

#raw(
  render-slices(code, (
    ("check-goals", "one breach per goal with no verdict"),
  )),
  lang: "typ",
  block: true,
)

= Turn it on

A `[[tasks.rules]]` entry names the module, the function and the field to pass
it. #link(u("/rules/"))[Rules] describes each key.

#raw(
  render-slices(
    config,
    (("goals-rule", "run check-goals on each task's x-goals"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

= See it refuse

This task gives a verdict for both goals:

#raw(
  render-slices(code, (("ship-onboarding-checklist", "passes"),)),
  lang: "typ",
  block: true,
)

This one says nothing about `no-lock-in`:

#raw(
  render-slices(code, (("export-report-as-csv", "fails"),)),
  lang: "typ",
  block: true,
)

`mt check` names the task, the line and the goal it skipped:

#terminal("self-validating-check")

It exits with code
#exit-code("self-validating-check"), so a CI job
or a pre-commit hook that runs it fails. `mt add`, `mt flip` and the other
write commands run the same function on the task they are about to write, and
refuse to write a task that fails it.
