# MindTape Roadmap

## MVP (Milestone 1): Read-Only Index + CLI

Goal: evaluate Typst files, index tasks into SQLite, query via CLI.

### M1.1 — Typst Evaluation Proof of Concept [COMPLETE]

- [x] Set up Rust project with `typst` crate dependencies
- [x] Implement minimal `World` trait (file loading, minimal fonts, no packages)
- [x] Evaluate a `.typ` file and extract its content tree
- [x] Traverse content tree and extract checklist items as tasks
- [x] Extract `due` metadata from `MetadataElem` nodes
- [x] Implement CLI with `--due` filter and `-N` limit
- [x] Validate with `itest/basic.sh` — confirms task extraction, sorting, formatting

**Implementation notes**:
- `typst-kit` not needed for eval-only; `clap` replaced with manual arg parsing
  for `-N` shorthand support
- Task struct uses `typst::foundations::Datetime` directly (not NaiveDate)
- `lib/prelude.typ` uses `metadata()` for property functions (not `@mindtape` package)
- Checkboxes parsed from `plain_text()` since Typst has no native checkbox syntax
- Code refactored into lib+bin crate with 77%+ test coverage (35 tests)

### M1.2 — `@mindtape` Functions (due, tag) [COMPLETE]

- [x] Define `due()` as Typst function producing identifiable content (via `metadata()`)
- [x] Define `tag()` function
- [x] Register functions in custom World so `#import "@mindtape/mindtape:0.1.0": due, tag` resolves
- [x] Extract `due` and `tag` values from content tree after evaluation
- [x] Update examples to use `@mindtape` imports

**Exit criteria**: parse `- [ ] task #due(datetime(...)) #tag("fun")` and extract
due date + tag as structured Rust values.

**Implementation notes**:
- `lib/prelude.typ` now exports `due()`, `id()`, and `tag()` — all use `metadata()`
- `lib/typst.toml` package manifest with `entrypoint = "prelude.typ"`
- `MindTapeWorld::resolve_path()` intercepts `@mindtape` package specs and
  resolves against `{root}/lib/` directory
- Import syntax: `#import "@mindtape/mindtape:0.1.0": due, tag, id`
- `Task` struct now has `tags: Vec<String>` field
- 43 tests (26 unit + 17 integration), 80.42% coverage

### M1.3 — SQLite Store [COMPLETE]

- [x] Define domain types: `TaskFile`, `Task`, `TaskProperty`, `FileBinding`
- [x] Define `Store` trait (anti-corruption layer)
- [x] Implement SQLite backend with migrations
- [x] Index a single file: evaluate -> extract -> store
- [x] Re-index on change: detect file hash change, update accordingly

**Exit criteria**: evaluate a `.typ` file and persist its tasks/bindings to SQLite.

**Implementation notes**:
- `Store` trait with `SqliteStore` impl in `src/store.rs`
- Schema v1 with `task_files`, `tasks`, `task_properties`, `file_bindings` tables
- `index_file()` orchestrates eval -> extract -> store pipeline with hash-based skip
- `to_store_records()` converts `EvalResult` to store domain types
- `TaskFilter` supports `done`, `tag`, `due_before`, `file_path`, `limit`
- `TaskView` joins tasks with file context and properties for query results
- SHA-256 content hashing for change detection (`hash_file()`)
- `thiserror` for structured `StoreError` variants
- 97 tests (57 unit + 31 eval integration + 9 store integration), 90.93% coverage

### M1.4 — Folder Watcher [COMPLETE]

- [x] Configuration file parsing (TOML: watched folders, db path)
- [x] `.mindtapeignore` support (gitignore-style patterns)
- [x] File watcher using `notify` crate with debouncing
- [x] On file change: re-evaluate + re-index
- [x] On file delete: remove from index
- [x] Initial full scan on startup

**Exit criteria**: `mindtape watch` starts, indexes all `.typ` files in configured
folders, and keeps the index updated as files change.

