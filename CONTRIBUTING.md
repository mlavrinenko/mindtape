# Contributing to MindTape

Thank you for your interest in contributing to MindTape! This document provides
guidelines for developers, maintainers, and AI agents working on the codebase.

## Quick Start

```bash
# Clone and enter nix shell
git clone <repo>
cd mindtape
nix develop

# Run checks (clippy + tests + file size limits)
just check

# Run all tests including integration tests
just all
```

## Development Workflow

### Prerequisites

- **Nix with flakes** enabled
- **Just** (task runner) — included in the Nix shell
- Rust toolchain is managed via Nix

### Common Commands

| Command | Purpose |
|---------|---------|
| `just check` | Clippy + tests + file size check (run before committing) |
| `just test` | Run tests only |
| `just clippy` | Clippy only |
| `just clippy-fix` | Auto-apply clippy suggestions |
| `just build` | Build the project |
| `just itest` | Shell integration tests |
| `just all` | Full suite (check + itest) |
| `just fmt` | Format code |
| `just count-tests` | Count total tests across workspace |

Use `just` commands instead of raw `cargo` — they handle `-q` flags and environment setup.

## Code Quality Standards

### File Size Limits

**Enforced by `just check-file-size`**:
- Rust files: **500 lines max** (exceptions: `sqlite.rs`, `watcher/tests.rs`)
- Markdown files: **200 lines max** (excluding `archive/`)

Rationale: keeps files focused, testable, and maintainable.

If a file exceeds the limit, refactor by extracting helpers or splitting modules.

### Clippy

- All clippy warnings **must** be fixed
- Run `just clippy-fix` to auto-apply mechanical fixes
- If a warning can't be fixed, use `#[allow(clippy::...)]` with a comment explaining why

### Testing

- Add tests for all new functionality
- Prefer unit tests over integration tests when possible
- See `docs/TESTING.md` for guidelines
- Current test count: run `just count-tests`

### Rust Conventions

See `CLAUDE.md` (project instructions for AI agents) for patterns:
- Workspace structure (3 crates: eval, store, CLI)
- Error handling via `anyhow` and `thiserror`
- Logging with `log` + `env_logger`
- Anti-corruption layers (Store trait, no typst types in sqlite.rs)

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]
```

Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`

Examples:
```
feat: add --sort support to list command
fix: handle missing task IDs in write-back
refactor: extract property-building helper
docs: update DESIGN.md with schema v7
```

## Schema Changes

**MindTape uses a drop-and-recreate strategy** (v7+) because the database is a
derived cache — Typst files are the source of truth.

### How to Change the Schema

1. **Delete** the current version file:
   ```bash
   rm crates/mindtape-store/src/store/migrations/v7.rs
   ```

2. **Create** the new version file:
   ```bash
   # Copy v7.rs as a starting point
   cp crates/mindtape-store/src/store/migrations/v7.rs \
      crates/mindtape-store/src/store/migrations/v8.rs
   
   # Edit v8.rs with your schema changes
   ```

3. **Update** `migrations/mod.rs`:
   ```rust
   mod v8;  // change from v7
   
   pub const CURRENT_SCHEMA_VERSION: i32 = 8;  // increment
   
   pub fn apply_migrations(conn: &Connection) -> Result<(), StoreError> {
       // ...
       conn.execute_batch(v8::SQL)  // change reference
   ```

4. **Update** test assertions:
   ```rust
   // In sqlite.rs tests
   assert_eq!(version, 8);  // was 7
   ```

5. **Update** docs:
   - `docs/design/store.md` — schema version reference
   - `crates/mindtape-store/CLAUDE.md` — version references
   - Add a note in this file's Schema Changes History section

6. **Run** `just check` to verify all tests pass

### Schema Changes History

- **v7** (2026-02): Denormalized `due`/`task_id` onto `tasks` table for SQL-side sorting

## Adding New Features

### CLI Commands

New commands go in `src/cli/commands/<name>.rs`:

1. Define an args struct with `#[derive(Parser)]`
2. Add a `run(&self) -> Result<()>` method
3. Add the command to the `Command` enum in `src/cli/mod.rs`
4. Match the command in `main.rs` and call `args.run()`

See `src/cli/commands/list.rs` for a reference implementation.

### Store Methods

To add a new query or data type:

1. Add domain types to `crates/mindtape-store/src/store/mod.rs`
2. Add method signature to the `Store` trait
3. Implement the method in `SqliteStore` (`sqlite.rs`)
4. Add tests to `sqlite.rs` (in the `mod tests` section)
5. Export the types from `crates/mindtape-store/src/lib.rs`

## Documentation

### Structure

- `README.md` — user-facing overview
- `CLAUDE.md` — project instructions for AI agents (auto-loaded)
- `CONTRIBUTING.md` — **this file** (contribution guidelines)
- `docs/DESIGN.md` — architecture overview
- `docs/TESTING.md` — testing guidelines
- `docs/design/*.md` — detailed design docs (eval, store, watcher, writeback)
- `crates/*/CLAUDE.md` — crate-specific context for AI agents

### Updating Docs

After implementing a feature:

1. Update `docs/DESIGN.md` if architecture changed
2. Update relevant crate `CLAUDE.md` files
3. Keep `CLAUDE.md` files under 200 lines (file size limit applies)
4. Archive old research/notes to `archive/research/`

## AI Agent Guidelines

If you're an AI agent (Claude, etc.) working on this codebase:

1. **Always run `just check` after code changes** and fix all warnings
2. **Use `just` recipes** instead of raw cargo commands
3. **Keep files small** — refactor if approaching 500/200 line limits
4. **Follow the todo workflow** — use TodoWrite tool for multi-step tasks
5. **Suggest commit messages** in conventional commit format after completing tasks
6. **Update DESIGN.md and CLAUDE.md** files when architecture changes
7. **After completing a task**, update docs and suggest a commit message

See `CLAUDE.md` for full AI agent instructions.

## Project Structure

```
mindtape/
├── Cargo.toml              # Workspace root + CLI binary crate
├── Justfile                # Development task runner
├── flake.nix               # Nix flake (dev shell + NixOS module)
├── CLAUDE.md               # AI agent instructions (auto-loaded)
├── CONTRIBUTING.md         # This file
├── docs/
│   ├── DESIGN.md           # Architecture overview
│   ├── TESTING.md          # Testing guidelines
│   └── design/             # Detailed design docs
├── src/                    # CLI binary crate
│   ├── main.rs             # Entry point + arg preprocessing
│   ├── cli/                # CLI arg definitions + commands
│   ├── config.rs           # TOML config loading
│   └── watcher/            # Folder watcher implementation
├── crates/
│   ├── mindtape-eval/      # Typst evaluation + task extraction
│   └── mindtape-store/     # Store trait + SQLite backend
├── tests/                  # Integration tests
├── lib/                    # Typst prelude (due, id, tag functions)
└── nix/
    └── module.nix          # NixOS module for systemd service
```

## Getting Help

- Check existing issues on GitHub
- Read `docs/DESIGN.md` for architecture overview
- Read crate-specific `CLAUDE.md` files for implementation details
- Use `just count-tests` to verify test count expectations

## License

See LICENSE file in the repository root.
