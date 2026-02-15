use std::path::PathBuf;

use clap::Parser;

/// Update task properties (due date, tags).
#[derive(Parser, Debug)]
pub struct SetArgs {
    /// Task ID (or masked pattern)
    pub task_id: String,

    /// Set due date (YYYY-MM-DD)
    #[arg(long)]
    pub due: Option<String>,

    /// Remove due date
    #[arg(long, conflicts_with = "due")]
    pub no_due: bool,

    /// Add a tag
    #[arg(long, value_name = "TAG")]
    pub add_tag: Vec<String>,

    /// Remove a tag
    #[arg(long, value_name = "TAG")]
    pub remove_tag: Vec<String>,

    /// Path to `SQLite` database
    #[arg(long)]
    pub db: Option<PathBuf>,
}
