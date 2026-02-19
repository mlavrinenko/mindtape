# Split watcher into module directory

## Problem

`src/watcher.rs` is 474 lines (under the 500 limit but tight) with 6 distinct
responsibilities. `src/watcher_tests.rs` is 626 lines and in the exception list.
Adding features (e.g., batch operations, smarter debounce) will push it over.

## What to do

Convert `src/watcher.rs` + `src/watcher_tests.rs` into `src/watcher/` module dir.

### Proposed layout

```
src/watcher/
  mod.rs       -- Watcher struct, new(), initial_scan(), run() (~80 lines)
  scan.rs      -- scan_entries(), index_one(), is_typ_file() (~50 lines)
  events.rs    -- handle_event(), is_inside_dotgit() (~45 lines)
  ignore.rs    -- build_ignore(), global_gitignore_path() (~65 lines)
  config.rs    -- add_entry(), remove_entries_not_in(), reload_config() (~60 lines)
  entry.rs     -- ResolvedEntry, resolve_entry() (~15 lines)
  errors.rs    -- WatchError, ScanResult (~25 lines)
  tests.rs     -- all 45+ tests from watcher_tests.rs
```

### Key decisions

- `mod.rs` holds the `Watcher` struct and delegates to submodules
- Methods like `scan_entries()` become `impl Watcher` methods in their
  respective files (Rust allows `impl` blocks across files in same crate)
- Free functions (`build_ignore`, `resolve_entry`, etc.) stay as free functions
  in their submodule
- `errors.rs` exports `WatchError` and `ScanResult` — used by all submodules
- `tests.rs` replaces `watcher_tests.rs` (remove the old file)

### Alternatively (simpler)

If the full split feels premature, just split into 3 files:
```
src/watcher/
  mod.rs       -- Watcher struct, public API, event loop, scanning
  ignore.rs    -- build_ignore(), global_gitignore_path()
  tests.rs     -- all tests
```

This isolates the most complex standalone logic (ignore patterns) and moves
tests out of the exception list.

### Wire it up

- `src/lib.rs` already has `pub mod watcher;` — no change needed (Rust resolves
  `watcher.rs` or `watcher/mod.rs` the same way)
- Delete `src/watcher.rs` and `src/watcher_tests.rs`
- Remove `src/watcher_tests.rs` from the `EXCEPTIONS` list in `Justfile`

### Verify

- `just check` passes
- `just itest` passes (watcher is exercised in shell integration tests)

## Files to modify

1. `src/watcher/mod.rs` (new, from watcher.rs)
2. `src/watcher/ignore.rs` (new, extracted)
3. `src/watcher/tests.rs` (new, from watcher_tests.rs)
4. Delete `src/watcher.rs` and `src/watcher_tests.rs`
5. `Justfile` — remove `src/watcher_tests.rs` from EXCEPTIONS if applicable
