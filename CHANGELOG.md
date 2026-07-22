# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Work-in-progress limits (`[[tasks.limits]]`)** — cap how many tasks may sit
  in a matched set at once and have `mt check` report a breach with the cap, the
  count and every task in the set. A limit is a `filter` plus a `max`, so no
  status name is hardcoded: the same primitive caps a status, a named view
  (`@stuck`), a priority band or a `kind`. Optional `message` replaces the
  default advice; `severity` is `"warn"` (default — a breach is a state of the
  backlog, cleared by finishing work, not by editing a file) or `"error"`, which
  fails the command for projects wanting a hard stop. Rests on the filter
  language's missing-field rule (an absent field is unequal to every literal), so
  a `kind == "x"` / `kind != "x"` pair partitions the whole backlog.
- **typstyle-clean writes (`[format]`)** — `mt` now runs typstyle over the Typst
  source it writes (`add`, `flip`, `review`, `recur`, `import`), so surgical
  edits and slot fills land in one canonical shape instead of leaving drift like
  stray body indentation. Uses `typstyle-core` as a library — in-process,
  deterministic, idempotent (a clean file is unchanged beyond the edited span),
  and error-tolerant (malformed source passes through) — pinning the
  `typst-syntax` version already resolved via `typst-harvest`, so no second
  `typst` enters the tree. `[format] enabled = false` opts out, keeping writes as
  raw byte splices for minimal diffs.
- **JSON Schema for `.mindtape/config.toml`** — generated from the config
  structs (`mindtape::config_schema_json`, draft 2020-12), so it never drifts
  from the accepted shape and its field docs come from the type doc comments.
  `mt install` now also writes it beside the app config
  (`~/.config/mindtape/config.schema.json`; skip with `--no-schema`), and
  `mt init` scaffolds a config whose first line is a `#:schema` comment
  directive pointing at it, so a schema-aware TOML editor (tombi, Taplo) assists
  filling values. `mt init --schema URL|PATH` retargets the directive (e.g. the
  published URL for a portable, checked-in config); `--no-schema` opts out.
  `mt install-lib` is now equivalent to `mt install --no-defaults --no-schema`.
- **`mt import [FILE|-]`** — import canonical task-JSON (a file arg or stdin
  `-`; the bare array `mt ls --json` emits by default, or its `{"tasks": [...]}`
  envelope) as native `.typ` tasks. Maps each record onto the project's config
  policy (`mindtape::map_canonical`: status legend, priority/difficulty scale,
  `kind` enum, tag allow-list, the `x-<ns>` extras discipline), mints a
  destination stem from the title (`mindtape::stem_for_title`, disambiguated
  `-2`/`-3`/... on collision), emits it (`mindtape::emit_task`), then writes
  the whole batch before reloading and validating the destination as one set —
  so records that link to each other resolve. A record that fails validation
  is reported and removed, never left half-written; `--dry-run` stages into a
  temp mirror of the project instead of touching the real destination.
  `--into`/`--to` mirror `mt add`'s destination resolution; `--status` defaults
  a missing status, `--map OLD=NEW` renames a foreign status before legend
  validation, `--drop-unmapped` drops (instead of keeping and reporting) a tag
  outside the allow-list. No `--from`/`--via` source selector — a foreign
  tracker's export becomes canonical JSON via a `jq` adapter, upstream of the
  pipe. "`mt add` without a template, from a full record."
- **`contrib/jq/{taskwarrior,github}.jq`** — source adapters mapping
  `task export`'s JSON and `gh issue list --json
  number,title,body,state,labels,assignees,createdAt,url`'s JSON onto the
  canonical task-JSON schema `mt import` consumes. Pure structural jq, no
  Rust and no `config.toml` read (`mt import` owns the per-project policy
  map): status maps onto neutral names shared by both sources (`pending`/
  `open` -> `todo`, `completed`/`closed` -> `done`, `deleted` -> `cancelled`),
  Taskwarrior's H/M/L priority maps onto MindTape's default 0..9 scale
  (8/5/2), tags come from `project`+`tags` or issue `labels`, and
  uuid/urgency/annotations/issue-number/url/assignees carry through
  losslessly as `x-tw`/`x-gh` extras. Both escape a literal `#` (and `\`)
  before splicing source text into the raw-Typst `body` field, since an
  unescaped `#` would open Typst code mode and fail to compile. `jq` is now
  a flake devShell dependency; fixtures and goldens live under
  `contrib/jq/fixtures/`.
