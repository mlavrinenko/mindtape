use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::cli::util::{atomic_write, open_query_db};
use crate::store::Store;

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

impl SetArgs {
    /// Run the set command.
    ///
    /// # Errors
    /// Returns error if database open, task lookup, file read, modification, or write fails.
    pub fn run(&self) -> Result<()> {
        let store = open_query_db(self.db.as_deref())?;

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

        // Apply modifications sequentially. Each step re-parses because the source
        // text changes after each modification.
        let mut content = source.text().to_string();
        let mut changes: Vec<String> = Vec::new();

        if let Some(date_str) = &self.due {
            let src = typst_syntax::Source::detached(&content);
            content = crate::eval::set_task_due(&src, &task.task_id, date_str)
                .context("failed to set due date")?;
            changes.push(format!("due={date_str}"));
        }

        if self.no_due {
            let src = typst_syntax::Source::detached(&content);
            content = crate::eval::remove_task_due(&src, &task.task_id)
                .context("failed to remove due date")?;
            changes.push("due removed".to_string());
        }

        for tag in &self.add_tag {
            let src = typst_syntax::Source::detached(&content);
            content = crate::eval::add_task_tag(&src, &task.task_id, tag)
                .context("failed to add tag")?;
            changes.push(format!("+tag:{tag}"));
        }

        for tag in &self.remove_tag {
            let src = typst_syntax::Source::detached(&content);
            content = crate::eval::remove_task_tag(&src, &task.task_id, tag)
                .context("failed to remove tag")?;
            changes.push(format!("-tag:{tag}"));
        }

        if changes.is_empty() {
            bail!("no changes specified (use --due, --no-due, --add-tag, or --remove-tag)");
        }

        atomic_write(&task.file_path, &content)
            .with_context(|| format!("failed to write file {}", task.file_path.display()))?;

        println!("Task {} updated: {}", task.task_id, changes.join(", "));
        Ok(())
    }
}
