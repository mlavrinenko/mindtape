---
name: mindtape
description: Drive MindTape, a daemon-free file-based task tracker, via its `mt` CLI
  and one-task-per-file Typst artifacts. Load in any repo holding `.mindtape/config.toml`
  for anything about tasks — listing, finding, filing, blocking, closing, validating,
  retiring, or "what should I work on" — even when the tool is not named. There it
  IS the tracker, not GitHub issues or ad-hoc TODOs; a repo with no tracker at all
  gets `mt init` offered before anything else.
---
<!-- Generated from docs/src/skill.typ by `just docs-md`. Do not edit; edit the source. -->

# MindTape (`mt`)

MindTape is a file-based task tracker. Each task is one Typst file (`#show: task.with(...)`); the filename stem is the identity. No daemon, database, or index — every `mt` command live-evaluates the configured roots.

Core rule: drive the `mt` CLI. A hand edit skips what `mt` stamps, slugs, injects, resolves and validates, and quietly yields an invalid task. Hand-edit a `.typ` only for what no command covers — chiefly the body — then run `mt check`.

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
| Advance a recurrence | `mt recur <task>` (new file or refresh in place) |
| Validate all tasks | `mt check` |
| Retire task(s) | `mt rm <task…>` (each a ref or a filter; union, one pass) |
| Inspect the effective config | `mt config show` (`--origin`, `--defaults`) |

`<task>` / `<id>` accepts a filename stem, a relative/absolute path, or a handle (the `#` column, e.g. `1b`). A handle is not a token from the last `mt ls`: its ordinal is the task’s position in the default view **recomputed on every command**, so any flip that re-sorts or filters renumbers it, and the trailing check letters catch most such shifts, not all. A flip reports what it moved (`handle <stem>: 1b → 3b`, or `→ none` when it left the view); for any second operation, prefer the stem, which never moves.

**Never write a handle into prose a human reads.** Use them freely for your own next command in the same turn, but a report, a commit message, a chat line and a task body all outlive the listing that produced the handle, and the reader who meets `1b` there has to re-run `mt ls` and hope. Cite the task file path instead, as a Markdown link (`[title](truth/tasks/<stem>.typ)`), so they click through instead of resolving anything. The rule is absolute: a conditional one is a judgment call you will get wrong on the run that matters.

Reach for `mt <cmd> --help` only when a command errors or needs a flag this skill lacks. Legend, kinds, priority scale, and tags are per-project: read `.mindtape/config.toml` (or `mt flip --list`), don’t guess.

## Finding tasks — `mt ls`

Bare `mt ls` tries the project’s `[[ls.rules]]` (auto-activation keyed off branch/env/weekday/date), then falls back to the default view (`[ls] default-view`, else `[views.default]`; usually active, startable tasks), naming the winner on a dim `applied: ...` line. An empty filter (`mt ls ''`) drops that filter and lists everything; there is no `--all` flag.

A filter argument replaces the filter facet only — the rule’s or view’s sort and columns still apply, and the trailer names both sources. So a listing you narrowed is not a listing you reset. For the project’s ordinary table instead of the rule’s, splice the default view back over both: `mt ls '<filter>' --columns @default --sort @default`.

`-n N` bounds a listing without a pipe. It cuts after the filter and sort, so you get the top N of the order you asked for, and a cut listing says so: a `truncated: showing 10 of 47` line under the table, the same pair as `--json`’s `truncated` envelope key. Read it before reporting a backlog as complete.

Pass a filter expression as the positional argument. Fields: `status priority difficulty kind title id due start reviewed path assignee depth topo blocks role` plus `status.name`, `status.date`, `status.note`, `status.index` — bare `status` keeps comparing the lifecycle name and sorting by legend position; `--sort` takes the same set except `role`, plus a computed `ready` rank. `role` is filter-only: it matches a role the task’s assignee holds, via the `[[actors]]` registry. `path` compares the project-root-relative path (`/`-separated on every platform); the `file` _column_ below is a working-directory-relative display, a different value under `-w`; `file` is not a filter or sort field, so naming it errors rather than listing nothing. `depth` is folder nesting under a root, always numeric, so `mt ls 'depth == 0'` lists the root tasks. `topo` and `blocks` read the `links:` graph, numeric and never absent: `topo` is dependency level (0 = nothing blocks it; ascending is a working order, ties broken by the next `--sort` spec), `blocks` is how many unfinished tasks transitively wait on this one — leverage a per-task score cannot see. A done task blocks 0; a cycle neither hangs nor panics. Operators: `== != < > <= >= && || !` and parentheses. Helpers:

- `has(field)` / `miss(field)` — field present / absent
- `has_tag("name")` — authored or folder-derived tag
- `contains("substr")` / `contains("substr", field, ...)` — case-insensitive substring, needle first; no field named searches `title`/`id`/body prose, or `[ls] contains-fields` where a project sets it
- `links("kind")` — carries a `links:` edge of that kind, built-in or custom
- `ready` / `blocked` — all direct deps terminal / not
- `terminal` — status is a configured terminal state
- `stale(N)` — untouched for more than N days; `now()` — today’s ISO date
- `in_view("name")` (also `@name`) — matches a named view’s filter
- `matches(field, "re")` / `in_dict("name", <string>)` — regex; membership in a `[dicts]` set. Their string argument may be `word(field, n)`, the n-th word counting from 1, punctuation-trimmed: `in_dict("verbs", word(title, 1))`
- `into("status")` — inside an `[[on.flip]]` hook’s `when` only; matches the status being stamped, the rest of the expression reading the pre-write task

`mt find <term…>` is pure sugar over `contains()`: `mt find login bug` == `mt ls 'contains("login") && contains("bug")'` (terms conjoin with `&&`). Every `mt ls` flag forwards; there is no `--in` — target a field with the filter directly (`mt ls 'contains("bug", title)'`).

Named views are `[views.<name>]` bundles (filter + optional sort/columns); bare `@name` applies all three facets, `@name` inside a bigger filter is the filter facet only. `--columns <cols|@view>` picks columns for one run. The `file` column prints each task’s source path relative to the working directory. It is opt-in on a terminal (`--columns '#,title,file'`) and automatic everywhere else, so your captured `mt ls` already carries the path to cite — you never need a second `--format path` run to get it. `--format csv` is unaffected and addresses tasks by `id`; its `file` cell is the root-relative path `--json` reports, not the cwd-relative display. An `assignee` column shows only when the listed rows disagree about it; naming it does not override that — `[ls.columns.assignee] when = 'true'` does. `--hyperlinks always` (default `never`, or `[ls] hyperlinks`) wraps the always-visible `#` column — and `file` when shown — in an OSC 8 link to that row’s target.

```sh
mt ls 'ready && priority > 5'
mt ls 'has_tag("backend") && !terminal'
mt ls 'due <= now()' --sort priority:desc
mt ls --sort blocks:desc --sort priority:desc  # unblocks the most, first
mt ls '@backend && !@blocked'        # compose named views by @name
mt ls 'stale(30)' --json             # --json / --format csv for other programs
mt ls ready --format path            # bare path list, one per line, no jq needed
rg TODO $(mt ls ready --format path) # compose with another program
```

## Creating a task — `mt add`

`mt add <title words>` files a new task from a template; the positional words become the title and the filename slug (`mt add fix login bug` → `fix-login-bug.typ`). It writes under the first configured root, injects the status and id, and validates _before_ writing. So does every other write command (`recur`, `flip`, `review`, `link`, `import`): a rejected write creates no file and a rejected rewrite is not applied, so fix the input and re-run rather than repairing or deleting anything. Common flags:

- `-t <template>` pick a template; `--list` list templates; `--describe` show slots
- `--to <subdir>` nest under a folder of the root (drives derived tags)
- `--set key=value` fill a slot, `--set key@path` fill it from a file; `--print` preview instead of writing
- `-f/--values <path>` fill slots from a JSON or YAML file (repeatable, merged left-to-right; `-` reads stdin); `.json`/`.yaml`/`.yml` pick the parser by extension, stdin and any other extension sniff the content; `--set` applies after and wins on overlap
- `--dedup fail|skip|add-suffix` on an existing stem: bail, exit 0 writing nothing, or mint `-2`, `-3`, …; overrides `[tasks.naming] dedup`, which defaults to `add-suffix` under a configured `stem` template, `fail` otherwise
- `--format text|path|path0|json` report the outcome instead of prose; `json` carries `{"outcome": "wrote"|"skipped"|"suffixed", "path": "..."}`, the same vocabulary as `mt ls --format`

`--describe` can list a slot no template declares: a required `assignee` (`[tasks.assignee] required`) is spliced in, backed by `$MINDTAPE_PRINCIPAL`. An unset variable is an unknown actor, not a default, so a non-interactive add fails naming the slot rather than guessing who works the task — fill it (`--set
assignee=<actor>`), with the OTHER actor’s name when filing for someone else.

Lead the title with an imperative verb — `fix login bug`, not `login bug fix`. A `[[tasks.rules]]` entry may check this (`in_dict("verbs", word(title, 1))`, see Validating below); default to it regardless.

### Slot semantics live in `references/add-slots.md`

