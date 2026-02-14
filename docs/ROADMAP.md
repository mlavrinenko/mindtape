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
- `lib/prelude.typ` uses `metadata()` for property functions (not `@mind-tape` package)
- Checkboxes parsed from `plain_text()` since Typst has no native checkbox syntax
- Code refactored into lib+bin crate with 77%+ test coverage (35 tests)

### M1.2 — `@mind-tape` Functions (due, tag) [COMPLETE]

- [x] Define `due()` as Typst function producing identifiable content (via `metadata()`)
- [x] Define `tag()` function
- [x] Register functions in custom World so `#import "@mind-tape/mind-tape:0.1.0": due, tag` resolves
- [x] Extract `due` and `tag` values from content tree after evaluation
- [x] Update examples to use `@mind-tape` imports

**Exit criteria**: parse `- [ ] task #due(datetime(...)) #tag("fun")` and extract
due date + tag as structured Rust values.

**Implementation notes**:
- `lib/prelude.typ` now exports `due()`, `id()`, and `tag()` — all use `metadata()`
- `lib/typst.toml` package manifest with `entrypoint = "prelude.typ"`
- `MindTapeWorld::resolve_path()` intercepts `@mind-tape` package specs and
  resolves against `{root}/lib/` directory
- Import syntax: `#import "@mind-tape/mind-tape:0.1.0": due, tag, id`
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

### M1.4 — Folder Watcher

- [ ] Configuration file parsing (TOML: watched folders, db path)
- [ ] `.mindtapeignore` support (gitignore-style patterns)
- [ ] File watcher using `notify` crate with debouncing
- [ ] On file change: re-evaluate + re-index
- [ ] On file delete: remove from index
- [ ] Initial full scan on startup

**Exit criteria**: `mind-tape watch` starts, indexes all `.typ` files in configured
folders, and keeps the index updated as files change.

### M1.5 — CLI Query Commands

- [ ] `mind-tape watch` — start the watcher daemon
- [ ] `mind-tape list` — list all tasks (with filters: --status, --tag, --due-before, --folder)
- [ ] `mind-tape list --file <path>` — list tasks from a specific file
- [ ] `mind-tape status` — show index stats (files watched, total tasks, last sync)
- [ ] `mind-tape files` — list indexed files

**Exit criteria**: full read-only workflow works end-to-end.

---

## Milestone 2: Richer Queries + UX

- [ ] Full-text search across task titles and `#let` binding values
- [ ] `mind-tape search "keyword"` command
- [ ] Output formats: table (default), JSON, CSV
- [ ] Due date awareness: overdue tasks, upcoming tasks
- [ ] `mind-tape agenda` — tasks due today/this week
- [ ] Cross-file reference tracking (which files import which)

---

## Milestone 3: Write-Back (Mutations via API)

- [ ] `mind-tape check <task-id>` — toggle task checkbox in `.typ` file
- [ ] `mind-tape set <task-id> --due <date>` — update task properties
- [ ] Safe file modification: parse -> modify AST -> write back (preserve formatting)
- [ ] Conflict detection: file changed on disk since last index

---

## Milestone 4: API Server

- [ ] HTTP/REST API (axum) exposing query and mutation endpoints
- [ ] WebSocket or SSE for live updates
- [ ] API key / local auth for security
- [ ] OpenAPI spec generation

---

## Milestone 5: Frontends

- [ ] TUI (ratatui)
- [ ] Web UI
- [ ] Editor integrations (VS Code, Neovim)

---

## Future / Ideas

- DuckDB as alternative store backend
- Typst package published to `@preview` for `due`, `tag`, etc.
- Custom user-defined task properties
- Recurring tasks
- Task dependencies / blocking relationships
- Notifications (desktop, email)
- Sync across machines (CRDTs, git-based)
- `mind-tape init` scaffolding for new projects

---

## Non-Goals (Explicit)

- MindTape is NOT a Typst renderer — we never produce PDFs or visual output
- MindTape is NOT a general Typst IDE — use tinymist for that
- MindTape does NOT replace Typst files — they are always the source of truth
- MindTape does NOT require internet access for core functionality