- **`mt graph [filter]`** — emit the task dependency graph as Graphviz DOT
  (default) or Mermaid (`--format mermaid`). Nodes are tasks labelled by title
  and status and filled by readiness (ready/blocked/terminal); edges are the
  typed `links:` relations, styled per kind. An optional filter scopes the
  induced subgraph (edges to out-of-scope tasks drop) and `--no-terminal` prunes
  closed tasks. A read-only projection reusing the same graph model that
  computes ready/blocked (`mindtape::edges`) — no new store, a diffable view of
  the same source.
- **Recurring tasks (`recur`, `mt recur`)** — a task's `recur: every(weeks: 1,
  on: (mon, wed, fri), anchor: "due", until: on(2027, 1, 1), count: 10,
  materialize: "spawn")` shapes a `(rule, anchor, until, count, materialize)`
  dict; no date math in the prelude — `[tasks.recur]` validates it and
  `mindtape::recur_dates` computes the next occurrence (chrono period
  arithmetic; month/year-end clamped). `rrule("FREQ=...")` shares the same
  shape as the escape hatch, gated behind `[tasks.recur] rrule` (parsed;
  expansion is a separate `rrule` cargo feature — see below). `mt recur X` is
  the only writer, never a side effect of a read (`mt ls`/`mt check` are
  unaffected, enforced by test): the default `spawn` materialize writes the
  next instance as a new file (stem minted by `[tasks.naming]`, or the
  built-in `{slug}-{date}` fallback) with
  a `recurred-from` edge back to its predecessor, refusing a second spawn
  unless `--force`; `mutate` refreshes the same file in place. `anchor:
  "completion"` (default) always advances from today; `anchor: "due"` catches
  up from the stale due date until strictly after today. `--dry-run` previews
  without writing; `--advance N` pre-materializes N future spawns bounded by
  `[tasks.recur] max-advance`; `--skip` advances the anchor/`count` without
  materializing anything. `mt done` stays a pure close on a recurring task —
  it never strips `recur` — and prints a notice pointing at `mt recur`.

- **`rrule(...)` expansion engine** — the `rrule("FREQ=MONTHLY;BYDAY=1MO")`
  escape hatch (see above) now expands, for the calendar-complex cadences
  `every()` cannot express: first Monday of the month (`BYDAY=1MO`), last day
  of the month (`BYMONTHDAY=-1`), every weekday (`BYDAY=MO,TU,WE,TH,FR`), and
  the rest of RFC 5545. Wrapped by a new `rrule` cargo feature (off by
  default) over the `rrule` crate; the common `every()` path stays free of
  its `chrono-tz` + `regex` dependencies unless a project actually authors an
  `rrule(...)`. `rrule(...)` is date-anchored only — it implies `anchor:
  "due"` and rejects `anchor: "completion"` (RFC 5545 has no
  completion-relative primitive); DTSTART is seeded from the task's `due` and
  the next occurrence is the rule's first instance strictly after it,
  bounded by the crate's own iteration cap so a pathological rule errors
  instead of hanging. `mt check` grammar-checks the raw rule string at parse
  time and quotes the offending rule in a malformed one's error.

- **Pinnable "today" (`$MINDTAPE_NOW`)** — every date `mt` stamps or reads —
  status/`reviewed` dates, `now()`/`stale()` in filters, the overdue marker,
  `mt recur` advances — resolves "today" once per process from `$MINDTAPE_NOW`
  (a `YYYY-MM-DD` date or an RFC 3339 timestamp; the date part is used), else
  the system clock (UTC). Pin it for deterministic output — docs generation,
  golden-file tests, a CI snapshot. A malformed value is a hard error naming the
  variable, never a silent fall-through. `$SOURCE_DATE_EPOCH` is deliberately
  not honoured: `mt`'s "today" is an operational query concept, not a
  build-artifact timestamp (Nix dev/CI shells set it to 1980-01-01).

- **Worktree override** — a global `-w/--worktree <path>` flag (and
  `$MINDTAPE_WORKTREE` fallback; the flag wins) points `mt` at a chosen project
  instead of discovering one by walking up from the cwd. The path must hold a
  `.mindtape/`; relative paths resolve against the cwd. Every command routes
  through the single discovery entry point, so `ls`, `add`, `check`, `flip`,
  `go`, `rm`, and `config` all honour it.

- **Worktree aliases** — name worktrees in an app-scoped
  `~/.config/mindtape/worktrees.toml` and select one with a leading `@name`:
  `mt @sn ls` expands to `mt -w <path> ls`. Entries are tried in order — an alias
  (`name` + `path`) matches by name; a `base` directory matches `@<token>` when
  `<base>/<token>` is a project, so one `base` covers a whole tree of worktrees.
  `~` expands to home; `mt worktrees` lists the configured entries. Only the
  leading argument is sugar, so the `@view` filter sigil is unaffected.

- **Named views** — declare reusable query bundles as `[views.<name>]` tables
  (a `filter` plus optional `sort`, `columns`, `description`). Reference one with
  an `@name` sigil: a bare `mt ls @broken` applies the view's filter, sort, and
  columns; `@name` inside a filter (`mt ls '@summer && !@broken'`) is the filter
  facet only, equal to the new `in_view("name")` predicate. New `mt ls --columns`
  picks columns for a run (`@view` for a view's set) and `--sort @view` splices a
  view's sort chain. New `mt views` lists and inspects views (`--json`). Views
  are validated at config load: filters parse, columns are legal, references
  resolve, and the reference graph is acyclic.
- **Conditional `ls` rules (`[[ls.rules]]`)** — bare `mt ls` adapts to ambient
  state: the first entry whose `condition` holds wins, composing `import-views`
  (last-wins) plus a manual override, else `default-view`. New env-only
  condition language: `on_branch(glob)`, `env(name)`, `weekday()`, `now()`.
- **Persistent content-hash task cache** — warm `mt ls`/`mt check` over a
  mostly-static backlog now read each unchanged task's harvested marker from a
  git-ignored `.mindtape/cache/` instead of re-evaluating it through Typst; only
  edited or new files re-eval (stacking with the parallel batch harvester on
  that cold path). Entries are keyed by the file's content hash under a build
  key (cache format + `mt` version + config + prelude/`lib` contents), so any of
  those changing busts the whole cache; a stale, corrupt, or wrong-format entry
  falls back to a live eval, never a wrong task. The cache is scratch — reads
  populate it without violating the read/write (CQRS) invariant, which now
  guards the authored tree only. Set `MINDTAPE_NO_CACHE=1` to bypass it.

### Fixed

- **`mt add` escapes text slot fills into Typst markup** — a text slot carrying
  ordinary prose used to land in markup verbatim, so a value containing `*`, `_`,
  `$`, `#`, `[`, `]`, `<` or `@` produced a task file that failed to load the
  instant it was written (an asterisk opened strong emphasis and the task died
  with `unclosed delimiter`). Every fill source now routes through the same
  escaper status notes already used — `--set`, `-f`, the positional title, the
  `MINDTAPE_ADD_*` env vars and the `--edit` YAML buffer — while numeric, choice
  and date slots stay unescaped, being literals rather than markup. A write that
  still fails verification now says the file was kept on disk, instead of
  reporting `wrote <path>` and leaving a broken artifact behind unmentioned.

