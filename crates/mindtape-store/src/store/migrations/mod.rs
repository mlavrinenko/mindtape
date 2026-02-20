//! Schema migrations, one file per version.

mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;

use log::debug;
use rusqlite::Connection;

use super::StoreError;

/// Ordered list of migrations. Each entry is `(target_version, sql)`.
const MIGRATIONS: &[(i32, &str)] = &[
    (1, v1::SQL),
    (2, v2::SQL),
    (3, v3::SQL),
    (4, v4::SQL),
    (5, v5::SQL),
    (6, v6::SQL),
];

/// Apply all pending migrations to `conn`.
pub fn apply_migrations(conn: &Connection) -> Result<(), StoreError> {
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|e| StoreError::Migration(e.to_string()))?;

    debug!("database schema version: {version}");

    for &(target, sql) in MIGRATIONS {
        if version < target {
            debug!("migrating to schema v{target}");
            conn.execute_batch(sql)
                .map_err(|e| StoreError::Migration(e.to_string()))?;
            conn.pragma_update(None, "user_version", target)
                .map_err(|e| StoreError::Migration(e.to_string()))?;
        }
    }

    Ok(())
}
