# Add --watch-root filter to list command

## Problem

`watch_root` is stored per file in the DB (`task_files.watch_root` column,
added in schema v5) and returned in `TaskView::watch_root` for display.
But there's no way to filter by it from the CLI. When watching multiple
roots (e.g., `~/work/` and `~/personal/`), users can't scope queries.

`--folder` does prefix matching on `file_path`, which is close but
requires knowing the full absolute path. `--watch-root` would match
against the configured watch entry path.

## What to do

### 1. Add field to TaskFilter

In `crates/mindtape-store/src/store/mod.rs`, add to `TaskFilter`:
```rust
pub watch_root: Option<PathBuf>,
```

### 2. Add SQL clause in SqliteStore

In `crates/mindtape-store/src/store/sqlite.rs`, in `build_task_query()`:
```rust
if let Some(ref root) = filter.watch_root {
    conditions.push("tf.watch_root = ?");
    params.push(root.to_string_lossy().to_string());
}
```

### 3. Add CLI flag to ListArgs

In `src/cli/commands/list.rs`, add to `ListArgs`:
```rust
/// Filter by watch root path
#[arg(long)]
pub watch_root: Option<PathBuf>,
```

And in the filter construction in `run()`:
```rust
watch_root: self.watch_root.clone(),
```

### 4. Update all TaskFilter construction sites

Grep for `TaskFilter {` — every place that builds one needs the new field.
Currently:
- `src/cli/commands/list.rs` (main one)
- `tests/query_integration.rs` (test helpers)
- `tests/store_integration.rs` (test helpers)

Add `watch_root: None` to all test sites (they don't use this filter).

### 5. Update CLAUDE.md CLI section

Add `--watch-root PATH` to the `list` command help line.

### Verify

- `just check` passes
- Add a test in `tests/query_integration.rs` that indexes files with
  different watch roots, then filters by one

## Files to modify

1. `crates/mindtape-store/src/store/mod.rs` — TaskFilter field
2. `crates/mindtape-store/src/store/sqlite.rs` — SQL WHERE clause
3. `src/cli/commands/list.rs` — CLI arg + filter wiring
4. `tests/query_integration.rs` — new test + update existing TaskFilter sites
5. `tests/store_integration.rs` — update TaskFilter sites
6. `CLAUDE.md` — CLI docs
