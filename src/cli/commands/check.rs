use std::path::PathBuf;

use clap::Parser;

/// Toggle a task's checkbox.
#[derive(Parser, Debug)]
pub struct CheckArgs {
    /// Task ID (or masked pattern)
    pub task_id: String,

    /// Path to `SQLite` database
    #[arg(long)]
    pub db: Option<PathBuf>,
}
