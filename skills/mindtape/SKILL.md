---
name: mindtape
description: >-
  How to drive MindTape, a daemon-free file-based task tracker, via its `mt` CLI
  and one-task-per-file Typst artifacts. Use this whenever you are in a repo that
  has a `.mindtape/config.toml` or `.typ` task files under a tasks root, or when
  the user asks to list or find tasks, see what is ready or blocked, file or add
  a task, mark a task done / in-progress / blocked, check or validate tasks,
  retire or remove a task, or edit a task's status, priority, or links. Also
  trigger on mentions of "mindtape", "mt ls", "mt add", "mt check", the task
  backlog, or "what should I work on" in such a repo, even when the tool is not
  named.
---

# MindTape (`mt`)

MindTape is a file-based task tracker. Each task is one Typst file
(`#show: task.with(...)`); the filename stem is the task's identity. There is no
daemon, database, or index — every `mt` command live-evaluates the configured
roots. You drive it through the `mt` binary.

Core rule: drive the `mt` CLI; do not hand-edit task files to list, file, or
re-status them. `mt` stamps dates, slugs filenames, injects required fields,
resolves and rewrites links, and validates against the project config — hand
edits skip all of that and quietly produce invalid or inconsistent tasks. Edit a
`.typ` by hand only for what no command covers: rewording the body, fixing a typo,
adjusting a field the CLI does not manage.

## Is this a MindTape project?

It is if `.mindtape/config.toml` exists at the repo root. That file declares the
task roots and the rules (status legend, priority scale, tags). When unsure where
tasks live, read it. Run `mt` from the directory holding `.mindtape/` (or any
descendant); it walks up to find the config.

## The basic loop

Most work is: find a task → read it → change its status → validate.

| Goal | Command |
|------|---------|
| List active tasks | `mt ls` |
| List everything | `mt ls ''` (empty filter; there is no `--all`) |
| Filter | `mt ls '<filter>'` (see below) |
| Apply a named view | `mt ls @<view>`; `mt views` lists them |
| Machine-readable list | `mt ls --json` |
| Read one task | `mt eval <file>` or `mt eval <file> --json` |
| Open a task in `$EDITOR` | `mt go <id>` |
| File a new task | `mt add <title…>` (see below) |
| Change status | `mt flip <status> <task…>` or the status verb |
| Validate all tasks | `mt check` |
| Retire a task | `mt rm <task>` |
| Inspect the effective config | `mt config show` (`--origin`, `--defaults`) |

`<task>` / `<id>` accepts a filename stem, a relative/absolute path, or a handle
ordinal from the last `mt ls` (the `#` column, e.g. `1b`).

The common flags are listed below, so reach for `mt <cmd> --help` only when a
command errors or you need a flag this skill does not cover — not as a warm-up.
The legend, kinds, priority scale, and tags are per-project; when you must pick
among project-defined values, read `.mindtape/config.toml` (or `mt flip --list`
for statuses) instead of guessing — but skip that lookup when the value is
already given to you.

## Finding tasks — `mt ls`

Bare `mt ls` applies the project's default filter (usually: active, startable
tasks) and prints a table with handle ordinals, status, priority, due date, and
title. Pass an empty filter (`mt ls ''`) to drop the default and list everything;
there is no `--all`.

Pass a filter expression as the positional argument. Fields:
`status priority difficulty kind title id due start reviewed file`.
Operators: `== != < > <= >= && || !` and parentheses. Helpers:

- `has(field)` / `miss(field)` — field present / absent
- `has_tag("name")` — authored or folder-derived tag
- `contains(field, "substr")` — substring match
- `search("query")` — full-text over title + body
- `ready` / `blocked` — all direct deps terminal / not
- `terminal` — status is a configured terminal state
- `stale(N)` — untouched for more than N days
- `now()` — today's ISO date, e.g. `due < now()`
- `in_view("name")` (also `@name`) — matches a named view's filter

Named views are `[views.<name>]` bundles (a filter plus optional sort/columns).
A bare `@name` argument applies all three facets; `@name` inside a bigger filter
is the filter facet only. `--columns <cols|@view>` picks columns for one run.

Examples:

