use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};

use crate::cli::format::{format_task_typst, format_tasks_csv};
use crate::cli::util::{open_query_db, print_json, resolve_query_db_path};
use crate::cli::{OutputFormat, QueryOpts};
use crate::store::{SortDir, SortField, SortSpec, Store, TaskFilter, TaskView};

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

    /// Filter by tag (repeatable; AND logic — task must have all listed tags)
    #[arg(long)]
    pub tag: Vec<String>,

    /// Filter tasks due before this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub due_before: Option<String>,

    /// Filter tasks due on or after this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub due_after: Option<String>,

    /// Filter by milestone / heading (case-insensitive substring match)
    #[arg(long)]
    pub milestone: Option<String>,

    /// Filter by title (case-insensitive substring match)
    #[arg(long)]
    pub title: Option<String>,

    /// Full-text search across task titles and milestones
    #[arg(long)]
    pub search: Option<String>,

    /// Filter by file path
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Filter by folder prefix
    #[arg(long)]
    pub folder: Option<PathBuf>,

    /// Filter by watch root path
    #[arg(long)]
    pub watch_root: Option<PathBuf>,

    /// Filter tasks with start date before this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub start_before: Option<String>,

    /// Filter tasks with start date on or after this date (YYYY-MM-DD)
    #[arg(long, value_name = "DATE")]
    pub start_after: Option<String>,

    /// Filter tasks with rank >= this value
    #[arg(long)]
    pub rank_min: Option<i64>,

    /// Filter tasks with rank <= this value
    #[arg(long)]
    pub rank_max: Option<i64>,

    /// Filter by property presence (repeatable; AND logic — task must have all listed properties).
    /// Valid properties: due, start, rank, tag, id.
    #[arg(long, value_name = "PROPERTY")]
    pub with: Vec<String>,

    /// Limit output to N items
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    /// Sort order (repeatable, e.g. --sort=due:asc --sort=title:desc).
    /// Fields: due, start, rank, id, file, position (pos), title, status. Direction defaults to asc.
    #[arg(long, value_parser = parse_sort_spec)]
    pub sort: Vec<SortSpec>,

    #[command(flatten)]
    pub query: QueryOpts,
}

/// Parse a `field[:dir]` string into a `SortSpec`.
fn parse_sort_spec(input: &str) -> Result<SortSpec, String> {
    let (field_str, dir_str) = match input.split_once(':') {
        Some((field_part, dir_part)) => (field_part, Some(dir_part)),
        None => (input, None),
    };

    let field = match field_str {
        "due" => SortField::Due,
        "start" => SortField::Start,
        "rank" => SortField::Rank,
        "id" => SortField::Id,
        "file" => SortField::File,
        "position" | "pos" => SortField::Position,
        "title" => SortField::Title,
        "status" => SortField::Status,
        _ => return Err(format!("unknown sort field: {field_str}")),
    };

    let dir = match dir_str {
        Some("asc") | None => SortDir::Asc,
        Some("desc") => SortDir::Desc,
        Some(other) => return Err(format!("unknown sort direction: {other} (use asc or desc)")),
    };

    Ok(SortSpec { field, dir })
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
        const VALID_WITH: &[&str] = &["due", "start", "rank", "tag", "id"];
        for prop in &self.with {
            if !VALID_WITH.contains(&prop.as_str()) {
                bail!(
                    "unknown property \"{prop}\" for --with filter\n\
                     valid properties: {}",
                    VALID_WITH.join(", ")
                );
            }
        }

        let format = self.query.output_format();
        let db_path = resolve_query_db_path(self.query.db.as_deref());
        let store = open_query_db(&db_path)?;

        let filter = TaskFilter {
            done: self.done_filter(),
            tags: self.tag.clone(),
            due_before: self.due_before.clone(),
            due_after: self.due_after.clone(),
            start_before: self.start_before.clone(),
            start_after: self.start_after.clone(),
            rank_min: self.rank_min,
            rank_max: self.rank_max,
            milestone: self.milestone.clone(),
            title_contains: self.title.clone(),
            search: self.search.clone(),
            file_path: self.file.clone(),
            folder: self.folder.clone(),
            watch_root: self.watch_root.clone(),
            with: self.with.clone(),
            limit: self.limit,
            sort: self.sort.clone(),
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
                print!("{}", format_typst_list(&tasks));
            }
        }
        Ok(())
    }
}

