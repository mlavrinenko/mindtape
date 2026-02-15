# MindTape — Project Context

## What is this?

A file-based task tracker that uses Typst (`.typ`) files as the source of truth.
Watches folders, evaluates Typst files via the compiler crates, indexes tasks and
metadata into SQLite, and exposes a CLI for querying.

## Key Docs

- `docs/DESIGN.md` — architecture, data model, design rationale
- `docs/ROADMAP.md` — milestone index with links to details
- `docs/TESTING.md` — testing guidelines
- `crates/mindtape-eval/CLAUDE.md` — eval crate context
- `crates/mindtape-store/CLAUDE.md` — store crate context

## Tech Stack

- **Language**: Rust (workspace with 3 crates)
- **Build**: Nix flake with `naersk`, `just` for task runner
- **Typst**: `typst`, `typst-eval`, `typst-library`, `typst-syntax` (0.14), `comemo` (0.5)
- **Database**: SQLite via `rusqlite` (behind a `Store` trait)
- **File watching**: `notify` 7.0 + `notify-debouncer-mini` 0.5
- **CLI**: `clap` 4 (derive) with `-N` shorthand via arg preprocessor
- **Serialization**: `serde` + `serde_json` for `--json` output
- **Config**: `toml` + `serde` for TOML config files
- **Logging**: `log` 0.4 + `env_logger` 0.11 — `-v`/`-vv`/`-vvv` or `RUST_LOG`

## Agent Rules

- Use `just` recipes instead of raw cargo commands (see `Justfile`)
- Use `-q` for cargo commands — only show errors/warnings, not compilation progress
- After any code changes, run `just check` (clippy + tests + file size) and fix all warnings
- If clippy suggests `--fix`, use `cargo clippy --fix --workspace --all-targets` to auto-apply mechanical fixes
- Always improve the `Justfile` when you notice missing or useful recipes
- Avoid dumping large tool outputs into context; summarize or truncate
- When working on a single crate, read that crate's `CLAUDE.md` for focused context
- Save research findings to `archive/research/` as markdown files
- Keep files small: Rust ≤800 lines, Markdown ≤200 lines (enforced by `just check-file-size`)
- **After completing a task with code/config changes**:
  1. Update `docs/ROADMAP.md` if milestones changed
  2. Update `docs/DESIGN.md` if architecture changed
  3. If implementing a REVIEW.md, update it with completion status and summary
  4. Suggest a conventional commit message (`feat:`, `fix:`, `refactor:`, `chore:`, `docs:`)

## Architecture Principles

- Typst files are always the source of truth — the database is a derived index
- Evaluation-only: we use `typst_eval::eval()`, never layout or render
- Anti-corruption layer: `Store` trait abstracts the database
- Workspace with focused crates; root crate is the CLI binary
- All pure logic is testable; main.rs is a thin CLI wrapper

## Workspace Layout

```
Cargo.toml              -- workspace root + CLI binary
Justfile                -- dev recipes (check, test, clippy, build, cover, itest, all, fmt, count-tests)

crates/
  mindtape-eval/        -- Typst evaluation + task extraction
    src/eval.rs          -- EvalError, Task, EvalResult, eval_file(), content traversal
    src/world.rs         -- MindTapeWorld (World trait impl), project root detection
    src/write.rs         -- AST-based write-back (toggle checkboxes)

  mindtape-store/       -- Store trait + SQLite backend
    src/id.rs            -- UUIDv7 generation, base62 encoding/decoding
    src/store/mod.rs     -- domain types, Store trait, StoreError
    src/store/sqlite.rs  -- SqliteStore impl, schema v3, migrations
    src/store/indexer.rs -- hash_file(), to_store_records(), index_file()

src/                    -- root crate: CLI binary
  cli/
    mod.rs              -- Cli (clap), Command enum, OutputFormat, QueryOpts, preprocess_args
    format.rs           -- shared formatters: format_task_view, csv_escape, etc.
    commands/
      mod.rs            -- re-exports all command modules
      eval.rs           -- filter_and_sort, format_task, due_sort_key
      watch.rs          -- WatchArgs
      list.rs           -- ListArgs, StatusFilter
      search.rs         -- SearchArgs, format_search_results/csv
      agenda.rs         -- AgendaArgs, format_agenda/csv
      deps.rs           -- DepsArgs, format_deps/csv
      check.rs          -- CheckArgs
      id.rs             -- IdArgs (generate/validate task IDs)
  config.rs             -- TOML config loading, WatchEntry, tilde expansion
  watcher.rs            -- Watcher struct, initial_scan, handle_event, run

tests/                  -- integration tests
  common.rs             -- shared setup (setup_typst_project, ymd)
  eval_integration.rs   -- Typst eval end-to-end tests
  store_integration.rs  -- eval -> store pipeline tests
  query_integration.rs  -- query command tests
  watcher_integration.rs -- watcher scan + event handling tests

lib/prelude.typ         -- due(), id(), tag() functions using metadata()
itest/basic.sh          -- shell integration tests
```

Dependency graph: `mindtape-eval` <- `mindtape-store` <- `mindtape` (root)

## Task Format (Typst Convention)

```typ
#import "@mindtape/mindtape:0.1.0": due, id, tag

= Milestone Title

- [ ] task text #due(datetime(...)) #id("uuid") #tag("category")
- [x] completed task

#let some_binding = "searchable value"
```

- Headings = milestones / groups
- Checklist items = tasks (parsed from `plain_text()`, not native Typst)
- `#due()`, `#id()`, `#tag()` = task properties (produce `MetadataElem` via `metadata()`)
- `#let` bindings = file-level metadata, indexed for search

## Testing

Run `just check` (clippy + tests) or `just all` (+ integration tests).
Use `just count-tests` for current test count. See `docs/TESTING.md` for guidelines.

## CLI Commands

- `mindtape <file.typ> [--due] [-N]` — eval a single file
- `mindtape watch [<path>] [--config <file>]` — watch and index folders
- `mindtape list [--status done|pending|all] [--tag TAG] [--due-before DATE] [--file PATH] [--folder PREFIX] [-N] [--db PATH] [--format table|json|csv]`
- `mindtape search <keyword> [-N] [--db PATH] [--format table|json|csv]`
- `mindtape agenda [--overdue] [--today] [--week] [-N] [--db PATH] [--format table|json|csv]`
- `mindtape status [--db PATH] [--format table|json|csv]`
- `mindtape files [--db PATH] [--format table|json|csv]`
- `mindtape deps [--file PATH] [--db PATH] [--format table|json|csv]`
- `mindtape check <task-id> [--db PATH]` — toggle task checkbox
- `mindtape set <task-id> [--due DATE] [--no-due] [--add-tag TAG] [--remove-tag TAG] [--db PATH]`
- `mindtape id` — generate new UUIDv7 (base62 by default, `--raw` for hyphenated)
- `mindtape id <ID>` — validate an ID (accepts UUIDv7 or base62)

All commands support `-v`/`--verbose` (global, repeatable) and `--json` as shorthand for `--format json`.

## Context Hygiene (Self-Maintenance)

- Keep CLAUDE.md under 200 lines — it's auto-loaded every session
- Don't hardcode test counts in docs — they go stale (use `just count-tests`)
- After modifying any auto-loaded file, verify it hasn't grown beyond target size
- Don't duplicate content across CLAUDE.md, DESIGN.md, and MEMORY.md — keep it in one place
- Archive completed research to `archive/research/`
