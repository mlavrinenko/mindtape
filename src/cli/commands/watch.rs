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

    /// Config file path (repeatable, entries are merged)
    #[arg(long)]
    pub config: Vec<PathBuf>,
}

impl WatchArgs {
    /// Run the watch command.
    ///
    /// # Errors
    /// Returns error if config loading, database setup, or watcher initialization fails.
    pub fn run(&self) -> Result<()> {
        let (cfg, config_paths) = load_watch_config(self)?;

        if cfg.watch.is_empty() {
            bail!(
                "no watch paths configured\nUsage: mindtape watch <path>\n   or: mindtape watch --config <file>"
            );
        }

        let db_path = config::resolve_db_path(&cfg);
        if let Some(parent) = db_path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create database directory {}", parent.display())
            })?;
        }

        let store = SqliteStore::open(&db_path)
            .with_context(|| format!("failed to open database at {}", db_path.display()))?;

        let mut watcher =
            Watcher::new(store, &cfg.watch).context("failed to set up file watcher")?;

        let scan = watcher.initial_scan();
        info!(
            "initial scan: {} found, {} indexed, {} skipped, {} errors",
            scan.found, scan.indexed, scan.skipped, scan.errors,
        );

        watcher.run(&config_paths).context("watcher error")?;
        Ok(())
    }
}

/// Build a Config from CLI args: --config file(s), path argument, or auto-discovery.
///
/// Returns the parsed config and the canonical paths of all config files
/// used. The vec is empty when a bare directory argument was given instead
/// of config files.
fn load_watch_config(args: &WatchArgs) -> Result<(Config, Vec<PathBuf>)> {
    if !args.config.is_empty() {
        let mut canon_paths = Vec::with_capacity(args.config.len());
        for p in &args.config {
            canon_paths.push(
                std::fs::canonicalize(p)
                    .with_context(|| format!("failed to resolve config path {}", p.display()))?,
            );
        }
        let cfg =
            config::load_and_merge(&canon_paths).with_context(|| "failed to load config files")?;
        return Ok((cfg, canon_paths));
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
            vec![],
        ));
    }

    if let Some(config_path) = config::find_config() {
        let cfg = config::load_config(&config_path)
            .with_context(|| format!("failed to load config from {}", config_path.display()))?;
        let canon = std::fs::canonicalize(&config_path)
            .with_context(|| format!("failed to resolve config path {}", config_path.display()))?;
        return Ok((cfg, vec![canon]));
    }

    Ok((
        Config {
            database: None,
            watch: vec![WatchEntry {
                path: ".".to_string(),
                recursive: true,
            }],
        },
        vec![],
    ))
}
