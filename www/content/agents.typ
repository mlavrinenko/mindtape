#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": REPO, SITE_URL, u

#let meta = (
  route: "/agents/",
  label: "For AI agents",
  title: "For AI agents",
  summary: "How a coding agent reads tasks as JSON, files tasks without prompts and works in its own git worktree.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The `owner/repo` both skill installers take, cut from the repository URL.
#let repo-slug = REPO.split("github.com/").last()

Want a coding agent to work through your backlog? It uses the same `mt`
commands you do. Tasks are files in your repo, so an agent reads and edits
them with the tools it already has, and `mt check` tells it when an edit
breaks a rule. The examples below run against `examples/api-service/`, a
sample backend project.

= Give your agent the skill

The MindTape repository ships an agent skill in
#link(REPO + "/tree/main/skills/mindtape")[`skills/mindtape/`]. Its `SKILL.md`
tells an agent which `mt` command does what and when to use it, and the agent
reads the files in `references/` when a task needs them. The skill uses the
#link("https://agentskills.io")[Agent Skills] format, so any agent that
supports the format can load it.

With GitHub CLI 2.90 or later, install it with `gh skill`, a public preview.
It takes the skill from the latest release:

// docs:hand-typed the build does not install from GitHub
#raw(
  "gh skill install "
    + repo-slug
    + " mindtape --agent claude-code --scope user",
  lang: "sh",
  block: true,
)

Replace `claude-code` with your agent. `gh skill install --help` lists the
agents it knows, and
#link("https://cli.github.com/manual/gh_skill_install")[its manual] covers the
other flags. `--scope user` loads the skill in every project.

Or use the #link("https://skills.sh")[skills] CLI, which needs Node.js and
installs from the default branch:

// docs:hand-typed the build does not install from GitHub
#raw(
  "npx skills add "
    + repo-slug
    + " --skill mindtape --agent claude-code --global",
  lang: "sh",
  block: true,
)

To install it by hand, copy `skills/mindtape/` with its `references/` into
the skills directory your agent reads.
#link("https://code.claude.com/docs/en/skills")[Claude Code],
#link("https://developers.openai.com/codex/build-skills")[Codex] and
#link(
  "https://docs.github.com/en/copilot/concepts/agents/about-agent-skills",
)[GitHub Copilot]
each document theirs.

= Read these docs as Markdown

Every page of this site has a Markdown copy at `index.md` under its URL, such
as #link(SITE_URL + "/concepts/index.md") for Concepts, and each page's header
links to its copy. #link(SITE_URL + "/llms.txt") lists every copy with its
summary, so an agent can find the page it needs from one file.

= Read tasks as JSON

Every read command prints JSON with `--json`: `mt ls`, `mt eval <task>`,
`mt config show`, `mt worktrees` and `mt add --describe=json`. Here's
`mt ls ready -n 1 --json`:

#raw(read("../generated/api-service-ls-ready.json"), lang: "json", block: true)

`applied.facets` says where each #link(u("/cli/ls/"))[facet] of the listing,
its filter, sort and columns, came from. `-n` keeps the reply short.
`truncated` says how many tasks matched and where the next page starts, so
`mt ls ready -n 1 --skip 1 --json` returns the next task. `truncated` is
`null` when the list is complete. An agent that sees a `schema_version` it
doesn't know should stop, not guess. #link(u("/schema/"))[The JSON schema]
describes every field.

`mt _cli --json` prints every command with its flags, defaults and accepted
values, so an agent can learn the CLI without parsing `--help`.

= File a task without prompts

`mt add --describe=json` lists the template's slots: each one's name, kind,
whether it's required, its default and its prompt. The agent writes the values
as YAML or JSON and passes them with `-f -` on stdin.
#link(u("/templates/"))[Templates & slots] covers the slots and the other ways
to fill them.

When stdin or stdout isn't a terminal, `mt add` opens no editor. A slot left
empty fails the add, and the error names every missing slot:

#terminal("api-service-add-refused")

It exits with code #exit-code("api-service-add-refused") and writes nothing.
With `--format json`, a successful add prints an object whose `outcome` is
`wrote`, `skipped` or `suffixed`, and whose `path` is the task file.

= Trust the exit code

`mt` exits with 0 when a command succeeds, 1 when it fails and 2 when the
command line is wrong, and it prints the reason on stderr. Every command that
writes a task checks the result first and writes nothing if the check fails,
so an agent can fix its input and run the command again.

- `mt check` exits with 1 when a task breaks a rule of severity `error`, and
  names the file and the line. Run it in CI or a pre-commit hook.
- A #link(u("/hooks/"))[flip hook] can run your tests when a task is closed,
  and refuse to close it while they fail.
- When stdout isn't a terminal, `mt` prints no colour codes. `NO_COLOR` or
  `--color never` turn colour off in a terminal too.

= Work in a git worktree

Tasks are files, so each git worktree has its own copy of the backlog. An
agent in its own worktree sets its task's status there, and that change merges
with the code that does the work. There's no server or index to update.

`mt` reads only the worktree it runs in. Two agents in two worktrees don't see
each other's status changes until their branches merge, so decide which agent
takes which task before they start.

To run `mt` on a worktree from another directory, pass `-w <path>` or set
`MINDTAPE_WORKTREE`. You can also name worktrees in `mindtape/worktrees.toml`
in your #link(u("/config/"))[config directory]. A `[[worktrees]]` entry with
`name` and `path` makes a leading `@name` argument stand for `-w <path>`. An
entry with `base` makes `@<dir>` stand for `-w <base>/<dir>` when that
directory holds a MindTape project. `mt worktrees` lists the entries.

= Pin today's date

Set `MINDTAPE_NOW` to a date such as `2026-10-01`, and `mt` uses it for every
date it stamps or compares. Then a test or a snapshot prints the same output
on every run. #link(u("/config/"))[Configuration] lists the formats it
accepts.

= Keep your own data in a task

If you build a tool on MindTape, store its data in a task field named
`x-<name>`. `mt` keeps the field as written and returns it under `extras` in
JSON. #link(u("/concepts/"))[Concepts] describes these fields, and
#link(u("/self-validating-tasks/"))[Self-validating tasks] shows how to check
one.
