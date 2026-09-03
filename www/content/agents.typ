#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/agents/",
  label: "For AI agents",
  title: "For AI agents",
  summary: "What makes MindTape legible to a coding agent: structured JSON, describable schemas, honest exit codes, and file-based state.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

None of this is an agent mode. It is the ordinary CLI surface — a shell script
and an agent drive the same commands, for the same reasons.

= Structured output everywhere

Every read command speaks JSON:

```sh
mt ls --json          # tasks as JSON (--format csv for CSV)
mt eval <file> --json # one task file, evaluated
mt config show --json # the effective config
mt add --describe=json # the slot schema for the active template
mt _cli --json        # the command tree — names, flags, defaults, enums
```

`mt _cli --json` reflects `mt`'s own clap definition into a stable JSON tree
(this site's #link(u("/cli/"))[CLI reference] renders from it), so a tool can
discover every command, flag, default, and enum value without scraping `--help`.

A task in `mt ls --json` (the hero's Für Elise task, captured verbatim from the
`learn-piano` example):

#raw(read("../generated/ls-task.json"), lang: "json", block: true)

`extras` carries the `x-<ns>` extension fields verbatim, so a tool built on
MindTape reads its own data back out without MindTape interpreting it.

`mt ls --json`'s top-level shape is `{"schema_version": ..., "tasks": [...],
"applied": {...}, "truncated": ...}`, not a bare array — `applied` names which
#link(u("/cli/"))[`[[ls.rules]]`] entry (if any) drove the listing and which
source won each facet, so an agent can tell a conditional listing from a plain
one without re-deriving it. `schema_version` (see #link(u("/schema/"))[the
  canonical schema]) names the envelope shape it is about to parse: a version it
does not recognize is a signal to bail out, not to mis-parse silently.

`truncated` closes the bounded-listing question. `mt ls ready -n 10 --json`
keeps a reply small, and `truncated` says whether small was the whole answer:
`null` when nothing was cut, else `{"shown": 10, "total": 47}` — ask again with
a bigger `-n` rather than reporting a backlog that has 37 more rows in it.

#terminal(read("../generated/ls-json.txt"))

= Describe before you fill

`mt add --describe=json` returns the slot schema — names, kinds, required flags,
and defaults — so an agent can construct a valid `mt add` call, or a JSON payload,
without guessing:

```sh
echo '{"title":"Practice arpeggios","priority":8}' | mt add -f -
```

Fill values also come from `--set k=v` and from environment variables — a
template's `slot(…, env: "VAR")` names an exact variable, otherwise every slot
reads `MINDTAPE_ADD_<SLOT>` — so a task can be filed with no interactive prompt.

= Honest exit codes and quiet output

`mt check` exits non-zero on any violation and names each one — drop it in CI or
a pre-commit hook. Pass `--color never` (or rely on `NO_COLOR`) for parseable
output in a non-TTY:

#terminal(read("../generated/check.txt"))

= Run from anywhere

An agent driving `mt` from outside the project tree points it at a worktree with
`-w/--worktree <path>` (or `$MINDTAPE_WORKTREE`; the flag wins). The path must
hold a `.mindtape/`; relative paths resolve against the cwd. It is a global
flag, so every command honours it and no `cd` into the checkout is needed.

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

// The full rule — which dates resolve from it, the accepted formats, the
// local (never UTC) fallback and the hard error on a malformed value — is
// /config/'s "Environment variables" section. Stating it twice is how the two
// copies drift; this page owns only the reason an agent reaches for it.
Pin `$MINDTAPE_NOW` and every date `mt` stamps or reads comes from it instead of
the clock — the one knob that makes a run reproducible, for docs generation,
golden-file tests, or a CI snapshot. #link(u("/config/"))[Environment variables]
has the rule.

```sh
MINDTAPE_NOW=2026-07-11 mt ls --json
```

= State is just files

An agent edits a task by writing the `.typ` file — there is no daemon to notify
and no index to rebuild. The next `mt` command re-evaluates the tree and sees the
change. Hand edits, scripted edits, and `mt` edits are indistinguishable.

#callout("note")[
  Building a tool on top of MindTape? Put your data under an `x-<ns>` field — see
  #link(u("/concepts/"))[the extension seam]. MindTape carries it through; your
  tool validates it.
]
