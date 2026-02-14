# mindtape-store

Store trait and SQLite backend. This crate defines the persistence layer
as an anti-corruption boundary — future backends (DuckDB, etc.) implement
the same `Store` trait.

## Responsibility

- Define domain types for persisted data (TaskFile, TaskRecord, etc.)
- Define the `Store` trait (anti-corruption layer)
- Implement `SqliteStore` with schema migrations
- Convert eval results to store records (`to_store_records`)
- Orchestrate file indexing (`index_file`: eval -> hash -> store)

## Source Layout

```
src/
  lib.rs           -- re-exports everything at crate root
  store/
    mod.rs         -- domain types, Store trait, StoreError
    sqlite.rs      -- SqliteStore impl, schema, migrations, tests
    indexer.rs     -- hash_file(), to_store_records(), index_file()
```

## Key Types

- `Store` trait — 8 methods (upsert, query, remove, stats)
- `SqliteStore` — SQLite implementation with schema v1
- `TaskFile`, `TaskRecord`, `TaskProperty`, `FileBinding` — domain types
- `TaskFilter` — query filters (done, tag, due_before, file_path, folder, limit)
- `TaskView`, `FileView`, `IndexStats` — query result types
- `StoreError` — error enum (Db, Migration, Path, Io, Eval)

## Technical Notes

- Schema v1 with `PRAGMA foreign_keys = ON` and cascading deletes
- SHA-256 hash-based skip for unchanged files (`hash_file`)
- `index_file()` depends on `mindtape-eval` for evaluation
- Dynamic dispatch: `index_file` takes `&mut dyn Store`

## Dependencies

- `mindtape-eval` (for `EvalResult`, `eval_file_full`, `World`)
- `typst` (for `World` trait in `index_file` signature)
- `rusqlite`, `sha2`, `thiserror`
