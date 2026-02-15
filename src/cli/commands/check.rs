use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::cli::util::{atomic_write, open_query_db, resolve_query_db_path};
use crate::store::Store;

/// Toggle a task's checkbox.
#[derive(Parser, Debug)]
pub struct CheckArgs {
    /// Task ID (or masked pattern)
    pub task_id: String,

    /// Path to `SQLite` database
    #[arg(long)]
    pub db: Option<PathBuf>,
}

impl CheckArgs {
    /// Run the check command.
    ///
    /// # Errors
    /// Returns error if database open, task lookup, file read, or write fails.
    pub fn run(&self) -> Result<()> {
        let db_path = resolve_query_db_path(self.db.as_deref());
        let store = open_query_db(&db_path)?;

        let task = store
            .find_task_by_id(&self.task_id)
            .context("failed to find task")?;

        let current_hash = crate::store::hash_file(&task.file_path)
            .with_context(|| format!("failed to read file {}", task.file_path.display()))?;

        if current_hash != task.file_hash {
            bail!(
                "file {} has changed since last index\nRun 'mindtape watch' to re-index, then try again.",
                task.file_path.display()
            );
        }

        let source = crate::eval::load_source(&task.file_path)
            .with_context(|| format!("failed to load file {}", task.file_path.display()))?;

        let new_content = crate::eval::toggle_task_checkbox(&source, &task.task_id)
            .context("failed to toggle checkbox")?;

        atomic_write(&task.file_path, &new_content)
            .with_context(|| format!("failed to write file {}", task.file_path.display()))?;

        let status = if task.is_done { "unchecked" } else { "checked" };
        println!("Task {} {}: {}", task.task_id, status, task.task_title);
        Ok(())
    }
}
