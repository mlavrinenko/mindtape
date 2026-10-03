#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/problems.typ": own-config, problem-config

#let meta = (
  route: "/gate-done-on-checks/",
  label: "Gate done on your checks",
  title: "Tasks closed while the tests fail",
  summary: "Run your test command when a task closes, and refuse to close it while that command fails.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

An agent marks its task done, and the test suite fails on the code it wrote.
The backlog says the work is finished. The next person to pull finds it broken
and has to work out which closed task caused it.

= The config

#problem-config("gate-done-on-checks")

= What `mt` does

`mt done` runs the test script, which fails, so `mt done` writes nothing and
exits non-zero. `--force` closes the task anyway and writes the waiver into
its status note:

#terminal("gate-done-on-checks")

= How it works

`[[on.flip]]` binds actions to a change of status. `into("done")` in `when`
matches a flip into `done`. A `run` action with `gate = true` refuses the flip
when its command exits non-zero, and prints what the command printed.
#link(u("/hooks/"))[Flip hooks] lists the action kinds and the environment a
command runs with.

`--force` lets a person or an agent close a task past a failing check. The
waiver stays in the task file, so a reviewer sees it in the diff.
#link(u("/statuses/"))[Statuses] covers the legend and its transitions.

This repo gates its own closes the same way in #own-config.