### Removed

- **`mt ls --all`** — pass an empty filter (`mt ls ''`) to drop the default and
  list everything. Omitting the filter applies the default view.
- **`[ls] default-filter`** — replaced by `default-view` (see Changed).

### Changed

- **`required` is a filter expression (breaking)** — `[tasks.priority] required`
  (and its `difficulty` / `estimate` / `[tasks.fields.<name>]` siblings) still
  takes `true` and `false`, but the third form is now any filter expression
  instead of the one-off `"non-terminal"` enum value: the field is required for
  exactly the tasks it matches (`"kind == \"bug\""`, `"@active"`). Rename
  `"non-terminal"` to `"!terminal"` — identical meaning, and no alias is kept, so
  the old spelling fails at config load with the replacement named. A malformed
  expression is one config error at load, not one violation per task.
- **`mt ls --json` envelope (breaking)** — now `{"tasks": [...], "applied":
  {"rule": ..., "facets": {...}}}`, not a bare array; update a `jq '.[...]'`
  consumer to `jq '.tasks[...]'`.
- **`[ls] default-filter` → `default-view`** — the default `mt ls` listing is now
  a view, not a bare filter, so it carries sort and columns too, not only a
  filter. Omitting the filter resolves `[ls] default-view` (a view name), else the
  implicit `[views.default]` view, else match-all. The embedded floor ships
  `[views.default]` with the previous default filter, so bare `mt ls` is unchanged
  out of the box; redeclare `[views.default]` (or set `default-view`) to add a
  sort/columns bundle. `[ls] default-filter` is removed.
