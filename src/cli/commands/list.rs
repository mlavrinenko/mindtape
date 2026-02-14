use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::cli::QueryOpts;

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
}
