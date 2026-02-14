use std::path::PathBuf;

use clap::Parser;

/// Watch and index folders.
#[derive(Parser, Debug)]
pub struct WatchArgs {
    /// Directory to watch (quick mode, no config file needed)
    pub path: Option<PathBuf>,

    /// Explicit config file path
    #[arg(long)]
    pub config: Option<PathBuf>,
}
