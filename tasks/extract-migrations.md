# Extract migrations from sqlite.rs

## Problem

`crates/mindtape-store/src/store/sqlite.rs` is 1,776 lines and in the
file-size exception list. Migration code (schema constants + `migrate()`)
is 182 lines that will grow with every schema change.

## What to do

Extract migration code into `crates/mindtape-store/src/store/migrations.rs`.

### Move to `migrations.rs`

- 6 schema constants: `SCHEMA_V1` through `SCHEMA_V6` (lines 18-98)
- `fn apply_migrations(conn: &Connection) -> Result<(), StoreError>` — refactored
  from `SqliteStore::migrate()` (lines 149-201). Takes a `&Connection` instead
  of `&mut self` so it's a free function.

### Keep in `sqlite.rs`

- `SqliteStore` struct and `open()` / `open_memory()` / `init()`
- `init()` calls `migrations::apply_migrations(&self.conn)` instead of `self.migrate()`
- `build_task_query()`, `fetch_task_views()`, `fetch_file_deps()`
- All `impl Store` methods
- All `#[cfg(test)] mod tests` (tests exercise migrations indirectly via `open_memory()`)

### Wire it up

In `crates/mindtape-store/src/store/mod.rs`, add:
```rust
mod migrations;
```

In `sqlite.rs`, change `init()`:
```rust
fn init(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrations::apply_migrations(conn)
}
```

### Verify

- `just check` — all 274 tests pass, 0 clippy warnings
- No public API changes — `migrations` module stays private
- Remove `sqlite.rs` from the `EXCEPTIONS` list in `Justfile` if it drops
  under 500 lines (it should: 1776 - 182 = ~1594, still over; removing tests
  too would help but isn't necessary yet)

## Files to modify

1. `crates/mindtape-store/src/store/migrations.rs` (new, ~200 lines)
2. `crates/mindtape-store/src/store/sqlite.rs` (remove migration code, update init)
3. `crates/mindtape-store/src/store/mod.rs` (add `mod migrations;`)
