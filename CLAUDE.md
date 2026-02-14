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
- **Database**: SQLite (planned, behind a `Store` trait for future swapability)
- **File watching**: `notify` crate (planned)
- **CLI**: Manual arg parsing (not clap) for `-N` shorthand support
- **Testing**: `cargo test` + `cargo tarpaulin` for coverage
- **Config**: TOML (planned)

## Agent Rules

- Use `--quiet` / `-q` for `cargo build`, `cargo test`, `cargo tarpaulin`, etc. — only show errors/warnings and test results, not compilation progress
- Avoid dumping large tool outputs into context; summarize or truncate when possible
- At the end of a session with code/config changes, suggest a conventional commit message (e.g. `feat:`, `fix:`, `refactor:`, `chore:`, `docs:`)

## Architecture Principles

- Typst files are always the source of truth — the database is a derived index
- Evaluation-only: we use `typst-eval::eval()`, never layout or render
- Anti-corruption layer: `Store` trait abstracts the database
- Single binary crate with lib.rs for testability
- All pure logic is testable; main.rs is a thin CLI wrapper

## Source Layout

```
src/
  lib.rs        -- pub mod declarations (cli, eval, world)
  main.rs       -- thin CLI entry point
  cli.rs        -- arg parsing, task filtering/sorting, output formatting
  eval.rs       -- Task struct, eval_file(), content tree traversal
  world.rs      -- MindTapeWorld (World trait impl), project root detection

tests/
  eval_integration.rs  -- end-to-end eval tests with temp .typ files

lib/
  prelude.typ   -- due(), id() functions using metadata()

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
#import "lib/prelude.typ": due, id

= Milestone Title

- [ ] task text #due(datetime(...)) #id("uuid")
- [x] completed task

#let some_binding = "searchable value"
```

- Headings = milestones / groups
- Checklist items = tasks (parsed from `plain_text()`, not native Typst)
- `#due()`, `#id()` = task properties (produce `MetadataElem` via `metadata()`)
- `#let` bindings = file-level metadata, indexed for search

## Testing

- Run tests: `cargo test`
- Coverage: `cargo tarpaulin` (target: 60%+, currently ~78%)
- Shell integration test: `cd itest && PATH="../target/debug:$PATH" bash basic.sh`
- See `docs/TESTING.md` for full guidelines

## Current Status

M1.1 complete. PoC evaluates `.typ` files, extracts tasks with due dates,
filters/sorts/limits, prints formatted output. Code refactored into lib+bin
crate with 35 tests and 77%+ coverage.
Next: M1.2 (tag function, `@mind-tape` package resolution).
