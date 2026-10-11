#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/hooks/",
  label: "Flip hooks",
  title: "Flip hooks",
  summary: "How `mt flip` runs `[[on.flip]]` hooks that print a message, list tasks, or run a command that can refuse the flip.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The run-less gate is a slice of this repo's own .mindtape/config.toml.
#let own = checked-slices(
  json("../generated/hooks-config.json"),
  "hooks-config",
  ("stale-gate",),
)

Want the tests to pass before a task can close, or a reminder when work
starts? `[[on.flip]]` runs actions when `mt flip` changes a task's status. An
action prints a message, lists tasks or runs a command. A command marked as a
gate can refuse the flip.
#link(u("/gate-done-on-checks/"))[Tasks closed while the tests fail] shows a
gate that runs the tests before `mt done`, and `--force` closing the task past
it.

= When a hook fires

`when` is a #link(u("/filters/"))[filter expression] over the task as it
stands before the write, so `status == "wip"` matches a task leaving `wip`.
`into("name")` matches a flip that stamps the status `name`. An empty `when`
matches every flip. `mt` checks each `into()` against the
#link(u("/statuses/"))[status legend] when it loads the config.

= Actions

`actions` is a list, and each action sets one kind:

#table(
  columns: 2,
  table.header([Action], [Effect]),
  [`{ show = "text" }`], [Prints the text.],
  [`{ list = "filter", title = "..." }`],
  [Prints the matching tasks as a table under `title`. A bare `@view` also brings that #link(u("/named-sets/#views"))[view]'s sort.],

  [`{ run = "command" }`],
  [Runs the command. If it fails, `mt` prints its output and a warning, and the flip goes ahead.],

  [`{ run = "command", gate = true }`],
  [Runs the command. A non-zero exit refuses the flip.],

  [`{ gate = true }`], [Refuses every flip the hook's `when` matches.],
)

`message` replaces the advice `mt` prints when a gate refuses or a command
fails. `title` goes with `list`, and `gate` and `message` go with `run`.
Actions run in the order written. A gate that refuses stops the actions after
it, and `mt` writes nothing for that task.

`show`, `list` and the output of a passing command print only when stdout is a
terminal and `--quiet` isn't set, so a scripted flip prints only the flip. A
failing command prints its output either way.

= Gates

A gate with no `run` refuses whenever `when` matches, so `when` is the whole
check. This repo refuses to start a task that nobody has reviewed or moved in
30 days:

#raw(
  render-slices(
    own,
    (("stale-gate", "refuse to start a task untouched for 30 days"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

`--force` takes a flip past a failing gate, with or without `run`, and
records that in the status note: the command, or the `message` of a gate with
no `run`. So the task file shows which check was skipped.

A gate command runs from the project root with three variables set:

#table(
  columns: 2,
  table.header([Variable], [Value]),
  [`MINDTAPE_TASK_FILE`], [The absolute path of the task file.],
  [`MINDTAPE_TASK_STEM`],
  [The task's id: its file stem, unless `id:` sets one.],

  [`MINDTAPE_TASK_STATUS`], [The status being stamped.],
)

= Trust and portability

#callout("warn")[
  A `run` command is as trusted as a `Justfile` recipe or a git hook. It comes
  from the project's config and runs with your permissions, through `sh -c`,
  or `cmd /C` on Windows. Nothing sandboxes it. Read a project's config before
  you run `mt flip` in it.
]

The two shells share no syntax. If you use the project on both, write a
command both accept, or call a script.

Only `mt flip` and its status commands, such as `mt done`, run actions.
`mt check`, `mt ls` and the other read commands run none. The one command a
read runs is a #link(u("/wait-on-upstream/"))[`waits-on` checker]. A hook whose
`when` doesn't parse, whose `into()` names no status, or whose action sets no
kind is a config error, so every command fails until you fix it.