- **Repointed onto published crates.io substrate** — the seven extracted
  crates (`typst-world`, `typst-harvest`, `typst-cst`, `typst-edit`,
  `typst-cleanup`, `swhid-mint`, `artifact-gate`) are now `crates.io`
  deps (`0.1.0`), not in-tree path crates; workspace is down to `mindtape`
  + `mindtape-cli`. No CLI-visible change. Version reset to `0.2.0`.
- **`typst-harvest` split over a new `typst-world` crate** — the eval-only
  `typst::World` (file-provider seam, `@local` package resolution, dependency
  tracking) and the Typst-free `HVal` value tree now live in `typst-world`;
  `typst-harvest` is a thin layer that adds only `metadata()` marker harvesting
  on top. Internal reshape: no CLI-visible or behavior change. Render support in
  `typst-world` sits behind an off-by-default `render` feature, so eval-only
  consumers pull no layout/render/font dependencies.
- **Binary renamed `mindtape` → `mt`** — the installed executable is now `mt`;
  invocations become `mt ls`, `mt check`, `mt add`, etc. Crate names
  (`mindtape`, `mindtape-cli`), the `@local/mindtape` Typst package, the
  `.mindtape/` config directory, and the `MINDTAPE_ADD_*` env prefix are
  unchanged.
- **`mindtape::naming::disambiguate` returns a `Result` (breaking)** — stem
  collision disambiguation (used by `mt add`, `mt import`, `mt recur` spawn)
  now returns `Result<String, NamingError>` instead of looping forever
  against an `exists` predicate that never reports a free stem. Past
  `MAX_DISAMBIGUATE_ATTEMPTS` (10,000) numeric suffixes it returns the new
  `NamingError::Exhausted(stem, ceiling)` instead of hanging; callers surface
  it as an ordinary command error. Breaking change to a public library API —
  the crate is still on its 0.2 pre-release line.
- **`mt recur`'s due-anchor catch-up no longer hangs** —
  `mindtape::recur_dates::next_date`'s catch-up loop (advancing a stale `due`
  forward by the rule until strictly after today) is now bounded by
  `MAX_CATCH_UP_STEPS` (10,000 steps) and returns the new
  `RecurError::CatchUpExhausted(due)` instead of spinning forever when the
  rule never advances (e.g. `every(days: 0)`) or an expansion engine keeps
  returning its own reference date. Previously `mt recur` would hang in that
  case.
- **Recur/date internal cleanup** — the `rrule` cargo feature's off-state now
  lives in one stub module (`recur_rrule_off.rs`, selected by `#[path]`)
  rather than `#[cfg]` twin functions inside `recur.rs`/`recur_dates.rs`, so
  every function in those two files compiles under any feature set;
  `epoch_days_to_ymd` also dropped an unreachable negative-`shifted` branch of
  Hinnant's civil-from-days algorithm (the input is `u64`, so it is always
  `>= 719_468`). Both are dead-code/coverage hygiene, no behaviour change.

### Added

- **Status transitions** — a status entry may declare `to = [...]`, the allowed
  next statuses, turning the flat legend into a state machine. Opt-in: with no
  `to` on any entry the legend stays any-to-any as before. Once declared,
  `mt flip` rejects a move that is not a listed edge (printing the allowed next
  states; `--force` overrides), and `mt check` flags a task whose status is
  unreachable from the initial one under the graph. A same-status re-stamp and
  the first stamp on a status-less task are always exempt.
- **`mt ls --interactive` (`-i`)** — an interactive picker over the `ls`
  list: type a task's `#`-column ordinal to highlight it (reverse-video, prefix
  match), `Enter` opens the first highlighted task in the editor (`go`),
  `Ctrl+Enter` opens every listed task, `Space` re-evaluates the project, and
  `q` / `Ctrl+C` leaves. Table-only and TTY-only. Backed by `crossterm`, which
  negotiates the terminal keyboard-enhancement protocol so `Ctrl+Enter` /
  `Ctrl+Backspace` carry their modifier where supported (`Del` is the always-
  reliable clear).
