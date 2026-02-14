# MindTape — Project Context

## What is this?

A file-based task tracker that uses Typst (`.typ`) files as the source of truth.
Watches folders, evaluates Typst files via the compiler crates, indexes tasks and
metadata into a database, and exposes a CLI (later API) for querying.

## Key Docs

- `README.md` — what it is, future plans, non-goals
- `docs/DESIGN.md` — architecture, data model, Typst evaluation details
- `docs/ROADMAP.md` — MVP milestones, future plans, non-goals
- `docs/TESTING.md` — testing guidelines, coverage targets
- `crates/mindtape-eval/CLAUDE.md` — eval crate context
- `crates/mindtape-store/CLAUDE.md` — store crate context

## Tech Stack

- **Language**: Rust (workspace with 3 crates)
- **Build**: Nix flake (`flake.nix`) with `naersk`, `just` for task runner
- **Typst evaluation**: `typst`, `typst-eval`, `typst-library`, `typst-syntax`
- **Comemo**: `comemo = "0.5"` (must match typst 0.14's version)
- **Database**: SQLite via `rusqlite` (behind a `Store` trait for future swapability)
- **File watching**: `notify` + `notify-debouncer-mini` (300ms debounce)
- **Ignore patterns**: `ignore` crate for `.mindtapeignore` (gitignore-style)
- **CLI**: Manual arg parsing (not clap) for `-N` shorthand support
- **Serialization**: `serde` + `serde_json` for `--json` output
- **Testing**: `cargo test` + `cargo tarpaulin` for coverage
- **Config**: `toml` + `serde` for TOML config files

## Agent Rules

- Use `just` recipes instead of raw cargo commands (see `Justfile` for available recipes)
- Use `--quiet` / `-q` for `cargo build`, `cargo test`, `cargo tarpaulin`, etc. — only show errors/warnings and test results, not compilation progress
- After any code changes, run `just check` (clippy + tests) and fix all warnings before considering the task done
- Always improve the `Justfile` when you notice missing or useful recipes
- Avoid dumping large tool outputs into context; summarize or truncate when possible
- At the end of a session with code/config changes, suggest a conventional commit message (e.g. `feat:`, `fix:`, `refactor:`, `chore:`, `docs:`)
- Save research findings (external crate APIs, googled solutions, version-specific quirks) to `docs/research/` as markdown files — one file per topic (e.g. `docs/research/notify-crate.md`). This preserves knowledge across sessions and avoids re-researching the same things.
- When working on a single crate, read that crate's `CLAUDE.md` for focused context.

## Architecture Principles

- Typst files are always the source of truth — the database is a derived index
- Evaluation-only: we use `typst-eval::eval()`, never layout or render
- Anti-corruption layer: `Store` trait abstracts the database
- Workspace with focused crates; root crate is the CLI binary
- All pure logic is testable; main.rs is a thin CLI wrapper

## Workspace Layout

```
Cargo.toml          -- workspace root + root crate (mindtape CLI binary)
Justfile            -- development recipes (check, test, clippy, build, cover, itest, all, fmt)

crates/
  mindtape-eval/    -- Typst evaluation + task extraction (~600 LOC)
    src/
      lib.rs        -- re-exports eval public API
      eval.rs       -- EvalError, Task, EvalResult, eval_file(), content traversal
      world.rs      -- MindTapeWorld (World trait impl), project root detection
    CLAUDE.md       -- crate-specific context

  mindtape-store/   -- Store trait + SQLite backend (~1,400 LOC)
    src/
      lib.rs        -- re-exports store public API
      store/
        mod.rs      -- domain types, Store trait, StoreError
        sqlite.rs   -- SqliteStore impl, schema, migrations, tests
        indexer.rs  -- hash_file(), to_store_records(), index_file()
    CLAUDE.md       -- crate-specific context

src/                -- root crate: CLI binary (~1,700 LOC)
  lib.rs            -- re-exports sub-crates (eval, store, world) + local modules (cli, config, watcher)
  main.rs           -- thin CLI entry point, command routing
  cli.rs            -- Command enum, arg parsing, formatting, --json support
  config.rs         -- TOML config loading, WatchEntry, tilde expansion
  watcher.rs        -- Watcher struct, initial_scan, handle_event, run (notify loop)

tests/              -- integration tests (root crate level)
  eval_integration.rs     -- end-to-end eval tests with temp .typ files
  store_integration.rs    -- eval -> store pipeline tests
  query_integration.rs    -- query command tests (list_files, get_stats, folder filter)
  watcher_integration.rs  -- watcher scan + event handling tests

lib/                -- Typst package files
  prelude.typ       -- due(), id(), tag() functions using metadata()
  typst.toml        -- package manifest for @mindtape/mindtape:0.1.0

itest/              -- shell integration tests
  basic.sh          -- 8 end-to-end tests (eval + query commands)
  res/piano.typ     -- test fixture
```

## Dependency Graph

```
mindtape-eval        (typst, typst-eval, typst-library, typst-syntax, comemo, thiserror)
       |
       v
mindtape-store       (mindtape-eval, typst, rusqlite, sha2, serde, thiserror)
       |
       v
mindtape (root)      (mindtape-eval, mindtape-store, typst, thiserror,
                      notify, ignore, toml, serde, serde_json)
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

- Run all checks: `just check` (clippy + tests)
- Run tests only: `just test`
- Coverage: `just cover` (target: 60%+)
- Shell integration test: `just itest`
- Everything: `just all`
- See `docs/TESTING.md` for full guidelines

## Current Status

MVP complete (M1.1-M1.5), M2.1 (search + output formats) complete, M2.2 (agenda) complete.
153 tests across workspace. Next: M2.3 (Cross-File References).

CLI commands:
- `mindtape <file.typ> [--due] [-N]` — eval a single file
- `mindtape watch [<path>] [--config <file>]` — watch and index folders
- `mindtape list [--status done|pending|all] [--tag TAG] [--due-before DATE] [--file PATH] [--folder PREFIX] [-N] [--db PATH] [--format table|json|csv]`
- `mindtape search <keyword> [-N] [--db PATH] [--format table|json|csv]` — full-text search
- `mindtape agenda [--overdue] [--today] [--week] [-N] [--db PATH] [--format table|json|csv]` — show overdue/today/week tasks
- `mindtape status [--db PATH] [--format table|json|csv]` — index stats
- `mindtape files [--db PATH] [--format table|json|csv]` — list indexed files

All query commands support `--json` as shorthand for `--format json`.
