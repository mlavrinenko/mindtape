---
name: mindtape
description: >-
  How to drive MindTape, a daemon-free file-based task tracker, via its `mt` CLI
  and one-task-per-file Typst artifacts. Use in any repo that has a
  `.mindtape/config.toml` or `.typ` task files under a tasks root, or when the
  user asks to list or find tasks, see what is ready or blocked, file or add a
  task, mark a task done / in-progress / blocked, check or validate tasks,
  retire or remove a task, or edit a task's status, priority, or links. Also
  trigger on "mindtape", "mt ls", "mt add", "mt check", the task backlog, or
  "what should I work on" in such a repo, even when the tool is not named.
---

# MindTape (`mt`)

MindTape is a file-based task tracker. Each task is one Typst file
(`#show: task.with(...)`); the filename stem is the identity. No daemon,
database, or index — every `mt` command live-evaluates the configured roots.

Core rule: drive the `mt` CLI; do not hand-edit task files to list, file, or
re-status them. `mt` stamps dates, slugs filenames, injects required fields,
resolves links, and validates against the project config — hand edits skip that
and quietly produce invalid tasks. Hand-edit a `.typ` only for what no command
covers: the body, and fields with no CLI writer (notably `links:`); then run
`mt check`.

## Is this a MindTape project?

Yes if `.mindtape/config.toml` exists at the repo root — it declares the task
roots and rules (status legend, priority scale, tags). Run `mt` from the
directory holding `.mindtape/`, or any descendant. From outside the tree:
`-w <path>`, `$MINDTAPE_WORKTREE`, or a `@name` that `mt worktrees` lists.

## The basic loop

Find a task → read it → change its status → validate.

| Goal | Command |
|------|---------|
| List active tasks | `mt ls` |
| List everything | `mt ls ''` (empty filter; there is no `--all`) |
| Filter | `mt ls '<filter>'` (see below) |
| Apply a named view | `mt ls @<view>`; `mt views` lists them |
| Dependency graph | `mt graph` (DOT; `--format mermaid`, `--no-terminal`) |
| Read one task | `mt eval <file>` (`--json`) |
| Open a task in `$EDITOR` | `mt go <id>` |
| File a new task | `mt add <title…>` (see below) |
| Change status | `mt flip <status> <task…>` or the status verb |
| Stamp `reviewed:` today | `mt review <task>` (one task) |
| Advance a recurring task | `mt recur <task>` (spawn a new file or refresh in place) |
| Validate all tasks | `mt check` |
| Retire a task | `mt rm <task>` |
| Inspect the effective config | `mt config show` (`--origin`, `--defaults`) |

`<task>` / `<id>` accepts a filename stem, a relative/absolute path, or a handle
ordinal from the last `mt ls` (the `#` column, e.g. `1b`).

Common flags are below; reach for `mt <cmd> --help` only when a command errors
or needs a flag this skill lacks. Legend, kinds, priority scale, and tags are
per-project: read `.mindtape/config.toml` (or `mt flip --list`), don't guess.

## Finding tasks — `mt ls`

Bare `mt ls` tries the project's `[[ls.rules]]` (auto-activation keyed off
branch/env/weekday/date), then falls back to the default view (`[ls]
default-view`, else `[views.default]`; usually active, startable tasks), naming
the winner on a dim `applied: ...` line. An empty filter (`mt ls ''`) drops all
of that and lists everything. There is no `--all` flag; `mt ls --all` errors
with `unexpected argument '--all'`.

Pass a filter expression as the positional argument. Fields:
`status priority difficulty kind title id due start reviewed file depth`.
`file` compares the project-root-relative path (`/`-separated on every
platform) — not the same as the `file` *column*'s working-directory-relative
display, below. `depth` is the folder nesting under a root (0 for a task in the
root itself, 1 for a sub-folder, ...), always numeric — so `mt ls 'depth == 0'`
lists the root tasks. Operators: `== != < > <= >= && || !` and parentheses.
Helpers:

