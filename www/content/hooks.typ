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

// The run-less gate is a slice of the `mindtape:flow` pack file.
#let own = checked-slices(
  json("../generated/hooks-config.json"),
  "hooks-config",
  ("stale-gate",),
)

Want the tests to pass before a task can close, or a reminder when work
starts? `[[on.flip]]` runs actions when `mt flip` changes a task's status. An
action prints a message, lists tasks or runs a command. A command marked as a
gate can refuse the flip.

= When a hook fires

`when` is a #link(u("/filters/"))[filter expression] over the task as it
stands before the write, so `status == "wip"` matches a task leaving `wip`.
`into("name")` matches a flip that stamps the status `name`. An empty `when`
matches every flip. `mt` checks each `into()` against the
#link(u("/statuses/"))[status legend] when it loads the config.

= Name a hook

A hook may carry a `name`. Hooks add up across a project, the config layers
above it and the #link(u("/packs/"))[packs] it adopts, and a hook that has a
name can be changed from a nearer layer. A later hook with the same `name`
replaces the earlier one where it stood, and `disable = ["<name>"]` drops it.
A hook with no `name` always joins the list and nothing replaces it.
`mt config show --origin` and #link(u("/cli/rules/"))[`mt rules`] label a
named hook `on.flip.<name>`, and the others `on.flip[<index>]`. Two hooks in
one file can't share a name.

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

  [`{ run = ["cargo", "test"] }`],
  [Runs the program with those arguments and no shell. It takes `gate` and `message` like a string `run`.],

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
check. The `mindtape:flow` pack refuses to start a task that nobody has
reviewed or moved in 30 days, the default of its `days` parameter:

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

A `run` command, gate or not, runs from the project root with these
variables set:

#table(
  columns: 2,
  table.header([Variable], [Value]),
  [`MINDTAPE_TASK_FILE`], [The absolute path of the task file.],
  [`MINDTAPE_TASK_STEM`],
  [The task's id: its file stem, unless `id:` sets one.],

  [`MINDTAPE_TASK_STATUS`], [The status being stamped.],
  [`MINDTAPE_FLIP_CHAIN`],
  [The flips in progress, this one last, one per line. `mt` reads it, so leave it be.],
)

An array gets no shell, so `$MINDTAPE_TASK_FILE` in one of its items stays as
written. The program reads the variable itself.

= A hook that flips a task

A command may run `mt flip` itself, say to close a parent once its last
child is done. That nested flip is a flip like any other, so the other task's
own hooks and gates run too.

What it can't do is flip a task into a status that's already in progress
above it. Picture a gate on `done` that runs `mt done` on its own task. The
inner flip would fire the same gate, which flips again, and so on forever.
So `mt` refuses the inner flip, names every flip in the chain, and exits 1.
Two hooks that close each other end the same way, on the second visit to the
first task. A gate that runs it fails like any failing gate, so the outer
flip is refused too, and `--force` doesn't lift the refusal on the inner one.

A chain also stops at 16 nested flips, which catches a hook that keeps adding
fresh tasks and flipping them.

= Trust and portability

#callout("warn")[
  A `run` command is as trusted as a `Justfile` recipe or a git hook. It comes
  from the project's config and runs with your permissions. A string runs
  through `sh -c`, or `cmd /C` on Windows, and an array runs its program
  directly. Nothing sandboxes it. So `mt` runs none until you
  read the config's commands and run #link(u("/cli/allow/"))[`mt allow`] in
  the repo.
]

Until then a gate with `run` refuses the flip and names `mt allow`, and
`--force` takes the flip past it as it would a failing gate. A `run` without
`gate` is skipped, and `mt` prints that it was. Either way `mt` warns once and
lists the commands it held back. A gate with no `run` runs nothing, so it needs
no approval.

The two shells share no syntax. If you use the project on both, write the
command as an array, such as `run = ["cargo", "test"]`. `mt` starts the
program itself and hands it each item as one argument, the same on every
platform. The program has to be one the platform can start: an executable on
`PATH` or a path to one, not a shell script on Windows. `mt allow` lists an
array as written, and turning a string into an array, or back, asks again.

Only `mt flip` and its status commands, such as `mt done`, run actions.
`mt check`, `mt ls` and the other read commands run none. The one command a
read runs is a #link(u("/wait-on-upstream/"))[`waits-on` checker]. A hook whose
`when` doesn't parse, whose `into()` names no status, or whose action sets no
kind is a config error, so every command fails until you fix it.
