#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, callout
#import "../utils/site.typ": u

#show: page.with(
  title: "For AI agents",
  active: "/agents/",
  summary: "MindTape is legible to coding agents: structured JSON, describable schemas, honest exit codes, and file-based state.",
)

MindTape is a good fit for AI coding agents for the same reasons it suits scripts:
state is plain files, output is structured, and every command is deterministic.

= Structured output everywhere

Every read command speaks JSON:

```sh
mt ls --json          # tasks as JSON (--format csv for CSV)
mt eval <file> --json # one task file, evaluated
mt config show --json # the effective config
mt add --describe=json # the slot schema for the active template
```

A task in `mt ls --json` (the hero's Für Elise task, captured verbatim from the
`learn-piano` example):

#raw(read("../generated/ls-task.json"), lang: "json", block: true)

`extras` carries the `x-<ns>` extension fields verbatim, so a tool built on
MindTape reads its own data back out without MindTape interpreting it.

`mt ls --json`'s top-level shape is `{"tasks": [...], "applied": {...}}`, not a
bare array — `applied` names which #link(u("/cli/"))[`[[ls.rules]]`] entry (if
any) drove the listing and which source won each facet, so an agent can tell a
conditional listing from a plain one without re-deriving it:

#terminal(read("../generated/ls-json.txt"))

= Describe before you fill

`mt add --describe=json` returns the slot schema — names, kinds, required flags,
and defaults — so an agent can construct a valid `mt add` call, or a JSON payload,
without guessing:

```sh
echo '{"title":"Practice arpeggios","priority":8}' | mt add -f -
```

Fill values also come from `--set k=v` and from `MINDTAPE_ADD_<SLOT>` environment
variables, so a task can be filed with no interactive prompt.

= Honest exit codes and quiet output

`mt check` exits non-zero on any violation and names each one — drop it in CI or a
pre-commit hook. On a clean project it exits 0. Pass `--color never` (or rely on
`NO_COLOR`) for clean, parseable output in a non-TTY:

#terminal(read("../generated/check.txt"))

= Run from anywhere

An agent driving `mt` from outside the project tree points it at a worktree with
`-w/--worktree <path>` (or `$MINDTAPE_WORKTREE`; the flag wins). The path must
hold a `.mindtape/`; relative paths resolve against the cwd. Every command —
`ls`, `add`, `check`, `flip`, `go`, `rm`, `config` — honours it uniformly, so no
`cd` into the checkout is needed.

```sh
mt -w ~/code/app check
MINDTAPE_WORKTREE=~/code/app mt ls
```

To skip the full path, name worktrees once in the app-scoped
`~/.config/mindtape/worktrees.toml` and select one with a leading `@name`, which
expands to `-w <path>` before dispatch. An alias binds a name to a path; a `base`
maps `@<name>` to `<base>/<name>` when that is a project, so one `base` covers a
whole tree of checkouts. Entries are tried top-to-bottom, first match wins; `~`
expands to home. `mt worktrees` lists them. Only the leading argument is sugar,
so the `@view` filter sigil (`mt ls @broken`) is unaffected.

```toml
# ~/.config/mindtape/worktrees.toml
[[worktrees]]
name = "sn"                       # mt @sn ls → ~/projects/home/speconaut
path = "~/projects/home/speconaut"

[[worktrees]]
base = "~/projects/home"          # mt @app ls → ~/projects/home/app (if a project)
```

```sh
mt @sn check
mt @app ls
```

= Reproducible "today"

Every date `mt` stamps or reads — `status`/`reviewed` dates, `now()` and
`stale()` in filters, the overdue marker, `mt recur` advances — resolves
"today" once per process from `$MINDTAPE_NOW` (`YYYY-MM-DD` or an RFC 3339
timestamp; the date part is used), else the system clock (UTC). Pin it for
deterministic output — docs generation, golden-file tests, a CI snapshot:

```sh
MINDTAPE_NOW=2026-07-11 mt ls --json
```

A malformed `$MINDTAPE_NOW` is a hard error naming the variable, never a silent
fall-through to the system clock.

= State is just files

An agent edits a task by writing the `.typ` file — there is no daemon to notify
and no index to rebuild. The next `mt` command re-evaluates the tree and sees the
change. Hand edits, scripted edits, and `mt` edits are indistinguishable.

#callout("note")[
  Building a tool on top of MindTape? Put your data under an `x-<ns>` field — see
  #link(u("/concepts/"))[the extension seam]. MindTape carries it through; your
  tool validates it.
]