- **`typst-cst` crate** — package-agnostic parse-plane structure extraction:
  the heading tree (depth, text, source span) parsed via `typst-syntax`, with no
  eval, World, or IO. A consumer-blind substrate for gating document structure.
- **`artifact-gate` crate** — plane-agnostic structural rule engine: match a
  generic node tree against three rule shapes — path cardinality
  (`require`/`forbid`/`unique`, exact or regex segments, direct-child or
  recursive), document ordering (one path's matches must precede another's), and
  closed-world allowlists (only named nodes permitted under a scope) — and emit
  diagnostics. Consumer-blind — no Typst, task model, or domain concepts.
  `mt check`'s body-header gating rides it over the `typst-cst` heading
  tree; speconaut can reuse it for its own artifact structure rules.
- **`typst-edit` crate** — consumer-blind in-place Typst source editing:
  parse-plane span queries plus an atomic, non-overlapping changeset (`apply`)
  over `typst-syntax`, no eval/World/IO. `find_calls(source, name)` locates any
  function call with its positional/named argument value spans and trailing
  `[…]` content block (`find_link_targets` is the thin link-only case); `line_of`
  maps an offset to a 1-based line. Foundation for `mt rm` link
  rewriting and the speconaut scanner collapse (see
  `docs/decisions/003-swhid-cleanup-tombstones-and-shared-edit-substrate.md`).
- **`swhid-mint` crate** — mint and resolve Software Heritage identifiers
  (SWHID, ISO/IEC 18670:2025). VCS-agnostic core (`render`/`parse`/`is_swhid_url`)
  plus a `git` backend feature (via the `gix` crate, no subprocess, no libgit2)
  that mints a revision-plus-path id for a tracked file and resolves one back. The
  tombstone substrate for `mt rm`.
- **`typst-cleanup` crate** — consumer-blind orchestration for retiring Typst
  artifacts: dirty-tree guard, per-file SWHID tombstone minting, and atomic
  inbound-`link()` rewriting, planned in memory before any write. The consumer
  supplies eligibility, link identity, and tombstone policy via a `Policy` trait.
  `mt rm` rides it; speconaut adopts it under a separate task. See
  `docs/decisions/004-gix-git-substrate.md` for the git substrate.
- **`mt rm`** — retire terminal-status tasks: deletes each terminal
  task file and rewrites every surviving `link("<stem>.typ")` that pointed at it
  into an SWHID tombstone `link("swh:1:rev:<commit>;path=<path>")`. Does not
  commit — the user reviews the diff and commits. Refuses a dirty working tree on
  a real run; `--dry-run` previews the plan without writing. Atomic — every mint
  and rewrite is planned in memory before a byte is written, so a wrong-shaped
  file or an unrecoverable commit aborts with no changes. `swh:` link targets are
  treated as history pointers by the loader/validator (not task graph edges).
- **`mt flip`** — set a task's status without opening the file:
  `flip <status> <task>...` stamps `<status>` dated today onto each target (the
  same shortcut/verbose rendering `add` uses, via `mindtape::status_value`).
  Every configured legend value also works as a top-level shortcut verb —
  `mt done 1b 2c` == `mt flip done 1b 2c` — generated from the legend
  (built-in subcommands win on a clash; a shadowing value is warned about);
  `mt flip --list` prints them. `-m/--note` sets the status note. Targets
  are resolved before any write (a status change reorders the default view), then
  valid edits apply and any resolution or post-edit validation problem is
  reported without rolling back the rest.
- **`mt go`** — open a task in the configured editor by handle, slug, or
  path (`[go] editor` → `$VISUAL` → `$EDITOR`; a `{task}` token expands to the
  resolved path, else the path is appended).
- **`mt review`** — stamp `reviewed:` with today's date on a non-terminal
  task; idempotent within the same day.

## [0.4.0] - 2026-06-14

Per-file rewrite. See `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.

### Added

- **Per-file task model** — one task per file via `#show: task.with(...)`, with
  a `(name, date, note)` status, configurable enum/number core fields, task
  links, tags, and an `x-<ns>` extension seam for downstream tools
