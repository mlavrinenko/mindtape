#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": own-config, recipe-config

#let meta = (
  route: "/trace-commits-to-tasks/",
  label: "Trace commits to tasks",
  title: "Trace commits to tasks",
  summary: "Refuse a commit that names no task, then ask which tasks changed a file before you edit it.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`git blame` shows you a line and the subject "fix(auth): reject expired
sessions". Nothing says which task asked for the change, what problem it fixed
or what it left out. An agent about to edit the file has the same question and
no way to answer it.

= Config

#recipe-config("trace-commits-to-tasks")

= Behavior

Add a `commit-msg` hook that runs `mt check --commit-msg`. From then on git
refuses a commit without a `Refs:` trailer and accepts it with one, and
`touches()` finds the task behind the changed file:

#terminal("trace-commits-to-tasks")

= Deep dive

`[commits] trailer` names the trailer that holds task stems. A stem is the
task's file name without `.typ`, as #link(u("/concepts/#identity"))[Concepts]
explains. The project also extends the `mindtape-tasks`
#link(u("/rule-packs/"))[rule pack], as
#link(u("/require-a-task-shape/"))[Require a task shape] shows. The pack's
`work-names-a-task` rule refuses a commit that names no task, unless the
commit changes only task files or starts with `fixup!`, `squash!` or `amend!`.
`mt check --commit-msg` also refuses a stem
that no task has. #link(u("/commit-rules/"))[Commit rules] covers the rule
fields, and #link(u("/rule-packs/"))[Rule packs] shows how to drop the rule.

Anyone can skip a hook with `--no-verify`, so run `mt check --commits` in CI
on the commits a push carries. #link(u("/commit-rules/"))[Commit rules] shows
how.

`touches("<path>")` matches each task named in the trailer of a commit that
changed the path. #link(u("/filters/"))[Filter language] covers it.

This repo checks every commit message against #own-config.