- `has(field)` / `miss(field)` — field present / absent
- `has_tag("name")` — authored or folder-derived tag
- `contains(field, "substr")` / `search("query")` — substring / full-text
- `ready` / `blocked` — all direct deps terminal / not
- `terminal` — status is a configured terminal state
- `stale(N)` — untouched for more than N days; `now()` — today's ISO date
- `in_view("name")` (also `@name`) — matches a named view's filter

Named views are `[views.<name>]` bundles (filter + optional sort/columns); bare
`@name` applies all three facets, `@name` inside a bigger filter is the filter
facet only. `--columns <cols|@view>` picks columns for one run. An opt-in
`file` column (never shown unless named, e.g. `--columns '#,title,file'`)
prints each task's source path relative to the working directory.
`--hyperlinks always` (default `never`; `[ls] hyperlinks` sets it in config)
wraps both the always-visible `#` column and, when shown, `file` in an OSC 8
sequence to the same row's target, so it is clickable in a plain terminal —
`#` alone is enough, with no `file` column needed.

```sh
mt ls 'ready && priority > 5'
mt ls 'has_tag("backend") && !terminal'
mt ls 'due <= now()' --sort priority:desc
mt ls '@backend && !@blocked'        # compose named views by @name
mt ls 'stale(30)' --json             # --json / --format csv for other programs
```

## Creating a task — `mt add`

`mt add <title words>` files a new task from a template; the positional words
become the title and the filename slug (`mt add fix login bug` →
`fix-login-bug.typ`). It writes under the first configured root, injects the
status and id, and validates — never hand-write the `.typ`. Common flags:

- `-t <template>` pick a template; `--list` list templates; `--describe` show slots
- `--to <subdir>` nest under a folder of the root (drives derived tags)
- `--set key=value` fill a slot; `--print` preview instead of writing
- `-f/--values <path>` fill slots from a JSON or YAML file (repeatable, merged
  left-to-right; `-` reads stdin); `.json`/`.yaml`/`.yml` pick the parser by
  extension, stdin and any other extension sniff the content; `--set` applies
  after and wins on overlap

### `--set` keys are slot names, and unknown ones are a hard error

A `--set`/`-f` key must name a slot of the active template. An unrecognised
key fails the command: `mt add fix bug --set priority=3` errors with
`unknown --set/-f key(s): priority; known slots: title, ...` rather than
writing anything. Run `mt add --describe` (plus `-t <template>` for a
non-default one) to see the real slot names, and `--print` to check the
result before it's written.

Slot names need not match field names. A framework priority is authored as
`priority: ice(impact: slot("impact"), …)`, so the slots are its leaves —
`--set impact=8 --set confidence=0.9 --set ease=4`, never `--set priority=8`.
Under a strict framework (`framework` + `allow-manual = false` in
`[tasks.priority]`) a scalar priority is invalid by construction, so even a
template that omits priority gets the framework call spliced in and the leaves
are still the slots. Same for any field the template nests inside a call.

Tags: `+word` positionals are sugar for the `tags` slot (`mt add fix login
+backend +auth`); or repeat `--set tags[]=backend --set tags[]=auth`. A bare
`--set tags=backend,auth` is a hard error — a `list` slot needs `[]=`.

### `text` slots are escaped, not markup

Every `text`-kind slot value (`-f`/`--set`/`--edit`/positional/environment,
JSON or YAML — the source makes no difference, escaping runs after parsing) is
markup-escaped before it lands in the file: a literal `` ` `` / `*` / `_` /
`#` / `[` / `]` / `@` / `<` you type becomes `` \` ``, `\*`, etc. — safe, but
never live Typst. So don't author code spans or emphasis inside slot content
(a `summary`/`scope` fill) expecting them to render — write plain prose there.
If a task genuinely needs formatted body text (a code span, a link), fill the
slot with a placeholder and hand-edit the body afterward — sanctioned per
"Authoring or editing a task file by hand" above, since only the section
*headers* are config-enforced, not the prose under them. Heuristic: after
`mt add`, search the written task for a literal backslash-backtick pair
(`` rg -F '\`' <file> ``) — a hit is almost always markup you meant literally,
not prose you meant escaped.

## Changing status — `mt flip` and status verbs

