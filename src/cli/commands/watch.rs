use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use log::info;

use crate::config::{self, Config, WatchEntry};
use crate::store::SqliteStore;
use crate::watcher::Watcher;

/// Watch and index folders.
#[derive(Parser, Debug)]
pub struct WatchArgs {
    /// Directory to watch (quick mode, no config file needed)
    pub path: Option<PathBuf>,

    /// Explicit config file path
    #[arg(long)]
    pub config: Option<PathBuf>,
}

impl WatchArgs {
    /// Run the watch command.
    ///
    /// # Errors
    /// Returns error if config loading, database setup, or watcher initialization fails.
    pub fn run(&self) -> Result<()> {
        let (cfg, config_path) = load_watch_config(self)?;

        if cfg.watch.is_empty() {
            bail!("no watch paths configured\nUsage: mindtape watch <path>\n   or: mindtape watch --config <file>");
        }

        let db_path = config::resolve_db_path(&cfg);
        if let Some(parent) = db_path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create database directory {}", parent.display()))?;
        }

        let store = SqliteStore::open(&db_path)
            .with_context(|| format!("failed to open database at {}", db_path.display()))?;

        let mut watcher = Watcher::new(store, &cfg.watch)
            .context("failed to set up file watcher")?;

        let scan = watcher.initial_scan();
        info!(
            "initial scan: {} found, {} indexed, {} skipped, {} errors",
            scan.found, scan.indexed, scan.skipped, scan.errors,
        );

        watcher.run(config_path.as_deref()).context("watcher error")?;
        Ok(())
    }
}

/// Build a Config from CLI args: --config file, path argument, or auto-discovery.
///
/// Returns the parsed config and the canonical path of the config file
/// (if one was used). The path is `None` when a bare directory argument
/// was given instead of a config file.
fn load_watch_config(args: &WatchArgs) -> Result<(Config, Option<PathBuf>)> {
    if let Some(ref config_path) = args.config {
        let cfg = config::load_config(config_path)
            .with_context(|| format!("failed to load config from {}", config_path.display()))?;
        let canon = std::fs::canonicalize(config_path)
            .with_context(|| format!("failed to resolve config path {}", config_path.display()))?;
        return Ok((cfg, Some(canon)));
    }

    if let Some(ref path) = args.path {
        return Ok((
            Config {
                database: None,
                watch: vec![WatchEntry {
                    path: path.to_string_lossy().to_string(),
                    recursive: true,
                }],
            },
            None,
        ));
    }

    if let Some(config_path) = config::find_config() {
        let cfg = config::load_config(&config_path)
            .with_context(|| format!("failed to load config from {}", config_path.display()))?;
        let canon = std::fs::canonicalize(&config_path)
            .with_context(|| format!("failed to resolve config path {}", config_path.display()))?;
        return Ok((cfg, Some(canon)));
    }

    Ok((
        Config {
            database: None,
            watch: vec![WatchEntry {
                path: ".".to_string(),
                recursive: true,
            }],
        },
        None,
    ))
}
