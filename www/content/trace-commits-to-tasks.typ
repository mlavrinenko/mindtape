#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/problems.typ": own-config, problem-config

#let meta = (
  route: "/trace-commits-to-tasks/",
  label: "Trace commits to tasks",
  title: "Commits nobody can trace to a task",
  summary: "Refuse a commit that names no task, then ask which tasks changed a file before you edit it.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`git blame` shows a line and the subject "fix(auth): reject expired sessions".
Nothing says which task asked for the change, what problem it fixed or what it
left out. An agent about to edit the file has the same question and no way to
answer it.

= The config

#problem-config("trace-commits-to-tasks")

= What `mt` does

A `commit-msg` hook runs `mt check --commit-msg`. git refuses the commit
without a `Refs:` trailer and accepts it with one. `touches()` then finds the
task behind the changed file:

#terminal("trace-commits-to-tasks")

= How it works

`[commits] trailer` names the trailer whose values are task stems. The project
also extends the `mindtape-tasks` rule pack, as
#link(u("/require-a-task-shape/"))[Tasks too vague to start] shows. The pack's
`work-names-a-task` rule refuses a commit that names no task, unless the commit
changes only task files.
`mt check --commit-msg` also refuses a stem that no task has.
#link(u("/commit-rules/"))[Commit rules] covers the rule fields, and
#link(u("/rule-packs/"))[Rule packs] shows how to drop the rule.

A hook can be skipped with `--no-verify`, so CI runs `mt check --commits` on
the commits a push carries. #link(u("/commit-rules/"))[Commit rules] shows it.

`touches("<path>")` matches each task named in the trailer of a commit that
changed the path. #link(u("/filters/"))[Filter language] covers it.

This repo checks every commit message against #own-config.
