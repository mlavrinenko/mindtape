# MindTape — Project Context

## What is this?

A file-based task tracker that uses Typst (`.typ`) files as the source of truth.
Watches folders, evaluates Typst files via the compiler crates, indexes tasks and
metadata into a database, and exposes a CLI (later API) for querying.

## Key Docs

- `docs/DESIGN.md` — architecture, data model, Typst evaluation details
- `docs/ROADMAP.md` — MVP milestones, future plans, non-goals
- `docs/TESTING.md` — testing guidelines, coverage targets

## Tech Stack

- **Language**: Rust
- **Build**: Nix flake (`flake.nix`) with `naersk`
- **Typst evaluation**: `typst`, `typst-eval`, `typst-library`, `typst-syntax`
- **Comemo**: `comemo = "0.5"` (must match typst 0.14's version)
- **Database**: SQLite via `rusqlite` (behind a `Store` trait for future swapability)
- **File watching**: `notify` + `notify-debouncer-mini` (300ms debounce)
- **Ignore patterns**: `ignore` crate for `.mindtapeignore` (gitignore-style)
- **CLI**: Manual arg parsing (not clap) for `-N` shorthand support
- **Testing**: `cargo test` + `cargo tarpaulin` for coverage
- **Config**: `toml` + `serde` for TOML config files

## Agent Rules

- Use `--quiet` / `-q` for `cargo build`, `cargo test`, `cargo tarpaulin`, etc. — only show errors/warnings and test results, not compilation progress
- After any code changes, run `cargo clippy --all-targets -q` and fix all warnings before considering the task done
- Avoid dumping large tool outputs into context; summarize or truncate when possible
- At the end of a session with code/config changes, suggest a conventional commit message (e.g. `feat:`, `fix:`, `refactor:`, `chore:`, `docs:`)
- Save research findings (external crate APIs, googled solutions, version-specific quirks) to `docs/research/` as markdown files — one file per topic (e.g. `docs/research/notify-crate.md`). This preserves knowledge across sessions and avoids re-researching the same things.

## Architecture Principles

- Typst files are always the source of truth — the database is a derived index
- Evaluation-only: we use `typst-eval::eval()`, never layout or render
- Anti-corruption layer: `Store` trait abstracts the database
- Single binary crate with lib.rs for testability
- All pure logic is testable; main.rs is a thin CLI wrapper

## Source Layout

```
src/
  lib.rs        -- pub mod declarations (cli, config, eval, store, watcher, world)
  main.rs       -- thin CLI entry point, routes eval/watch/list/status/files commands
  cli.rs        -- Command enum, arg parsing, task filtering/sorting, formatting
  config.rs     -- TOML config loading, WatchEntry, tilde expansion
  eval.rs       -- Task struct, eval_file(), content tree traversal
  store.rs      -- Store trait, SqliteStore, index_file(), domain types
  watcher.rs    -- Watcher struct, initial_scan, handle_event, run (notify loop)
  world.rs      -- MindTapeWorld (World trait impl), project root detection

tests/
  eval_integration.rs     -- end-to-end eval tests with temp .typ files
  store_integration.rs    -- eval -> store pipeline tests
  query_integration.rs    -- query command tests (list_files, get_stats, folder filter)
  watcher_integration.rs  -- watcher scan + event handling tests

lib/
  prelude.typ   -- due(), id(), tag() functions using metadata()
  typst.toml    -- package manifest for @mindtape/mindtape:0.1.0

itest/
  basic.sh      -- shell integration test
  res/piano.typ -- test fixture
```

## Data Flow

```
.typ files -> Typst eval -> Module (scope + content) -> extract tasks/bindings -> Store -> CLI queries
```

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

- Run tests: `cargo test`
- Coverage: `cargo tarpaulin` (target: 60%+, currently ~79%)
- Shell integration test: `cd itest && PATH="../target/debug:$PATH" bash basic.sh`
- See `docs/TESTING.md` for full guidelines

## Current Status

MVP complete (M1.1–M1.5). Full read-only workflow: evaluate Typst files, index
into SQLite, query via CLI. CLI commands:
- `mindtape <file.typ> [--due] [-N]` — eval a single file
- `mindtape watch [<path>] [--config <file>]` — watch and index folders
- `mindtape list [--status done|pending|all] [--tag TAG] [--due-before DATE] [--file PATH] [--folder PREFIX] [-N] [--db PATH]`
- `mindtape status [--db PATH]` — index stats
- `mindtape files [--db PATH]` — list indexed files

177 tests. Next: Milestone 2 (Richer Queries + UX).
