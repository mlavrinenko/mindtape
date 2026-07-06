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

A task in `mt ls --json`:

```json
{
  "id": "wire-oauth",
  "title": "Wire OAuth login",
  "status": { "name": "wip", "date": "2026-07-06", "note": null },
  "priority": 8.0,
  "due": "2026-07-20",
  "tags": ["backend", "auth"],
  "depends_on": [],
  "derived_tags": [],
  "extras": {},
  "file": "tasks/wire-oauth.typ"
}
```

`extras` carries the `x-<ns>` extension fields verbatim, so a tool built on
MindTape reads its own data back out without MindTape interpreting it.

= Describe before you fill

`mt add --describe=json` returns the slot schema — names, kinds, required flags,
and defaults — so an agent can construct a valid `mt add` call, or a JSON payload,
without guessing:

```sh
echo '{"title":"Wire OAuth","priority":8}' | mt add -f -
```

Fill values also come from `--set k=v` and from `MINDTAPE_ADD_<SLOT>` environment
variables, so a task can be filed with no interactive prompt.

= Honest exit codes and quiet output

`mt check` exits non-zero on any violation and names each one — drop it in CI or a
pre-commit hook. Pass `--color never` (or rely on `NO_COLOR`) for clean,
parseable output in a non-TTY:

#terminal("$ mt --color never check
design-schema: priority: out of bounds (max 9)
Error: 1 violation(s), 0 load error(s)")

= State is just files

An agent edits a task by writing the `.typ` file — there is no daemon to notify
and no index to rebuild. The next `mt` command re-evaluates the tree and sees the
change. Hand edits, scripted edits, and `mt` edits are indistinguishable.

#callout("note")[
  Building a tool on top of MindTape? Put your data under an `x-<ns>` field — see
  #link(u("/concepts/"))[the extension seam]. MindTape carries it through; your
  tool validates it.
]
