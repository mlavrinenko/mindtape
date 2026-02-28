use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use termtree::Tree;

use crate::cli::format::{format_task_leaf, format_tasks_csv};
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

    /// Limit output to N items
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    /// Sort order (repeatable, e.g. --sort=due:asc --sort=title:desc).
    /// Fields: due, id, file, position (pos), title, status. Direction defaults to asc.
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
        let format = self.query.output_format();
        let db_path = resolve_query_db_path(self.query.db.as_deref());
        let store = open_query_db(&db_path)?;

        let filter = TaskFilter {
            done: self.done_filter(),
            tags: self.tag.clone(),
            due_before: self.due_before.clone(),
            due_after: self.due_after.clone(),
            milestone: self.milestone.clone(),
            title_contains: self.title.clone(),
            search: self.search.clone(),
            file_path: self.file.clone(),
            folder: self.folder.clone(),
            watch_root: self.watch_root.clone(),
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
                for tree in build_task_trees(&tasks) {
                    print!("{tree}");
                }
            }
        }
        Ok(())
    }
}

/// Build a list of trees from tasks, merging shared directory prefixes.
///
/// Files sharing a watch-root (or root display name) are merged into
/// one tree, with shared intermediate directories deduplicated.
/// Headings and tasks are inserted under each file's label node.
#[allow(clippy::indexing_slicing)]
fn build_task_trees(tasks: &[TaskView]) -> Vec<Tree<String>> {
    let mut roots: Vec<Tree<String>> = Vec::new();

    for task in tasks {
        let segments = file_segments(&task.file_path, task.watch_root.as_deref());
        let leaf = Tree::new(format_task_leaf(task));
        let headings: Vec<&str> = task
            .milestone
            .as_deref()
            .map(|m| m.split(" > ").collect())
            .unwrap_or_default();

        // Find or create a matching root tree.
        let root_name = &segments[0];
        let root_idx = roots
            .iter()
            .position(|r| r.root == *root_name)
            .unwrap_or_else(|| {
                roots.push(Tree::new(root_name.clone()));
                roots.len() - 1
            });
        let root = &mut roots[root_idx];

        // Walk / create intermediate directory + file-label nodes.
        let file_node = ensure_path(root, &segments[1..]);
        insert_into_heading_path(file_node, &headings, leaf);
    }

    roots
}

/// Compute the display segments for a file: root name, directories, file label.
fn file_segments(file_path: &Path, watch_root: Option<&Path>) -> Vec<String> {
    let abs_str = file_path.to_string_lossy();
    let file_name = file_path
        .file_name()
        .unwrap_or(OsStr::new("?"))
        .to_string_lossy();
    let file_label = format!("`{file_name}` #link(\"{abs_str}\")");

    let (root_name, dir_segments) = if let Some(root) = watch_root
        && let Ok(rel) = file_path.strip_prefix(root)
    {
        let root_name = format!(
            "== {}",
            root.file_name()
                .unwrap_or(OsStr::new("?"))
                .to_string_lossy()
        );
        let dirs: Vec<String> = rel
            .parent()
            .map(|p| {
                p.components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        (root_name, dirs)
    } else {
        let root_name = format!(
            "== {}",
            file_path
                .parent()
                .and_then(|p| p.file_name())
                .unwrap_or(OsStr::new("?"))
                .to_string_lossy()
        );
        (root_name, Vec::new())
    };

    let mut segs = Vec::with_capacity(1 + dir_segments.len() + 1);
    segs.push(root_name);
    segs.extend(dir_segments);
    segs.push(file_label);
    segs
}

/// Walk down a tree following `segments`, reusing existing children or creating new ones.
///
/// Returns a mutable reference to the deepest node (the file-label node).
#[allow(clippy::indexing_slicing)]
fn ensure_path<'a>(tree: &'a mut Tree<String>, segments: &[String]) -> &'a mut Tree<String> {
    let mut node = tree;
    for seg in segments {
        let pos = node.leaves.iter().position(|child| child.root == *seg);
        if let Some(idx) = pos {
            node = &mut node.leaves[idx];
        } else {
            node.push(Tree::new(seg.clone()));
            node = node.leaves.last_mut().expect("just pushed");
        }
    }
    node
}

/// Insert a task leaf into the correct position in the heading tree.
///
/// Walks the heading path, creating or reusing heading nodes as needed,
/// then appends the leaf at the deepest level.
#[allow(clippy::indexing_slicing)]
fn insert_into_heading_path(
    root: &mut Tree<String>,
    headings: &[&str],
    leaf: Tree<String>,
) {
    if headings.is_empty() {
        root.push(leaf);
        return;
    }

    let mut node = root;
    for heading in headings {
        let heading_str = (*heading).to_string();
        // Find existing child with this heading name.
        let pos = node
            .leaves
            .iter()
            .position(|child| child.root == heading_str);

        if let Some(idx) = pos {
            // Safe: idx comes from position() on node.leaves
            node = &mut node.leaves[idx];
        } else {
            node.push(Tree::new(heading_str));
            // Safe: we just pushed, so last index is valid
            let last = node.leaves.len() - 1;
            node = &mut node.leaves[last];
        }
    }

    node.push(leaf);
}
