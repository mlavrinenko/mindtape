# mindtape-store

Store trait and SQLite backend. This crate defines the persistence layer
as an anti-corruption boundary — future backends (DuckDB, etc.) implement
the same `Store` trait.

## Responsibility

- Define domain types for persisted data (TaskFile, TaskRecord, etc.)
- Define the `Store` trait (anti-corruption layer)
- Implement `SqliteStore` with schema migrations (currently v6)
- Convert eval results to store records (`to_store_records`)
- Orchestrate file indexing (`index_file`: eval -> hash -> store)
- Generate and parse task IDs (UUIDv7, base62)

## Source Layout

```
src/
  lib.rs           -- re-exports everything at crate root
  id.rs            -- UUIDv7 generation, base62 encoding, ID parsing/validation
  store/
    mod.rs         -- domain types, Store trait, StoreError
    sqlite.rs      -- SqliteStore impl, schema v6, migrations, tests
    indexer.rs     -- hash_file(), to_store_records(), index_file(), index_file_with_deps()
```

## Key Types

- `Store` trait — 13 methods (upsert, query, remove, stats, dependencies, find)
- `SqliteStore` — SQLite implementation with schema v6
- `TaskFile`, `TaskRecord`, `TaskProperty`, `PropertyKind`, `FileBinding` — domain types
- `TaskFilter` — query filters (done, tag, due_before/after, search, file_path, folder, limit)
- `TaskView`, `FileView`, `IndexStats` — query result types
- `FileDependencies`, `FileReference` — dependency graph types
- `TaskWithFile` — task + file path for write-back lookup
- `StoreError` — error enum (Db, Eval, FileSystem, NotFound, Conflict)

## Store Trait Methods

- `upsert_task_file`, `remove_task_file`, `get_file_hash` — file management
- `upsert_tasks`, `upsert_bindings` — task and binding storage
- `query_tasks`, `list_files`, `get_stats` — queries
- `upsert_file_references`, `get_file_dependencies`, `list_file_dependencies` — deps
- `find_task_by_id` — lookup by UUID, base62, or `*suffix` mask

## Schema History

- v1: Core tables (task_files, tasks, task_properties, file_bindings)
- v2: COLLATE NOCASE indexes
- v3: file_references table for dependency tracking
- v4: Added `milestone TEXT` column to tasks
- v5: `relative_path` → `file_path` (absolute), added `watch_root TEXT`
- v6: FTS5 virtual table `tasks_fts` (title, milestone) with auto-sync triggers

## Technical Notes

- `PRAGMA foreign_keys = ON` with cascading deletes
- SHA-256 hash-based skip for unchanged files (`hash_file`)
- `index_file()` depends on `mindtape-eval` for evaluation
- `index_file_with_deps()` variant uses `MindTapeWorld` for dependency tracking
- FTS5 `tasks_fts` enables full-text search via `TaskFilter::search`

## Dependencies

- `mindtape-eval` (for `EvalResult`, `eval_file_full`, `World`)
- `typst` (for `World` trait in `index_file` signature)
- `rusqlite`, `sha2`, `uuid`, `base62`, `thiserror`, `log`
