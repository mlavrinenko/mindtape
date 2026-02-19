# Store Architecture

## Overview

The Store trait provides an anti-corruption layer between the indexing pipeline
and the database backend. SQLite is the current implementation, but the trait
enables future backends like DuckDB.

## Store Trait

See `crates/mindtape-store/src/store/mod.rs` for the trait definition and domain types.

13 methods in 5 groups:

- **File management**: `upsert_task_file`, `remove_task_file`, `get_file_hash`
- **Data storage**: `upsert_tasks`, `upsert_bindings`
- **Queries**: `query_tasks`, `list_files`, `get_stats`
- **Dependencies**: `upsert_file_references`, `get_file_dependencies`, `list_file_dependencies`
- **Task lookup**: `find_task_by_id` (UUID, base62, or `*suffix` mask)

## Core Entities

See `crates/mindtape-store/src/store/mod.rs` for full type definitions.

### Database Tables

```
TaskFile
  id, file_path (absolute), watch_root, title, updated_at, eval_hash

Task
  id, task_file_id, title, is_done, position, milestone

TaskProperty
  id, task_id, kind (due | tag | id | custom), key, value

FileBinding
  id, task_file_id, name, value_type, value_json

FileReferences
  id, source_file_id, target_path

TasksFts (FTS5 virtual table)
  title, milestone — auto-synced via triggers
```

### Domain Types

- `TaskView` — denormalized task with file path, folder, properties
- `FileView` — file metadata with task counts
- `IndexStats` — aggregate statistics
- `FileDependencies` — bidirectional import graph
- `TaskWithFile` — task + file path for write-back lookup

All views derive `Serialize` for `--json` output.

## Schema

See `crates/mindtape-store/src/store/sqlite.rs` for migrations and schema.

Current version: **v6** (see crate CLAUDE.md for full migration history)

Key constraints:
- `PRAGMA foreign_keys = ON` for referential integrity
- `CASCADE DELETE` on foreign keys for automatic cleanup
- Indexes on query-critical columns (`file_path`, `target_path`, etc.)

## Indexing Pipeline

See `crates/mindtape-store/src/store/indexer.rs` for implementation.

### Hash-Based Skip

```rust
fn index_file(path: &Path, store: &mut dyn Store) -> Result<()>
```

1. Compute SHA-256 hash of file content
2. Query stored hash via `get_file_hash()`
3. If hashes match, skip re-indexing
4. Otherwise: eval → convert → upsert

### Record Conversion

```rust
fn to_store_records(eval_result: &EvalResult) -> StoreRecords
```

Converts `EvalResult` (from typst-eval) into database records:
- `TaskRecord` with position tracking
- `TaskProperty` for due dates, tags, IDs
- `FileBinding` for scope exports

### Cross-File References

```rust
fn index_file_with_deps(path: &Path, store: &mut dyn Store) -> Result<()>
```

Extended indexing that tracks file dependencies:
1. Eval file with `eval_file_full_with_deps()`
2. Convert tasks/bindings as usual
3. Upsert file references via `upsert_file_references()`

## SQLite Implementation

See `crates/mindtape-store/src/store/sqlite.rs` for full implementation.

### Key Features

- Migrations system for schema versioning
- Transaction-based upserts for consistency
- Efficient queries with prepared statements
- Comprehensive unit tests (42 tests in sqlite.rs)

## Error Handling

See `crates/mindtape-store/src/store/mod.rs` for `StoreError` definition.

```rust
pub enum StoreError {
    #[from]
    Eval(EvalError),  // Propagates eval errors with ?
    Db(String),
    FileSystem(String),
    NotFound(String),
    Conflict(String),
}
```

Clean propagation from eval crate via `#[from]` attribute.
