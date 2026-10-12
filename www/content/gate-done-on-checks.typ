#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": own-config, recipe-config

#let meta = (
  route: "/gate-done-on-checks/",
  label: "Gate done on your checks",
  title: "Gate done on your checks",
  summary: "Run your test command when a task closes, and refuse to close it while that command fails.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

An agent marks its task done, and the test suite fails on the code it wrote.
The backlog says the work is finished. The next time you pull, you find it
broken and have to work out which closed task caused it.

= Config

#recipe-config("gate-done-on-checks")

= Behaviour

`mt done` runs the test script. The script fails, so `mt done` writes nothing
and exits non-zero. `--force` closes the task anyway and writes the waiver
into its status note:

#terminal("gate-done-on-checks")

= Deep dive

`[[on.flip]]` binds actions to a flip, the change of status that `mt flip` or
a status command such as `mt done` makes. `into("done")` in `when` matches a
flip into `done`. A `run` action with `gate = true` refuses the flip when its
command exits non-zero, and prints what the command printed.
`mt` runs the command only once you have run `mt allow` in the clone. Before
that, the gate refuses the flip and asks for it.
#link(u("/hooks/"))[Flip hooks] lists the action kinds and the environment a
command runs with.

`--force` lets you or an agent close a task past a failing check. The waiver
stays in the task file, so whoever reviews the change sees it in the diff.
#link(u("/statuses/"))[Statuses] covers the statuses a project allows and the
moves between them.

This repo gates its own closes the same way in #own-config.
