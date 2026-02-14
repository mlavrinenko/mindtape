# Store Architecture

## Overview

The Store trait provides an anti-corruption layer between the indexing pipeline
and the database backend. SQLite is the current implementation, but the trait
enables future backends like DuckDB.

## Store Trait

See `crates/mindtape-store/src/store/mod.rs` for the trait definition and domain types.

Core operations:
- `upsert_task_file` — insert or update file metadata
- `upsert_tasks` — insert or update tasks for a file
- `upsert_bindings` — insert or update file-level bindings
- `upsert_file_references` — track cross-file imports
- `remove_task_file` — delete file and cascading data
- `query_tasks` — filtered task queries
- `get_file_hash` — retrieve stored hash for conflict detection
- `list_files` — get all indexed files
- `get_stats` — index statistics
- `search` — full-text search across tasks and bindings
- `query_agenda` — overdue/today/week task views
- `get_file_dependencies` — dependency graph for a file
- `list_file_dependencies` — all file dependencies

## Core Entities

See `crates/mindtape-store/src/store/mod.rs` for full type definitions.

### Database Tables

```
WatchedFolder
  id, path, created_at

TaskFile
  id, watched_folder_id, relative_path, title, updated_at, eval_hash

Task
  id, task_file_id, title, is_done, position

TaskProperty
  id, task_id, kind (due | tag | id | custom), key, value

FileBinding
  id, task_file_id, name, value_type, value_json

FileReferences (v3)
  id, source_file_id, target_path
```

### Domain Types

- `TaskView` — denormalized task with file path, folder, properties
- `FileView` — file metadata with task counts
- `IndexStats` — aggregate statistics
- `BindingView` — binding with file context
- `SearchResults` — combined task + binding search results
- `AgendaView` — tasks grouped by time bucket (overdue/today/week)
- `FileDependencies` — bidirectional import graph

All views derive `Serialize` for `--json` output.

## Schema

See `crates/mindtape-store/src/store/sqlite.rs` for migrations and schema.

Current version: **v3**

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

### Search Implementation

Full-text search uses `LIKE` queries across:
- Task titles
- Task property values
- Binding names and values

Case-insensitive, matches substrings.

### Agenda Queries

Time-based filtering:
- **Overdue**: `due_date < today`
- **Today**: `due_date = today`
- **Week**: `due_date > today AND due_date <= today + 6 days`

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