```sh
mt ls 'status == "wip"'
mt ls 'ready && priority > 5'
mt ls 'has_tag("backend") && !terminal'
mt ls 'due <= now()' --sort priority:desc
mt ls @broken                        # a named view (filter + sort + columns)
mt ls '@backend && !@blocked'        # compose views by @name
```

For another program to consume, use `mt ls --json` (or `--format csv`).

## Creating a task — `mt add`

`mt add <title words>` files a new task from a template. The positional words
become the title and the filename slug (`mt add fix login bug` →
`fix-login-bug.typ`). It writes under the first configured root and validates the
result. For a plain task that single command is enough — no discovery step, and
never hand-write the `.typ`: `mt add` injects the status and id, slugs the
filename, and validates for you. Common flags:

- `-t <template>` — pick a template (default from config)
- `--to <subdir>` — nest under a folder of the root (drives derived tags)
- `--set key=value` — fill a template slot
- `--print` — preview the filled task instead of writing
- `--describe` — show the template's fillable slots
- `--list` — list available templates

Inspect slots first only when filing into an unfamiliar custom template:
`mt add --describe -t <template>` (or `mt add --list` to see what exists). For the
built-in default template, go straight to `mt add <title>`.

Multiple tags: `+word` positional args are sugar for the `tags` slot, e.g.
`mt add fix login +backend +auth`; or repeat `--set tags[]=backend --set
tags[]=auth`. A bare `--set tags=backend,auth` is one literal tag, not two — a
`list` slot always needs `[]=` (or `-f values.json` with a JSON array).

## Changing status — `mt flip` and status verbs

`mt flip <status> <task…>` stamps a new status, dated today. The status must be a
value from the project's legend. Every legend value is also a shortcut verb, so
`mt done 1b` ≡ `mt flip done 1b`. Add `-m "note"` to attach a note.

The legend is project-defined; if unsure a status exists, check `mt flip --list`,
but when the target is obvious (user says "mark it done" and `done` is in the
legend) just run it. Use the command, not a hand edit of the `status:` line — it
stamps today's date and validates the transition. If the legend declares `to`
transition edges, `mt flip` rejects a disallowed move and lists the valid next
states; pass `--force` to override (a same-status re-stamp is always allowed).

```sh
mt flip --list                 # show the legend and its verbs
mt wip 1b -m "started"         # if `wip` is a legend value
mt flip blocked 2c 3a          # set several at once
mt flip done 1b --force        # bypass the transition graph
```

## Validating — `mt check`

`mt check` evaluates every task against the project config: status/kind enums,
required fields, numeric bounds on priority/difficulty, link resolution and
dependency cycles, tag allow-list, and body-header rules. Run it after editing
task files by hand. A clean run means the backlog is valid.

## Authoring or editing a task file by hand

A task file is Typst. Edit the fields inside `task.with(...)`:

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Fix login bug",
  status: wip(2026, 6, 28)[started investigation],  // (name, y, m, d)[note]
  priority: 6,                  // raw int, or a framework call like ice(...)
  difficulty: 3,
  kind: "bug",                  // enum from config
  estimate: (hours: 4),         // opaque unit dict
  depends-on: (link("other-task.typ"),),  // links by file stem
  tags: ("backend",),
  due: on(2026, 7, 1),          // ISO date, or omit
)

= Notes

Body is normal Typst markup, free for context, checklists, etc.
```

Rules to respect:
- One task per file; the filename stem is the identity. Renaming a file breaks
  inbound `link()`s — prefer `mt rm` to retire, or update links and `mt check`.
- Status values, kinds, priority scale, and allowed tags are all defined in
  `.mindtape/config.toml`. Read it before inventing values.
- Custom/extension fields must be namespaced `x-<ns>` (e.g. `x-speconaut: ...`).
- After any hand edit, run `mt check`.

## When the tool fights you

If a command errors, read the message — it usually names the offending field or
file. `mt check` after edits surfaces most problems. For full flag sets, prefer
`mt <cmd> --help` over guessing.

## Improving this skill (internal note)

This skill is intentionally minimal and grown step by step. If you hit something
this skill does not cover, or guidance here led you astray, surface a concrete
suggestion to improve this file (what was missing, what command/flag you needed,
the exact error) so the next run goes better. (Internal-only note for now; remove
on publish.)
