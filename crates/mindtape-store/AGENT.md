# mindtape-store

Store trait and SQLite backend. Anti-corruption boundary for persistence.

For architecture details see [docs/design/store.md](../../docs/design/store.md).

## Source Layout

```
src/
  lib.rs           -- re-exports everything at crate root
  id.rs            -- UUIDv7 generation, base62 encoding, ID parsing/validation
  store/
    mod.rs         -- domain types, Store trait, StoreError
    sqlite.rs      -- SqliteStore impl, schema v7, tests
    migrations/    -- v7.rs (single clean schema, drop-and-recreate)
    indexer.rs     -- hash_file(), to_store_records(), index_file(), index_file_with_deps()
```

## Technical Notes

- `PRAGMA foreign_keys = ON` with cascading deletes
- SHA-256 hash-based skip for unchanged files (`hash_file`)
- `index_file()` depends on `mindtape-eval` for evaluation
- FTS5 `tasks_fts` enables full-text search via `TaskFilter::search`
- `due` and `task_id` denormalized onto `tasks` table for efficient SQL sorting

## Dependencies

- `mindtape-eval` (for `EvalResult`, `eval_file_full`, `World`)
- `typst` (for `World` trait in `index_file` signature)
- `rusqlite`, `sha2`, `uuid`, `base62`, `thiserror`, `log`
