#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/hooks/",
  label: "Hooks",
  title: "Hooks",
  summary: "`[[on.flip]]` hooks gate or report a status change, and `[[on.add]]` hooks fill fields and choose where a new task lands.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The run-less gate is a slice of the `mindtape:flow` pack file.
#let own = checked-slices(
  json("../generated/hooks-config.json"),
  "hooks-config",
  ("stale-gate",),
)

`[[on.flip]]` runs actions when `mt flip` changes a task's status. An action
prints a message, lists tasks or runs a command. A command marked as a
gate can refuse the flip. `[[on.add]]` runs actions when `mt add` files a
task; Add hooks below covers it.

= When a hook fires

`when` is a #link(u("/filters/"))[filter expression] over the task as it
stands before the write, so `status == "wip"` matches a task leaving `wip`.
`into("name")` matches a flip that stamps the status `name`. An empty `when`
matches every flip. `mt` checks each `into()` against the
#link(u("/statuses/"))[status legend] when it loads the config.

= Name a hook

A hook may have a `name`. Hooks add up across a project, the config layers
above it and the #link(u("/packs/"))[packs] it adopts, and a hook that has a
name can be changed from a nearer layer. A later hook with the same `name`
replaces the earlier one at that position, and `disable = ["<name>"]` drops it.
A hook with no `name` always joins the list and nothing replaces it.
`mt config show --origin` labels a named hook `on.flip.<name>` or
`on.add.<name>`, and the others `on.flip[<index>]` or `on.add[<index>]`.
#link(u("/cli/explain/#rules"))[`mt explain rules`] lists flip hooks, and
#link(u("/cli/explain/#add-hooks"))[`mt explain add`] lists add hooks. Two flip
hooks in one file can't share a name, and neither can two add hooks.

= Actions

`actions` is a list, and each action sets one kind:

