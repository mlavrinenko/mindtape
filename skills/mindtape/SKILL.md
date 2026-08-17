---
name: mindtape
description: How to drive MindTape, a daemon-free file-based task tracker, via its
  `mt` CLI and one-task-per-file Typst artifacts. Use in any repo that has a `.mindtape/config.toml`
  or `.typ` task files under a tasks root, or when the user asks to list or find tasks,
  see what is ready or blocked, file or add a task, mark a task done / in-progress
  / blocked, check or validate tasks, retire or remove a task, or edit a task's status,
  priority, or links. Also trigger on "mindtape", "mt ls", "mt add", "mt check", the
  task backlog, or "what should I work on" in such a repo, even when the tool is not
  named.
---
<!-- Generated from docs/src/skill.typ by `just docs-md`. Do not edit; edit the source. -->

# MindTape (`mt`)

MindTape is a file-based task tracker. Each task is one Typst file (`#show: task.with(...)`); the filename stem is the identity. No daemon, database, or index — every `mt` command live-evaluates the configured roots.

Core rule: drive the `mt` CLI; do not hand-edit task files to list, file, or re-status them — a hand edit skips what `mt` stamps, slugs, injects, resolves and validates, and quietly yields an invalid task. Hand-edit a `.typ` only for what no command covers: the body, and fields with no CLI writer (notably `links:`); then run `mt check`.

## Is this a MindTape project?

Yes if `.mindtape/config.toml` exists at the repo root — it declares the task roots and rules (status legend, priority scale, tags). Run `mt` from the directory holding `.mindtape/`, or any descendant. From outside the tree: `-w <path>`, `$MINDTAPE_WORKTREE`, or a `@name` that `mt worktrees` lists.

## The basic loop

Find a task → read it → change its status → validate.

| Goal | Command |
| --- | --- |
| List active tasks | `mt ls` |
| List everything | `mt ls ''` (the empty filter) |
| Filter | `mt ls '<filter>'` (see below) |
| Search by text | `mt find <term…>` (see below) |
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

`<task>` / `<id>` accepts a filename stem, a relative/absolute path, or a handle ordinal from the last `mt ls` (the `#` column, e.g. `1b`).

Reach for `mt <cmd> --help` only when a command errors or needs a flag this skill lacks. Legend, kinds, priority scale, and tags are per-project: read `.mindtape/config.toml` (or `mt flip --list`), don’t guess.

## Finding tasks — `mt ls`

Bare `mt ls` tries the project’s `[[ls.rules]]` (auto-activation keyed off branch/env/weekday/date), then falls back to the default view (`[ls] default-view`, else `[views.default]`; usually active, startable tasks), naming the winner on a dim `applied: ...` line. An empty filter (`mt ls ''`) drops all of that and lists everything; there is no `--all` flag.

Pass a filter expression as the positional argument. Fields: `status priority difficulty kind title id due start reviewed file assignee depth role` — `--sort` takes the same set except `role`, plus a computed `ready` rank. `role` is filter-only and resolves through the `[[actors]]` registry: it matches a role the task’s assignee holds. `file` compares the project-root-relative path (`/`-separated on every platform) — not the same as the `file` _column_’s working-directory-relative display, below. `depth` is the folder nesting under a root (0 for a task in the root itself, 1 for a sub-folder, …), always numeric — so `mt ls 'depth == 0'` lists the root tasks. Operators: `== != < > <= >= && || !` and parentheses. Helpers:

- `has(field)` / `miss(field)` — field present / absent
- `has_tag("name")` — authored or folder-derived tag
- `contains("substr")` / `contains("substr", field, ...)` — case-insensitive substring, needle first; no field named searches `title`/`id`/body prose, or `[ls] contains-fields` where a project sets it
- `links("kind")` — carries a `links:` edge of that kind, built-in or custom
- `ready` / `blocked` — all direct deps terminal / not
- `terminal` — status is a configured terminal state
- `stale(N)` — untouched for more than N days; `now()` — today’s ISO date
- `in_view("name")` (also `@name`) — matches a named view’s filter
- `into("status")` — only inside an `[[on.flip]]` hook’s `when`, where it matches the status being stamped while the rest of the expression reads the task as it stands before the write

`mt find <term…>` is pure sugar over `contains()`: `mt find login bug` == `mt ls 'contains("login") && contains("bug")'` (terms conjoin with `&&`). Every `mt ls` flag forwards; there is no `--in` — target a field with the filter directly (`mt ls 'contains("bug", title)'`).

Named views are `[views.<name>]` bundles (filter + optional sort/columns); bare `@name` applies all three facets, `@name` inside a bigger filter is the filter facet only. `--columns <cols|@view>` picks columns for one run. An opt-in `file` column (never shown unless named, e.g. `--columns '#,title,file'`) prints each task’s source path relative to the working directory. `--hyperlinks always` (default `never`, or set `[ls] hyperlinks`) wraps the always-visible `#` column — and `file` when shown — in an OSC 8 link to that row’s target.

```sh
mt ls 'ready && priority > 5'
mt ls 'has_tag("backend") && !terminal'
mt ls 'due <= now()' --sort priority:desc
mt ls '@backend && !@blocked'        # compose named views by @name
mt ls 'stale(30)' --json             # --json / --format csv for other programs
mt ls ready --format path            # bare path list, one per line, no jq needed
rg TODO $(mt ls ready --format path) # compose with another program
```

## Creating a task — `mt add`