**Implementation notes**:
- `src/config.rs`: TOML config loading with `serde` + `toml` crate
- `src/watcher.rs`: `Watcher` struct with `initial_scan()`, `handle_event()`, `run()`
- CLI extended with `Command` enum: `Eval` (backwards compat) + `Watch` subcommand
- `mindtape watch [path]` for quick use, `mindtape watch --config <file>` for multi-folder
- `notify-debouncer-mini` with 300ms debounce; `path.exists()` to distinguish modify vs delete
- `ignore` crate for directory walking and `.mindtapeignore` pattern matching
- Config auto-discovery: `mindtape.toml` or `./.mindtape/config.toml` in cwd, then `~/.config/mindtape/config.toml`
- Default DB path: `~/.local/share/mindtape/index.db`
- Per-file `MindTapeWorld` creation reuses existing pattern from `index_file()`
- 136 tests (90 unit + 31 eval integration + 9 store integration + 6 watcher integration), 79.48% coverage

### M1.5 — CLI Query Commands [COMPLETE]

- [x] `mindtape watch` — start the watcher daemon (done in M1.4)
- [x] `mindtape list` — list all tasks (with filters: --status, --tag, --due-before, --folder)
- [x] `mindtape list --file <path>` — list tasks from a specific file
- [x] `mindtape status` — show index stats (files indexed, total tasks, last sync)
- [x] `mindtape files` — list indexed files

**Exit criteria**: full read-only workflow works end-to-end.

**Implementation notes**:
- Three new CLI commands: `list`, `status`, `files` — all query the SQLite index
- `Store` trait extended with `list_files()` and `get_stats()` methods
- `TaskFilter` extended with `folder` field for path prefix matching
- New types: `FileView` (file + task count), `IndexStats` (aggregate counts)
- All query commands support `--db <path>` override; default DB auto-discovered
  from config or `~/.local/share/mindtape/index.db`
- `list` defaults to pending tasks only; `--status all` shows everything
- `list` output grouped by file with indented tasks
- Format: `- [x] (due DATE) title [tags]` for task views
- 177 tests (124 unit + 31 eval integration + 7 query integration + 9 store integration + 6 watcher integration)
- Integration test script (`itest/basic.sh`) expanded with 8 end-to-end query tests

---

## Milestone 2: Richer Queries + UX

### M2.1 — Search + Output Formats [COMPLETE]

- [x] Full-text search across task titles and `#let` binding values
- [x] `mindtape search "keyword"` command
- [x] Output formats: table (default), JSON, CSV via `--format` flag
- [x] `--json` kept as shorthand for `--format json`

**Implementation notes**:
- `Store::search()` method with LIKE-based matching (case-insensitive)
- `BindingView` and `SearchResults` domain types
- Schema v2 migration adds NOCASE indexes on `tasks.title` and `file_bindings.name`
- `OutputFormat` enum replaces `bool json` across all query commands
- CSV formatters for tasks, files, stats, and search results
- 204 tests across workspace

### M2.2 — Due Date Awareness + Agenda [COMPLETE]

- [x] Due date awareness: overdue tasks, upcoming tasks
- [x] `mindtape agenda` — tasks due today/this week

**Implementation notes**:
- `Store::query_agenda()` method returns `AgendaView` with overdue/today/this_week tasks
- Overdue uses `<` (strictly before today), not `<=`
- This week = today + 6 days (7 day window including today)
- CLI supports `--overdue`, `--today`, `--week` flags (default: all sections)
- All output formats supported: table, JSON, CSV
- Added 3 store tests + 4 CLI tests (153 total tests)

### M2.3 — Cross-File References

- [ ] Cross-file reference tracking (which files import which)

---

## Milestone 3: Write-Back (Mutations via API)

- [ ] `mindtape check <task-id>` — toggle task checkbox in `.typ` file
- [ ] `mindtape set <task-id> --due <date>` — update task properties
- [ ] Safe file modification: parse -> modify AST -> write back (preserve formatting)
- [ ] Conflict detection: file changed on disk since last index

---

## Milestone 4: API Server

- [ ] HTTP/REST API (axum) exposing query and mutation endpoints
- [ ] WebSocket or SSE for live updates
- [ ] API key / local auth for security
- [ ] OpenAPI spec generation
