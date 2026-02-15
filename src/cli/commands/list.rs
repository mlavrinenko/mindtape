use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use crate::cli::format::{format_task_view, format_tasks_csv};
use crate::cli::util::{open_query_db, print_json};
use crate::cli::{OutputFormat, QueryOpts};
use crate::store::{Store, TaskFilter};

/// Filter for task status.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum StatusFilter {
    Done,
    Pending,
    All,
}

/// List tasks from the index.
#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Filter by status (default: pending)
    #[arg(long, value_enum)]
    pub status: Option<StatusFilter>,

    /// Filter by tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Filter tasks due before this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub due_before: Option<String>,

    /// Filter by file path
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Filter by folder prefix
    #[arg(long)]
    pub folder: Option<PathBuf>,

    /// Limit output to N items
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    #[command(flatten)]
    pub query: QueryOpts,
}

impl ListArgs {
    /// Resolve the `done` filter for the store layer.
    ///
    /// `None` means show all, `Some(true)` = done only, `Some(false)` = pending only.
    #[must_use]
    pub fn done_filter(&self) -> Option<bool> {
        match self.status {
            Some(StatusFilter::Done) => Some(true),
            Some(StatusFilter::Pending) | None => Some(false),
            Some(StatusFilter::All) => None,
        }
    }

    /// Run the list command.
    ///
    /// # Errors
    /// Returns error if database open or query fails, or if JSON/CSV formatting fails.
    pub fn run(&self) -> Result<()> {
        let format = self.query.output_format();
        let store = open_query_db(self.query.db.as_deref())?;

        let filter = TaskFilter {
            done: self.done_filter(),
            tag: self.tag.clone(),
            due_before: self.due_before.clone(),
            file_path: self.file.clone(),
            folder: self.folder.clone(),
            limit: self.limit,
        };

        let tasks = store
            .query_tasks(&filter)
            .context("failed to query tasks")?;

        match format {
            OutputFormat::Json => print_json(&tasks)?,
            OutputFormat::Csv => print!("{}", format_tasks_csv(&tasks)?),
            OutputFormat::Table => {
                if tasks.is_empty() {
                    eprintln!("no tasks found");
                    return Ok(());
                }
                let mut current_file = String::new();
                for task in &tasks {
                    let file_str = task.file_path.to_string_lossy();
                    if file_str != current_file {
                        if !current_file.is_empty() {
                            println!();
                        }
                        let header = task.file_title.as_deref().unwrap_or(&file_str);
                        println!("{header} ({file_str})");
                        current_file = file_str.to_string();
                    }
                    println!("  {}", format_task_view(task));
                }
            }
        }
        Ok(())
    }
}
