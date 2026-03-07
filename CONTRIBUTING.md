# Contributing to MindTape

## Quick Start

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop
# then fix something
# then make sure it's fine:
just check
```

## Common Commands

| Command | Purpose |
|---------|---------|
| `just check` | Clippy + tests + file size check (run before committing) |
| `just test` | Run tests only |
| `just clippy` | Clippy only |
| `just clippy-fix` | Auto-apply clippy suggestions |
| `just build` | Build the project |
| `just fmt` | Format code |
| `just cover` | Coverage report via cargo tarpaulin |
| `just count-tests` | Count total tests across workspace |

Use `just` recipes instead of raw `cargo` — they handle flags and environment setup.

## Code Quality

### File Size Limits (enforced by `just check-file-size`)

- Rust files: **500 lines** max
- Markdown files: **200 lines** max

If a file exceeds the limit, refactor by extracting modules or splitting content.

### Clippy

All warnings must be fixed. Run `just clippy-fix` for mechanical fixes.

## Testing

Coverage target is configured in `tarpaulin.toml`.

### Running Tests

```bash
just check          # clippy + all tests (preferred)
just test           # tests only
just cover          # coverage report via cargo tarpaulin
just count-tests    # show current test count
```

### Test Organization

**Unit tests** (`#[cfg(test)] mod tests` in source files) — test pure functions
with no I/O where possible.

**Integration tests** (`tests/` directory) — test the full pipeline end-to-end
with temporary files and databases. Shared setup lives in `tests/common.rs`.

### Writing Good Tests

- Test behavior, not implementation details
- Name tests descriptively: `test_<function>_<scenario>`
- Test edge cases: empty input, missing fields, boundary values
- Use `assert_eq!` for value comparisons (better failure messages)
- Use shared helpers from `tests/common.rs` for integration tests

## Commit Messages

[Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add --sort support to list command
fix: handle missing task IDs in write-back
refactor: extract property-building helper
docs: update store design doc
```

Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`

## Extending MindTape

### CLI Commands

New commands go in `src/cli/commands/<name>.rs`:

1. Define an args struct with `#[derive(Parser)]`
2. Add a `run(&self) -> Result<()>` method
3. Add the command to `Command` enum in `src/cli/mod.rs`
4. Match the command in `main.rs`

See `src/cli/commands/list.rs` for a reference implementation.

### Store Methods

1. Add domain types to `crates/mindtape-store/src/store/mod.rs`
2. Add method to the `Store` trait
3. Implement in `SqliteStore` (`sqlite.rs`)
4. Add tests, export types from `lib.rs`

### Schema Changes

MindTape uses drop-and-recreate (the DB is a derived cache). See
`crates/mindtape-store/AGENT.md` for schema details and the migration procedure.