- **`typst-harvest` crate** — a package-agnostic, eval-only Typst World plus
  labelled-metadata-marker harvesting; a reusable substrate for file-based tools
- **`.mindtape/config.toml`** — per-project status lifecycle, enum/number field
  constraints (required-unless-terminal, bounds), tag allow-list, default filter
- **`mindtape check`** — config-driven validation incl. link resolution
- **`mindtape install-lib`** — installs the Typst library (once per machine)
- **`mindtape init`** — scaffolds project config (once per project)
- Dev tooling: parallel `just check`, sccache, cargo-machete, linecop, outdatty

### Changed

- Split into a three-crate workspace: `typst-harvest`, `mindtape` (library),
  `mindtape-cli` (binary). The filter language now runs over live-evaluated
  tasks instead of SQL
- Typst library bumped to 0.2.0 (new task DSL; breaking)

### Removed

- The `watch` daemon, the SQLite index, the `mindtape-store` crate,
  eval-hash skipping, the global cross-project agenda, inline checkbox markers
  and `#id()` identity, and the NixOS service module

## [0.3.0] - 2026-03-23

### Added

- **Global agenda filter** — `[agenda]` now supports a top-level `filter` field
  that is AND-ed into every task section, e.g. hide tasks with a future
  `#start()` date across all sections
- **Default `#start()` filter** in the built-in agenda: tasks whose start date
  hasn't arrived are automatically hidden

### Changed

- **Agenda config restructured** — `[[agenda]]` sections are now nested under
  `[agenda]` as `[[agenda.sections]]`, enabling agenda-level settings alongside
  per-section configuration (breaking change to config format)

### Fixed

- Resolve workspace version for naersk Nix build

## [0.2.0] - 2026-03-16

### Added

- **`mindtape agenda`** subcommand — configurable Typst-rendered agenda with
  pluggable sections (task list, file index)
- **File index section** for the agenda, showing indexed files and their status
- **`mindtape inspect`** improvements — comfy-table rendering, `--format typst`
  output, `~`-shortened paths, and file indexing error tracking

### Fixed

- Exclude imported duplicate tasks from query results
- Exclude nested sub-items from parent task title during evaluation
- Clear stale file errors on unchanged content hash and prune deleted files on scan
- Remove incorrect header from `mindtape list` output, adjust header levels

### Changed

- CI: use stable Rust toolchain, guard unnecessary workflow runs, allow manual
  workflow dispatch

## [0.1.0] - 2026-03-07

Initial release.

### Added

- **Typst-based task format** with `#due()`, `#start()`, `#id()`, `#tag()`,
  `#rank()` metadata functions and `#high`, `#medium`, `#low` aliases
- **Typst evaluation engine** (`mindtape-eval`) — extracts structured task data
  from `.typ` files using `typst-eval`, with AST-based write-back support
- **SQLite store** (`mindtape-store`) — indexes tasks into a queryable database
  with FTS5 full-text search, schema migrations, and a `Store` trait abstraction
- **File watcher** with `.gitignore`-aware filtering, hot-reload config, and
  cross-file dependency tracking
- **CLI commands**:
  - `mindtape <file>` — evaluate a single Typst file
  - `mindtape watch` — watch folders and build the index
  - `mindtape list` — query tasks with `--filter` expressions, `--sort`, and
    `--format` (table/json/csv/typst) output
  - `mindtape check <id>` — toggle task checkboxes (writes back to `.typ` files)
  - `mindtape set <id>` — update task properties (due, start, rank, tags)
  - `mindtape inspect` — show index stats, files, and dependency graphs
  - `mindtape id` — generate and convert base62-encoded UUIDv7 task IDs
  - `mindtape init` — install the Typst prelude library
  - `mindtape eval` — evaluate filter expressions
  - `mindtape deps` — show file dependencies
- **Filter expressions**: `has(due)`, `miss(tag)`, `has_tag("backend")`,
  `search("auth")`, comparisons on `due`, `start`, `rank`, and boolean combinators
- **NixOS module** for declarative service configuration
- **Cross-platform release binaries** (Linux x86_64/aarch64, macOS x86_64/aarch64)

[0.3.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.3.0
[0.2.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.2.0
[0.1.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.1.0