`mt flip <status> <task…>` stamps a new status, dated today. The status must be a
legend value; every legend value is also a shortcut verb, so `mt done 1b` ≡
`mt flip done 1b`. `-m "note"` attaches a note. Use the command, not a hand edit
of the `status:` line. If the legend declares `to` edges, a disallowed move is
rejected with the valid next states listed; `--force` overrides (a same-status
re-stamp is always fine).

A project may bind actions to a flip (`[[on.flip]]`) — e.g. a gate that refuses
a close until it passes. `mt flip` is the only command that runs them; reads
never do. A refusal prints the command's output; `--force` waives it and records
the waiver in the note.

```sh
mt flip --list                 # the legend and its verbs
mt wip 1b -m "started"         # if `wip` is a legend value
mt flip done 1b 2c --force     # several at once; waive a failing gate
```

### Blocked is a dependency, not a status

`ready` / `blocked` in filters derive from `links:` — a non-terminal task with a
non-terminal `depends-on` target is blocked. Most legends carry no `blocked`
value (common: proposed/accepted/wip/done/cancelled/failed); check `mt flip
--list` before reaching for one.

Express blockage as an edge, not a status. `links:` has no CLI writer, so this
is a sanctioned hand edit — add `links: depends-on("upstream-task.typ"),`, then
`mt check`.

When the blocker is external (an upstream release, another team's decision)
there is no task to depend on. Either file a task for the external thing and
depend on it (preferred — keeps `mt ls 'blocked'` truthful and gives the blocker
somewhere to be closed), or park the task in a pre-start legend state with the
reason in the note: `mt flip proposed 1b -m "blocked on upstream X"`. Do not
leave it in `wip` as a stand-in: `wip` means someone is working it, and under
`[[tasks.limits]]` a parked task burns the WIP cap and blocks a real start.

## Validating — `mt check`

`mt check` validates every task against the project config: status/kind enums,
required fields, numeric and estimate bounds, links, cycles, tags, body headers,
`[[tasks.limits]]` caps, comment policy, and file-budget drift (warn-only).

## Authoring or editing a task file by hand

A task file is Typst. Edit the fields inside `task.with(...)`:

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Fix login bug",
  status: wip(2026, 6, 28)[started investigation],  // (name, y, m, d)[note]
  priority: 6,                          // raw int, or a call like ice(...)
  difficulty: 3,
  kind: "bug",                          // enum from config
  assignee: "ada",                      // actor working it, config-bounded
  estimate: (hours: 4),                 // opaque unit dict
  links: depends-on("other-task.typ"),  // typed edges by file stem
  tags: ("backend",),
  due: on(2026, 7, 1),                  // ISO date, or omit
)
```

Rules to respect:
- One task per file; the stem is the identity. Renaming breaks inbound links —
  prefer `mt rm` to retire, which rewrites them to tombstones. Without git,
  `[rm] strategy` decides: `reject` (default, lists the referrers), `unlink`,
  or `retire` (keeps the file, flips the status).
- Status, kind, priority scale, and tags come from `.mindtape/config.toml`.
- Custom/extension fields must be namespaced `x-<ns>` (e.g. `x-speconaut: ...`).
  Every other field name is a declared parameter of `task`, so a typo is not
  kept as an extension — it fails with ``unknown task slot `titel` ``.
- Edge helpers take plain stem strings, never `link("a.typ")` — that is content,
  which reads as the trailing note. Passing one (or a second note, a named arg,
  a non-string) is an eval error naming the helper, not a dropped edge.
- The body is Typst, so a bare `//` starts a line comment and swallows the rest
  of the line; under `[tasks.comments] policy = "forbid"` `mt check` rejects the
  file. `https://…` is safe (Typst auto-links it), but `file://`, other URL
  schemes, and any literal `//` are not — put those in backticks.
- After any hand edit, run `mt check`.

## When the tool fights you

Read the error — it usually names the offending field or file. For full flag
sets prefer `mt <cmd> --help`. An unrecognised `--set`/`-f` key is a hard error,
not silent (see the `--set` note). If this skill misled you, say so concretely
so it improves.
