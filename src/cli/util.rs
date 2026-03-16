use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config;
use crate::store::SqliteStore;

/// Resolve the path to the query database.
///
/// Uses the explicit `--db` override if given, otherwise resolves from
/// config auto-discovery or the default path.
pub fn resolve_query_db_path(db_override: Option<&Path>) -> std::path::PathBuf {
    if let Some(p) = db_override {
        p.to_path_buf()
    } else {
        let cfg = config::find_config()
            .and_then(|p| config::load_config(&p).ok())
            .unwrap_or(config::Config {
                database: None,
                watch: vec![],
                agenda: vec![],
            });
        config::resolve_db_path(&cfg)
    }
}

/// Open the `SQLite` database for query commands.
///
/// # Errors
/// Returns error if database doesn't exist or can't be opened.
pub fn open_query_db(db_path: &Path) -> Result<SqliteStore> {
    if !db_path.exists() {
        anyhow::bail!(
            "database not found: {}\nRun 'mindtape watch' first to create the index.",
            db_path.display()
        );
    }

    SqliteStore::open(db_path)
        .with_context(|| format!("failed to open database at {}", db_path.display()))
}

/// Print value as JSON to stdout.
///
/// # Errors
/// Returns error if JSON serialization fails.
pub fn print_json(value: &impl serde::Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(value).context("failed to serialize JSON")?;
    println!("{json}");
    Ok(())
}

/// Replace `$HOME` prefix with `~` for shorter display paths.
#[must_use]
pub fn shorten_home(path: &Path, home: &str) -> String {
    let abs = path.to_string_lossy();
    if !home.is_empty() && abs.starts_with(home) {
        format!("~{}", &abs[home.len()..])
    } else {
        abs.to_string()
    }
}

/// Return the current user's home directory as a string (empty if unavailable).
#[must_use]
pub fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_default()
}

/// Shorten a `PathBuf` display for human-readable output.
#[must_use]
pub fn short_path(path: &Path) -> String {
    shorten_home(path, &home_dir())
}

/// Write `content` to `target` atomically via a temp file + rename.
///
/// # Errors
/// Returns error if temp file creation, write, or persist fails.
pub fn atomic_write(target: &Path, content: &str) -> Result<()> {
    let dir = target.parent().unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .with_context(|| format!("failed to create temp file in {}", dir.display()))?;
    tmp.write_all(content.as_bytes())
        .context("failed to write temp file")?;
    tmp.persist(target)
        .with_context(|| format!("failed to persist temp file to {}", target.display()))?;
    Ok(())
}
