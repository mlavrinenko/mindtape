# MindTape Testing Guidelines

## Running Tests

```bash
cargo test                     # run all tests
cargo tarpaulin --skip-clean   # coverage report (text summary)
cargo tarpaulin --out html     # coverage report (open tarpaulin-report.html)

# Shell integration test (requires mindtape binary in PATH)
cd itest && PATH="../target/debug:$PATH" bash basic.sh
```

## Test Organization

### Unit tests (`#[cfg(test)] mod tests` in source files)

Test pure functions directly with no I/O where possible.

- `src/cli.rs` — arg parsing, task filtering/sorting, date formatting
- `src/config.rs` — TOML parsing, tilde expansion, path resolution
- `src/store.rs` — CRUD operations, query filters, hashing, schema migration
- `src/world.rs` — project root detection (uses `tempfile`), date utility,
  World construction

### Integration tests (`tests/` directory)

Test the full pipeline end-to-end with temporary files and databases.

- `tests/eval_integration.rs` — creates temp `.typ` files, evaluates through
  `MindTapeWorld` + `eval_file()`, asserts on extracted tasks
- `tests/store_integration.rs` — eval -> store pipeline: evaluates files and
  indexes them into an in-memory SQLite store
- `tests/watcher_integration.rs` — watcher initial scan and event handling
  with real filesystem operations

### Shell integration tests (`itest/` directory)

Test the compiled binary's CLI behavior with real `.typ` fixture files.

- `itest/basic.sh` — verifies `mindtape res/piano.typ --due -2` output

## Writing Good Tests

### DO

- Test behavior, not implementation details
- Name tests descriptively: `test_<function>_<scenario>`
- Test edge cases: empty input, missing fields, boundary values
- Keep tests independent (no shared mutable state between tests)
- Use `assert_eq!` for value comparisons (better failure messages)
- Use `eval_typ()` helper for integration tests that need Typst evaluation

### DON'T

- Don't write tests just to increase coverage numbers
- Don't test private implementation details that might change
- Don't use `#[should_panic]` when `Result`-based assertions work
- Don't create complex test fixtures when a simple string suffices

## Coverage

**Target**: 60%+ via `cargo tarpaulin` (do NOT use `-q` flag — not supported).

The current architecture makes 70-85% realistic because `main()` is a thin
wrapper (~20 lines, intentionally untested) and all logic lives in the
library crate.

### Current metrics (M1.4)

- **137 tests** (95 unit + 31 eval integration + 5 store integration + 6 watcher integration)
- **~79% coverage** via `cargo tarpaulin`

### What's covered

| Module | Coverage | Notes |
|--------|----------|-------|
| `src/cli.rs` | ~100% | Pure functions, fully unit tested |
| `src/eval.rs` | ~100% | Covered via integration tests |
| `src/config.rs` | ~100% | Unit tests for parsing, paths, defaults |
| `src/store.rs` | ~85% | Unit + integration tests (CRUD, queries, hashing) |
| `src/watcher.rs` | ~65% | Integration tests (scan, events); `run()` loop untested |
| `src/world.rs` | ~78% | Unit tests + integration tests |
| `src/main.rs` | 0% | Binary entry point, intentionally untested |

### What's NOT covered (and why)

- `main()` — just glue code with `process::exit()`, not worth testing
- `watcher::run()` — blocking notify event loop, tested indirectly via `initial_scan()` and `handle_event()`
- Some error paths in `world.rs` — filesystem edge cases that are hard
  to trigger reliably in tests

## Test Helpers

### `eval_typ(source: &str)` (in `tests/eval_integration.rs`)

Creates a temp directory with `Cargo.toml` marker and a `test.typ` file,
evaluates it, and returns `Result<Vec<Task>, String>`. Use for any test
that needs Typst evaluation:

```rust
#[test]
fn my_test() {
    let tasks = eval_typ("- [ ] Buy milk").unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Buy milk");
}
```

### `eval_typ_with_prelude(prelude, source)` (same file)

Same as `eval_typ` but also creates `lib/prelude.typ` in the temp dir,
for testing tasks that import `#due()` or `#id()`.

### `setup_watch_dir()` (in `tests/watcher_integration.rs`)

Creates a temporary directory with `Cargo.toml`, `lib/` directory, and
`.mindtapeignore` — the minimal structure for watcher tests. Returns a
`TempDir` to use as a project root.

### `setup()` (in `tests/store_integration.rs`)

Creates a watch directory via `setup_watch_dir()` and an in-memory
`SqliteStore`. Uses `dir.keep()` to prevent cleanup during debugging.