`mt add <title words>` files a new task from a template; the positional words become the title and the filename slug (`mt add fix login bug` → `fix-login-bug.typ`). It writes under the first configured root, injects the status and id, and validates. Common flags:

- `-t <template>` pick a template; `--list` list templates; `--describe` show slots
- `--to <subdir>` nest under a folder of the root (drives derived tags)
- `--set key=value` fill a slot; `--print` preview instead of writing
- `-f/--values <path>` fill slots from a JSON or YAML file (repeatable, merged left-to-right; `-` reads stdin); `.json`/`.yaml`/`.yml` pick the parser by extension, stdin and any other extension sniff the content; `--set` applies after and wins on overlap
- `--dedup fail|skip|add-suffix` collision policy on an existing stem: `fail` bails, `skip` exits 0 writing nothing, `add-suffix` mints `-2`, `-3`, …; overrides `[tasks.naming] dedup`, which defaults to `add-suffix` under a configured `stem` template and `fail` otherwise
- `--format text|path|path0|json` report the outcome instead of prose; `json` carries `{"outcome": "wrote"|"skipped"|"suffixed", "path": "..."}`, the same vocabulary as `mt ls --format`

`--describe` can list a slot no template declares: a required `assignee` (`[tasks.assignee] required`) is spliced in, backed by `$MINDTAPE_PRINCIPAL`. An unset variable is an unknown actor and not a default, so a non-interactive add fails naming the slot rather than guessing who works the task — fill it (`--set assignee=<actor>`), and pass the OTHER actor’s name when you are filing work for someone else.

### Slot semantics live in `references/add-slots.md`

Three things about slots you cannot infer from `--describe`, each of which silently produces a wrong task if you guess: token keys are slot NAMES (a framework priority’s slots are its leaves — `impact`/`confidence`/`ease`, never `priority`), a `list` slot needs `name+=value` and rejects a bare `name=a,b`, and `text` slots are markup-escaped while `raw` slots splice in live. Read `references/add-slots.md` before filling slots on an unfamiliar template.

Prefer `-f -` with YAML when you are generating values rather than typing them: one structured channel, no shell quoting, and a block scalar carries a prose slot verbatim. The flag form (`--set k=v`, `--set k+=v`) and the same tokens after `--` are there for argv you assemble by hand.

## Changing status — `mt flip` and status verbs

`mt flip <status> <task…>` stamps a new status, dated today. The status must be a legend value; every legend value is also a shortcut verb, so `mt done 1b` ≡ `mt flip done 1b`. `-m "note"` attaches a note. If the legend declares `to` edges, a disallowed move is rejected with the valid next states listed; `--force` overrides (a same-status re-stamp is always fine).

A project may bind actions to a flip (`[[on.flip]]`) — e.g. a gate that refuses a close until it passes. `mt flip` is the only command that runs them; reads never do. A refusal prints the command’s output; `--force` waives it and records the waiver in the note.

```sh
mt flip --list                 # the legend and its verbs
mt wip 1b -m "started"         # if `wip` is a legend value
mt flip done 1b 2c --force     # several at once; waive a failing gate
```

### Blocked is a dependency, not a status

`ready` / `blocked` in filters derive from `links:` — a non-terminal task with a non-terminal `depends-on` target is blocked. Most legends carry no `blocked` value; check `mt flip --list` before reaching for one.

`links:` has no CLI writer, so expressing blockage is a sanctioned hand edit — add `links: depends-on("upstream-task.typ"),`, then `mt check`.

When the blocker is external (an upstream release, another team’s decision) there is no task to depend on. Either file a task for the external thing and depend on it (preferred — keeps `mt ls 'blocked'` truthful), or park the task in a pre-start legend state with the reason in the note: `mt flip proposed 1b -m "blocked on upstream X"`. Do not leave it in `wip` as a stand-in — under `[[tasks.limits]]` a parked task burns the WIP cap and blocks a real start.

## Validating — `mt check`

`mt check` validates every task against the project config: status/kind enums, required fields, numeric and estimate bounds, links, cycles, tags, body headers, `[[tasks.limits]]` caps, comment policy, and file-budget drift (warn-only).

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

- One task per file; the stem is the identity. Renaming breaks inbound links — prefer `mt rm` to retire, which rewrites them to tombstones. Without git, `[rm] strategy` decides: `reject` (default, lists the referrers), `unlink`, or `retire` (keeps the file, flips the status).
- Status, kind, priority scale, and tags come from `.mindtape/config.toml`.
- Custom/extension fields must be namespaced `x-<ns>` (e.g. `x-speconaut: ...`). Every other field name is a declared parameter of `task`, so a typo is not kept as an extension — it fails with an `unknown task slot` error naming the field, e.g. `titel`.
- Edge helpers take plain stem strings, never `link("a.typ")` — that is content, which reads as the trailing note. Passing one (or a second note, a named arg, a non-string) is an eval error naming the helper, not a dropped edge.
- The body is Typst, so a bare `//` starts a line comment and swallows the rest of the line; under `[tasks.comments] policy = "forbid"` `mt check` rejects the file. `https://…` is safe (Typst auto-links it), but `file://`, other URL schemes, and any literal `//` are not — put those in backticks.
- After any hand edit, run `mt check`.

## When the tool fights you

Read the error — it usually names the offending field or file. An unrecognised `--set`/`-f` key is a hard error, not silent (see `references/add-slots.md`). If this skill misled you, say so concretely so it improves.