/// Format tasks as valid Typst markup grouped by project, file, and heading.
fn format_typst_list(tasks: &[TaskView]) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut out = String::new();
    let mut has_output = false;

    // Track current context to avoid re-emitting headings.
    let mut cur_root = String::new();
    let mut cur_dirs: Vec<String> = Vec::new();
    let mut cur_file = PathBuf::new();
    let mut cur_headings: Vec<String> = Vec::new();

    for task in tasks {
        let segments = file_segments(&task.file_path, task.watch_root.as_deref());
        let root_name = &segments.root;
        let headings: Vec<&str> = task
            .milestone
            .as_deref()
            .map(|m| m.split(" > ").collect())
            .unwrap_or_default();

        // New root project — emit heading with blank line separator.
        if cur_root != *root_name {
            if has_output {
                out.push('\n');
            }
            out.push_str(&format!("== {root_name}\n"));
            cur_root.clone_from(root_name);
            cur_dirs.clear();
            cur_file = PathBuf::new();
            cur_headings.clear();
            has_output = true;
        }

        // New file within the same root.
        if cur_file != task.file_path {
            // Emit only new intermediate directory headings.
            let dir_match = cur_dirs
                .iter()
                .zip(segments.dirs.iter())
                .take_while(|(a, b)| a == b)
                .count();
            cur_dirs.truncate(dir_match);
            for (i, dir) in segments.dirs.iter().enumerate().skip(dir_match) {
                let heading_marks = "=".repeat(3 + i);
                out.push_str(&format!("{heading_marks} {dir}\n"));
                cur_dirs.push(dir.clone());
            }

            // Emit file heading.
            let file_depth = 3 + segments.dirs.len();
            let file_marks = "=".repeat(file_depth);
            let short_path = shorten_home(&task.file_path, &home);
            let file_name = task
                .file_path
                .file_name()
                .unwrap_or(OsStr::new("?"))
                .to_string_lossy();
            out.push_str(&format!("{file_marks} `{file_name}` {short_path}\n"));
            cur_file.clone_from(&task.file_path);
            cur_headings.clear();
        }

        // Emit milestone headings that haven't been emitted yet.
        let milestone_base = 3 + segments.dirs.len() + 1;
        let heading_match = cur_headings
            .iter()
            .zip(headings.iter())
            .take_while(|(a, b)| a.as_str() == **b)
            .count();
        cur_headings.truncate(heading_match);
        for (i, heading) in headings.iter().enumerate().skip(heading_match) {
            let heading_marks = "=".repeat(milestone_base + i);
            out.push_str(&format!("{heading_marks} {heading}\n"));
            cur_headings.push((*heading).to_string());
        }

        // Emit the task.
        out.push_str(&format_task_typst(task));
        out.push('\n');
    }

    out
}

struct FileSegments {
    root: String,
    dirs: Vec<String>,
}

/// Compute the display segments for a file: root name and intermediate directories.
fn file_segments(file_path: &Path, watch_root: Option<&Path>) -> FileSegments {
    if let Some(root) = watch_root
        && let Ok(rel) = file_path.strip_prefix(root)
    {
        let root_name = root
            .file_name()
            .unwrap_or(OsStr::new("?"))
            .to_string_lossy()
            .to_string();
        let dirs: Vec<String> = rel
            .parent()
            .map(|p| {
                p.components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        FileSegments { root: root_name, dirs }
    } else {
        let root_name = file_path
            .parent()
            .and_then(|p| p.file_name())
            .unwrap_or(OsStr::new("?"))
            .to_string_lossy()
            .to_string();
        FileSegments { root: root_name, dirs: Vec::new() }
    }
}

/// Replace `$HOME` prefix with `~` in a path for shorter display.
fn shorten_home(path: &Path, home: &str) -> String {
    let abs = path.to_string_lossy();
    if !home.is_empty() && abs.starts_with(home) {
        format!("~{}", &abs[home.len()..])
    } else {
        abs.to_string()
    }
}
