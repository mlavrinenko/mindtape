# mindtape-store

Store trait and SQLite backend. Anti-corruption boundary for persistence.

## Store Trait

13 methods in 5 groups:

- **File management**: `upsert_task_file`, `remove_task_file`, `get_file_hash`
- **Data storage**: `upsert_tasks`, `upsert_bindings`
- **Queries**: `query_tasks`, `list_files`, `get_stats`
- **Dependencies**: `upsert_file_references`, `get_file_dependencies`, `list_file_dependencies`
- **Task lookup**: `find_task_by_id` (UUID, base62, or `*suffix` mask)

## Schema Change Procedure

1. Delete current migration (e.g. `migrations/v7.rs`)
2. Create next version (copy as starting point)
3. Update `migrations/mod.rs`: bump version, update `execute_batch` call
4. Update test assertions in `sqlite.rs`
5. Run `just check`

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
