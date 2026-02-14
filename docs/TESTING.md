# MindTape Testing Guidelines

## Running Tests

```bash
just check          # clippy + all tests (preferred)
just test           # tests only
just cover          # coverage report via cargo tarpaulin
just itest          # shell integration tests (builds first)
just all            # check + itest
just count-tests    # show current test count
```

## Test Organization

### Unit tests (`#[cfg(test)] mod tests` in source files)

Test pure functions directly with no I/O where possible.

- `src/cli.rs` — arg parsing, task filtering/sorting, date formatting
- `src/config.rs` — TOML parsing, tilde expansion, path resolution
- `crates/mindtape-store/src/store/sqlite.rs` — CRUD operations, query filters, hashing, schema migration
- `crates/mindtape-eval/src/world.rs` — project root detection, World construction

### Integration tests (`tests/` directory)

Test the full pipeline end-to-end with temporary files and databases.
Shared setup lives in `tests/common.rs` (`setup_typst_project()`, `ymd()`).

- `tests/eval_integration.rs` — creates temp `.typ` files, evaluates, asserts on extracted tasks
- `tests/store_integration.rs` — eval -> store pipeline with in-memory SQLite
- `tests/query_integration.rs` — query commands (list_files, get_stats, folder filter)
- `tests/watcher_integration.rs` — watcher initial scan and event handling

### Shell integration tests (`itest/` directory)

Test the compiled binary's CLI behavior with real `.typ` fixture files.

- `itest/basic.sh` — end-to-end tests for eval + query commands

## Writing Good Tests

### DO

- Test behavior, not implementation details
- Name tests descriptively: `test_<function>_<scenario>`
- Test edge cases: empty input, missing fields, boundary values
- Keep tests independent (no shared mutable state between tests)
- Use `assert_eq!` for value comparisons (better failure messages)
- Use shared helpers from `tests/common.rs` for integration tests

### DON'T

- Don't write tests just to increase coverage numbers
- Don't test private implementation details that might change
- Don't use `#[should_panic]` when `Result`-based assertions work
- Don't create complex test fixtures when a simple string suffices
- Don't hardcode test counts in documentation (they go stale — use `just count-tests`)

## Coverage

**Target**: 60%+ via `cargo tarpaulin` (do NOT use `-q` flag — not supported).

The architecture makes 70-85% realistic because `main()` is a thin wrapper
(~20 lines, intentionally untested) and all logic lives in the library crate.