Three things `--describe` cannot tell you, each of which silently produces a wrong task if you guess: token keys are slot NAMES (a framework priority’s slots are its leaves — `impact`/`confidence`/`ease`, never `priority`), a `list` slot needs `name+=value` and rejects a bare `name=a,b`, and `text` slots are markup-escaped while `raw` slots splice in live. Read `references/add-slots.md` before filling slots on an unfamiliar template.

Prefer `-f -` with YAML for values you generate rather than type: one structured channel, no shell quoting, and a block scalar carries a prose slot verbatim. The flag form (`--set k=v`, `--set k+=v`) and the same tokens after `--` are for argv you assemble by hand. `k@path` reads a value out of a file converting nothing — the slot’s kind still decides — for a file you already wrote, above all a `.typ` bound for a `raw` slot.

## Filing a batch — `mt import`

`mt import` reads canonical task-JSON and is the right tool for filing a whole backlog at once. Its two quiet sharp edges (the stem comes from `title`, not `id`; intra-batch `links` resolve only backwards) are why the deep `references/import.md` exists — read it before the first batch.

## Changing status — `mt flip` and status verbs

`mt flip <status> <task…>` stamps a new status, dated today. The status must be a legend value; every legend value is also a shortcut verb, so `mt done 1b` ≡ `mt flip done 1b`. `-m "note"` attaches a note. A move the legend’s `to` edges disallow is rejected with the valid next states listed; `--force` overrides (a same-status re-stamp is always fine).

A project may bind actions to a flip (`[[on.flip]]`) — e.g. a gate refusing a close until it passes. Only `mt flip` runs them; a refusal prints its output. They run as `sh -c` inheriting `mt`’s OWN environment, and `sh` re-enters no `direnv` hook of its own. So a gate binary that lives in the project’s dev shell is found exactly when `mt` itself was run with that shell loaded, and otherwise exits 127 — which prints as a FAILED gate, inviting a `--force` over a gate that never ran. Exit 127 is “binary not found”, never a refusal: fix the environment `mt` runs in and flip again. `--force` is for waiving a gate you have READ.

```sh
mt flip --list                 # the legend and its verbs
mt wip 1b -m "started"         # a legend value is its own verb
mt flip done 1b 2c --force     # several at once; waive a failing gate
```

### Blocked is a dependency, not a status

`ready` / `blocked` in filters derive from `links:` — a non-terminal task with a non-terminal `depends-on` target is blocked. Most legends carry no `blocked` value; check `mt flip --list` before reaching for one.

Express blockage with `mt link` (below), never a hand edit.

An external blocker (an upstream release, another team’s decision) has no task to depend on. Either file a task for it and depend on that (preferred — keeps `mt ls 'blocked'` truthful), or park the task in a pre-start legend state with the reason in the note: `mt flip proposed 1b -m "blocked on upstream X"`. Not `wip` as a stand-in — under `[[tasks.limits]]` a parked task burns the WIP cap and blocks a real start.

## Cross-linking — `mt link`

`mt link <task> <kind> <target…>` upserts one edge per target into `links:`. `mt link --apply <file>` (`-` for stdin) reads an adjacency map instead — YAML or JSON, stem to kind to targets, a target being a stem or a `{target, note}` mapping. It is the tool for wiring a batch you just filed:

```yaml
wire-oauth:
  depends-on: [pick-a-provider, read-the-rfc]
  related:
    - target: write-docs
      note: same release
```

Both forms merge and dedupe on (kind, target), so re-running writes nothing. `--replace` (`--apply` only) makes the map the complete edge set per stem; `--dry-run` prints the plan. Kind, target and note rule are checked BEFORE the write, so a refusal leaves nothing to repair — a kind whose note is `required` (`related`, out of the box) needs `-m "why"`, `depends-on`’s is optional. A per-entry failure is reported and the rest still land.

## Validating — `mt check`

`mt check` validates every task against the project config: status/kind enums, required fields, numeric and estimate bounds, links, cycles, tags, body headers, section content (`[tasks.body.content]`), `[[tasks.limits]]` caps, `[[tasks.rules]]` assertions, comment policy, and file-budget drift (warn-only). A `[[tasks.rules]]` entry at `severity = "error"` also refuses the write commands above; at `warn` it prints and lets them through.

## Authoring or editing a task file by hand

The full worked example and the rules (stem identity, `x-<ns>` namespacing, edge helpers, body pitfalls) live in `references/hand-editing.md`. After any hand edit, run `mt check`.

When a task proposes a shape — a config key, a function signature, a CLI invocation, a file layout — show it as a raw block; let prose carry only what the block cannot say.

## When the tool fights you

Read the error — it usually names the offending field or file. An unrecognised `--set`/`-f` key is a hard error, not silent (see `references/add-slots.md`). If this skill misled you, say so concretely so it improves.
