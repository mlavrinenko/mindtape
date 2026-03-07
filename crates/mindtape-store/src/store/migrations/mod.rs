//! Schema migrations.
//!
//! The database is a derived cache (Typst files are the source of truth).
//! When the schema version doesn't match, we drop everything and recreate
//! from scratch — the watcher will re-index on next startup.

mod v9;

use log::{debug, info};
use rusqlite::Connection;

use super::StoreError;

/// Current schema version. Bump this when the schema changes.
pub const CURRENT_SCHEMA_VERSION: i32 = 9;

/// Apply migrations: if the DB version doesn't match, drop and recreate.
pub fn apply_migrations(conn: &Connection) -> Result<(), StoreError> {
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|e| StoreError::Migration(e.to_string()))?;

    debug!("database schema version: {version}");

    if version == CURRENT_SCHEMA_VERSION {
        return Ok(());
    }

    if version != 0 {
        info!("schema version {version} != {CURRENT_SCHEMA_VERSION}, recreating cache database");
    }

    conn.execute_batch(v9::SQL)
        .map_err(|e| StoreError::Migration(e.to_string()))?;
    conn.pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)
        .map_err(|e| StoreError::Migration(e.to_string()))?;

    Ok(())
}
