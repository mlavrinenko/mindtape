#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/hooks/",
  label: "Flip hooks",
  title: "Flip hooks",
  summary: "How a project binds actions to a task changing status: printing a message, listing what else is in flight, or running a command that may refuse the flip.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Both blocks below are slices of this repo's own .mindtape/config.toml — the
// hooks MindTape actually flips its own backlog under, not a shape retyped
// for the page.
#let parts = (
  (
    "run-gate",
    "a command the flip must pass before any terminal status is stamped",
  ),
  ("stale-gate", "the same mechanism with no command to run at all"),
)

#let slices = checked-slices(
  json("../generated/hooks-config.json"),
  "hooks-config",
  parts.map(p => p.at(0)),
)

`[[on.flip]]` binds actions to a task changing status. One table, three action
kinds — `show`, `list` and `run`, tabulated below. This repo's own config binds
the last of them to closing a task:

#raw(
  render-slices(slices, parts.slice(0, 1), comment: "#"),
  lang: "toml",
  block: true,
)

`when` is a #link(u("/filters/"))[filter expression] over the task *as it stands
before the write* — so `status == "wip"` names the status being left — plus the
`into("name")` predicate for the status being stamped. An action runs only on a
transition its `when` matches. An empty `when` fires on every flip.

= Actions

Each action declares exactly one kind:

#table(
  columns: 2,
  table.header([Action], [Effect]),
  [`{ show = "text" }`], [Print `text`.],
  [`{ list = "filter or @view", title = "..." }`],
  [Print the matching tasks as a table under `title`. A bare `@view` also picks up that view's sort.],

  [`{ run = "cmd", gate = false, message = "..." }`],
  [Execute `cmd`. With `gate = true` a non-zero exit *refuses the flip*, prints the command's output, and stops: nothing is written for that task and the actions after it do not run. Without `gate` the output is informational. `message` replaces the default advice on failure.],
)

= Gates

`run` is optional on a gate: `{ gate = true, message = "..." }` with no `run`
refuses the flip unconditionally whenever the hook's `when` already matched —
no command runs, so there is nothing to print but the message. The condition
lives entirely in `when`; a gate with a `run` command is for a check `when`
cannot express itself, such as an external command's exit code. An action with
no `run` and no `gate = true` — nothing to show, list, run, or refuse — is a
config error at load, naming the action's position.

#raw(
  render-slices(slices, parts.slice(1, 2), comment: "#"),
  lang: "toml",
  block: true,
)

Actions run in declaration order. `--force` waives a failing gate and records
the waiver in the status note — for the hook above,
`forced past failing gate: "mmz --is-fresh --tag gate"`. A
gate command runs from the project root with the transition in its
environment:

#table(
  columns: 2,
  table.header([Variable], [Value]),
  [`MINDTAPE_TASK_FILE`], [Absolute path of the task file being flipped.],
  [`MINDTAPE_TASK_STEM`],
  [The task's identity (its file stem, unless `id:` overrides).],

  [`MINDTAPE_TASK_STATUS`], [The status being stamped.],
)

`show` and `list` output is suppressed under `--quiet` and whenever stdout is
not a terminal, so a scripted flip stays clean. Gates still run, and a failing
one still prints its output.

#callout("warn")[
  *Trust boundary.* An `[[on.flip]]` command is exactly as trusted as a
  `Justfile` recipe or a git hook: it comes from the repository's own config and
  runs through the platform shell — `sh -c`, or `cmd /C` on Windows — with your
  full authority, from the project root. Nothing sandboxes it. Read a project's
  config before running `mt flip` under it, the same way you read a `Makefile`
  before running `make`.
]

A `run` command is shell text, not an argv, and the two shells share no syntax,
so a command written for one platform is not portable to the other. A project
targeting both writes a line both shells accept, or calls a script.

Only `mt flip` runs actions. `mt check`, `mt ls` and every other read command
execute nothing. A malformed hook (an unparseable `when`, an `into(...)` naming
no configured status, an action with no kind) is a config error at load, so it
fails every command equally rather than surprising the first flip that reaches
it.
