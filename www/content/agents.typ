#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": REPO, u

#let meta = (
  route: "/agents/",
  label: "For AI agents",
  title: "For AI agents",
  summary: "How a coding agent reads tasks as JSON, files tasks without prompts and works in its own git worktree.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A coding agent drives MindTape through the same `mt` commands you use. Tasks
are files in your repo, so an agent reads and edits them with the tools it
already has, and `mt check` tells it when an edit breaks a rule. The examples
below run against
`examples/api-service/`, a backend team's project.

= Give your agent the skill

The MindTape repository ships an agent skill in
#link(REPO + "/tree/main/skills/mindtape")[`skills/mindtape/`]. It tells an
agent which `mt` command does what, and when to use it. Copy that directory
into the directory your agent loads skills from.

= Read tasks as JSON

Every read command prints JSON with `--json`: `mt ls`, `mt eval <task>`,
`mt config show`, `mt worktrees` and `mt add --describe=json`. Here is
`mt ls ready -n 1 --json`:

#raw(read("../generated/api-service-ls-ready.json"), lang: "json", block: true)

`-n` keeps the reply short. `truncated` says how many tasks matched and where
the next page starts, so `mt ls ready -n 1 --skip 1 --json` returns the next
task. `truncated` is `null` when the list is complete. An agent that meets a
`schema_version` it does not know should stop rather than guess.
#link(u("/schema/"))[The JSON schema] describes every field.

`mt _cli --json` prints every command with its flags, defaults and accepted
values, so an agent can learn the CLI without parsing `--help`.

= File a task without prompts

`mt add --describe=json` lists the template's slots: each one's name, kind,
whether it is required, its default and its prompt. The agent writes the values
as YAML or JSON and passes them with `-f -` on stdin.
#link(u("/templates/"))[Templates & slots] covers the slots and the other ways
to fill them.

When stdin or stdout is not a terminal, `mt add` opens no editor. A slot left
empty fails the add, and the error names every missing slot:

#terminal("api-service-add-refused")

It exits with code
#exit-code("api-service-add-refused") and writes
nothing. With `--format json`, a successful add prints its result as an object
whose `outcome` is `wrote`, `skipped` or `suffixed`, and whose `path` is the
task file.

= Trust the exit code

`mt` exits with 0 when a command succeeds, 1 when it fails and 2 when the
command line is wrong, and it prints the reason on stderr. Every command that
writes a task checks the result first and writes nothing when the check fails,
so an agent can fix its input and run the command again.

- `mt check` exits with 1 when a task breaks a rule of severity `error`, and
  names the file and the line. Run it in CI or a pre-commit hook.
- A #link(u("/hooks/"))[flip hook] can run your tests when a task is closed,
  and refuse to close it while they fail.
- When stdout is not a terminal, `mt` prints no colour codes. `NO_COLOR` or
  `--color never` turn colour off in a terminal too.

= Work in a git worktree

Tasks are files, so each git worktree has its own copy of the backlog. An agent
in its own worktree sets its task's status there, and that change merges with
the code that does the work. There is no server or index to update.

`mt` reads only the worktree it runs in. Two agents in two worktrees do not see
each other's status changes until their branches merge, so decide which agent
takes which task before they start.

To run `mt` on a worktree from another directory, pass `-w <path>` or set
`MINDTAPE_WORKTREE`. You can also name worktrees in `mindtape/worktrees.toml`
in your #link(u("/config/"))[config directory]. A `[[worktrees]]` entry with `name` and
`path` makes a leading `@name` argument stand for `-w <path>`. An entry with
`base` makes `@<dir>` stand for `-w <base>/<dir>` when that directory holds a
MindTape project. `mt worktrees` lists the entries.

= Pin today's date

Set `MINDTAPE_NOW` to a date such as `2026-10-01`, and `mt` uses it for every
date it stamps or compares. A test or a snapshot then prints the same output on
every run. #link(u("/config/"))[Configuration] lists the formats it accepts.

= Keep your own data in a task

A tool built on MindTape stores its data in a task field named `x-<name>`.
`mt` keeps the field as written and returns it under `extras` in JSON.
#link(u("/concepts/"))[Concepts] describes these fields, and
#link(u("/self-validating-tasks/"))[Self-validating tasks] shows how to check
one.