#table(
  columns: 2,
  table.header([Action], [Effect]),
  [`{ show = "text" }`], [Prints the text.],
  [`{ list = "filter", title = "..." }`],
  [Prints the matching tasks as a table under `title`. A bare `@view` also applies that #link(u("/named-sets/#views"))[view]'s sort.],

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

A gate with no `run` refuses whenever `when` matches, so `when` decides on its
own. The `mindtape:flow` pack refuses to start a task that no one has
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
above it. A `done` gate that runs `mt done` on its own task would fire itself
repeatedly. So `mt` refuses the inner flip, names every flip in the chain, and
exits 1.
Two hooks that close each other end the same way, on the second visit to the
first task. A gate that runs it fails like any failing gate, so the outer
flip is refused too, and `--force` doesn't lift the refusal on the inner one.

A chain also stops at 16 nested flips, which catches a hook that keeps adding
fresh tasks and flipping them.

= Add hooks

`[[on.add]]` declares defaults for a new task. An action sets fields, appends
to list fields, or chooses where the task's file goes:

// docs:hand-typed the example is the shape, not a run: no fixture has a fix template
```toml
[[on.add]]
name = "fixes"
when = 'template == "fix"'
actions = [{ set = { due = "2026-12-01" }, dir = "tasks/fixes" }]
```

`set` writes writable field names, `append` adds items to a list field, and a
`derive` returns a field's value. One action cannot name the same field in more
than one of the three. `set.dir` is refused: `dir` is a destination, not a
field. `when` is a #link(u("/filters/"))[filter expression] over the new task.
`template == "name"`, `template != "name"` and `from_template("name")` test
the template `mt add` used, and the name must be a known template. A bare
`template`, another operator, a non-literal name and an unknown template are
config errors. An empty `when` fires on every add.

`dir` is a project-root-relative template. `{template}` expands to the template
name, and `{title}`, `{priority}`, `{difficulty}`, `{kind}`, `{assignee}`,
`{due}` and `{start}` expand to the task's fields, after the action's `set` and
`append`. A macro's value must be one path component, so a title with a `/` is
refused when the task is added. The literal `""` is the project root, and a
hooked add refuses it unless a configured root covers the project root. An
unknown macro, an absolute path, or an empty, `.` or `..` component is a config
error. A hook `dir` outside every configured root is refused at add time once
the hook fires, and so is an explicit `--dir` outside every root when hooks are
enabled.

Add hooks merge across packs and layers like `[[on.flip]]`. A later hook with
the same `name` replaces the earlier one in place, `disable` drops it by name,
and an unnamed hook appends. #link(u("/cli/explain/#add-hooks"))[`mt explain add`]
lists the hooks in effect, with the pack or layer that supplied each one.

Hooks run in declaration order, and each `when` sees the task as the earlier
hooks left it. An explicit `--set field=…`, a positional title and a value
typed in `--edit` win over a hook `set` on the same field; an explicit `+=`
does not, and hook appends accumulate. A later `set` replaces the draft value,
so it drops the appends before it. An explicit `--dir` wins over a hook `dir`,
which is then not expanded at all.

An action may derive a value instead of writing a literal, and a `dir` may be
derived too:

// docs:hand-typed the shape of a project helper no fixture here ships
```toml
[[on.add]]
name = "fixes"
when = 'template == "fix"'
actions = [
  { derive = { typst = "add.typ", function = "fix-tags", field = "tags" } },
  { dir = { derive = { typst = "add.typ", function = "fix-dir" } } },
]
```

The function receives the task as it stands, minus every field the draft cannot
answer (its `id` and `file`, its `path` and `dir`, the graph facts and the
derived values), plus `template`. A field value that does not fit the field, or
a derived `dir` outside every root, is refused.

A `when` reads the same draft, so it refuses those unsafe fields too. It also
refuses `ready()`, `blocked()` and `count(...)`: their answer depends on the
filed set, which does not include the task yet. An `old("field")` in `when`
reads the task before any hook ran, so a later hook can test what an earlier
one changed. It is read-only, works wherever a value is expected
(`matches(old("title"), …)` too), and exists only in an `[[on.add]]` `when`.
It refuses every field the draft cannot answer, including `id`, `path`, `dir`,
`file`, `depth`, `topo`, `blocks`, `children`, `progress` and every
`derived.<name>`.

`mt add` prints each action it applies on stderr, naming the hook. Pass
`--no-hooks` to skip every hook; explicit `--set` values still apply. A
`[[tasks.rules]]` assertion on `dir` can hold the layout a hook chose, and
`mt add` checks it before it writes.

= Trust and portability

#callout("warn")[
  A flip hook's `run` command is as trusted as a `Justfile` recipe or a git
  hook. It comes from the project's config and runs with your permissions. A
  string runs through `sh -c`, or `cmd /C` on Windows, and an array runs its
  program directly. Nothing sandboxes it. So `mt` runs none until you
  read the config's commands and run #link(u("/cli/allow/"))[`mt allow`] in
  the repo.
]

Until then a gate with `run` refuses the flip and names `mt allow`, and
`--force` takes the flip past it as it would a failing gate. A `run` without
`gate` is skipped, and `mt` prints that it was. Either way `mt` warns once and
lists the commands it did not run. A gate with no `run` runs nothing, so it needs
no approval.

The two shells share no syntax. If you use the project on both, write the
command as an array, such as `run = ["cargo", "test"]`. `mt` starts the
program itself and passes it each item as one argument, the same on every
platform. The program has to be one the platform can start: an executable on
`PATH` or a path to one, not a shell script on Windows. `mt allow` lists an
array as written, and turning a string into an array, or back, asks again.

A flip hook's actions run only when `mt flip` or one of its status commands,
such as `mt done`, changes a status. An add hook's actions run only on
`mt add`. `mt check`, `mt ls` and the other read commands run neither. They
run one command: a #link(u("/wait-on-upstream/"))[`waits-on` checker]. A flip
hook whose `when` doesn't parse, whose `into()` names no status, or whose
action sets no kind is a config error, so every command fails until you fix
it.
