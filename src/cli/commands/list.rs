use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use crate::cli::format::{format_task_typst, format_tasks_csv};
use crate::cli::util::{open_query_db, print_json, resolve_query_db_path, shorten_home};
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
    /// Fields: due, start, rank, id, file, position (pos), title, status. Direction defaults to asc.
    #[arg(long, value_parser = parse_sort_spec)]
    pub sort: Vec<SortSpec>,

    /// Filter expression (evalexpr syntax, e.g. `has(due) || miss(start)`)
    #[arg(long, value_name = "EXPR")]
    pub filter: Option<String>,

    /// Extra heading levels to add to all Typst output headings (for embedding)
    #[arg(long, default_value_t = 0)]
    pub heading_offset: usize,

    /// Include path headings (watch root and intermediate directories) above file headings
    #[arg(long)]
    pub path_headings: bool,

    /// Include in-file milestone headings above tasks
    #[arg(long)]
    pub milestone_headings: bool,

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
        let format = self.query.output_format();
        let db_path = resolve_query_db_path(self.query.db.as_deref());
        let store = open_query_db(&db_path)?;

        let filter = TaskFilter {
            done: self.done_filter(),
            folder: self.folder.clone(),
            watch_root: self.watch_root.clone(),
            limit: self.limit,
            sort: self.sort.clone(),
            expr: self.filter.clone(),
        };

        let tasks = store
            .query_tasks(&filter)
            .context("failed to query tasks")?;

        match format {
            OutputFormat::Json => print_json(&tasks)?,
            OutputFormat::Csv => print!("{}", format_tasks_csv(&tasks)?),
            OutputFormat::Table | OutputFormat::Typst => {
                if tasks.is_empty() {
                    eprintln!("no tasks found");
                    return Ok(());
                }
                print!(
                    "{}",
                    format_typst_list(
                        &tasks,
                        self.heading_offset,
                        self.path_headings,
                        self.milestone_headings,
                    )
                );
            }
        }
        Ok(())
    }
}

/// Emit milestone headings that haven't been emitted yet.
fn emit_milestones(
    out: &mut String,
    headings: &[&str],
    cur_headings: &mut Vec<String>,
    base_depth: usize,
) {
    let matched = cur_headings
        .iter()
        .zip(headings.iter())
        .take_while(|(a, b)| a.as_str() == **b)
        .count();
    cur_headings.truncate(matched);
    for (i, heading) in headings.iter().enumerate().skip(matched) {
        let marks = "=".repeat(base_depth + i);
        out.push_str(&format!("{marks} {heading}\n"));
        cur_headings.push((*heading).to_string());
    }
}

/// Format tasks as valid Typst markup grouped by project, file, and heading.
fn format_typst_list(
    tasks: &[TaskView],
    heading_offset: usize,
    path_headings: bool,
    milestone_headings: bool,
) -> String {
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
        let new_root = cur_root != *root_name;
        if new_root {
            if path_headings {
                if has_output {
                    out.push('\n');
                }
                let root_marks = "=".repeat(2 + heading_offset);
                out.push_str(&format!("{root_marks} {root_name}\n"));
            }
            cur_root.clone_from(root_name);
            cur_dirs.clear();
            cur_file = PathBuf::new();
            cur_headings.clear();
            has_output = true;
        }

        // New file within the same root.
        if cur_file != task.file_path {
            if has_output && !(new_root && path_headings) {
                out.push('\n');
            }
            emit_dir_headings(
                &mut out,
                &segments.dirs,
                &mut cur_dirs,
                heading_offset,
                path_headings,
            );
            let depth = if path_headings {
                3 + heading_offset + segments.dirs.len()
            } else {
                2 + heading_offset
            };
            emit_file_heading(&mut out, &task.file_path, &home, depth, path_headings);
            cur_file.clone_from(&task.file_path);
            cur_headings.clear();
            has_output = true;
        }

        if milestone_headings {
            let base = if path_headings {
                4 + heading_offset + segments.dirs.len()
            } else {
                3 + heading_offset
            };
            emit_milestones(&mut out, &headings, &mut cur_headings, base);
        }

        // Emit the task.
        out.push_str(&format_task_typst(task));
        out.push('\n');
    }

    out
}

/// Emit the file heading line.
fn emit_file_heading(out: &mut String, path: &Path, home: &str, depth: usize, show_name: bool) {
    let marks = "=".repeat(depth);
    let short = shorten_home(path, home);
    if show_name {
        let name = path
            .file_name()
            .unwrap_or(OsStr::new("?"))
            .to_string_lossy();
        out.push_str(&format!("{marks} `{name}` {short}\n"));
    } else {
        out.push_str(&format!("{marks} {short}\n"));
    }
}

/// Emit only new intermediate directory headings when path headings are enabled.
fn emit_dir_headings(
    out: &mut String,
    dirs: &[String],
    cur_dirs: &mut Vec<String>,
    heading_offset: usize,
    path_headings: bool,
) {
    if !path_headings {
        return;
    }
    let dir_match = cur_dirs
        .iter()
        .zip(dirs.iter())
        .take_while(|(a, b)| a == b)
        .count();
    cur_dirs.truncate(dir_match);
    for (i, dir) in dirs.iter().enumerate().skip(dir_match) {
        let marks = "=".repeat(3 + heading_offset + i);
        out.push_str(&format!("{marks} {dir}\n"));
        cur_dirs.push(dir.clone());
    }
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
        FileSegments {
            root: root_name,
            dirs,
        }
    } else {
        let root_name = file_path
            .parent()
            .and_then(|p| p.file_name())
            .unwrap_or(OsStr::new("?"))
            .to_string_lossy()
            .to_string();
        FileSegments {
            root: root_name,
            dirs: Vec::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::store::TaskView;

    fn make_task(title: &str, file: &str, watch_root: Option<&str>) -> TaskView {
        TaskView {
            title: title.to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from(file),
            file_title: None,
            due: None,
            start: None,
            rank: None,
            tags: vec![],
            task_id: None,
            milestone: None,
            watch_root: watch_root.map(PathBuf::from),
        }
    }

    #[test]
    fn heading_offset_shifts_all_levels() {
        let tasks = vec![make_task(
            "Buy milk",
            "/projects/myapp/todo.typ",
            Some("/projects/myapp"),
        )];
        let out_0 = format_typst_list(&tasks, 0, true, true);
        let out_2 = format_typst_list(&tasks, 2, true, true);
        assert!(out_0.contains("== myapp\n"));
        assert!(out_0.contains("=== "));
        assert!(out_2.contains("==== myapp\n"));
        assert!(out_2.contains("===== "));
        assert!(!out_2.starts_with("== "));
        assert!(!out_2.contains("\n== "));
    }

    #[test]
    fn heading_offset_zero_is_default() {
        let tasks = vec![make_task("Task", "/proj/a/todo.typ", Some("/proj/a"))];
        let out = format_typst_list(&tasks, 0, true, true);
        assert!(out.starts_with("== a\n"));
    }

    #[test]
    fn heading_offset_applies_to_subdirs() {
        let tasks = vec![make_task(
            "Nested",
            "/proj/root/sub/deep/todo.typ",
            Some("/proj/root"),
        )];
        let out_0 = format_typst_list(&tasks, 0, true, true);
        let out_1 = format_typst_list(&tasks, 1, true, true);

        // offset 0: root == (2), dir "sub" === (3), dir "deep" ==== (4), file ===== (5).
        assert!(out_0.contains("=== sub\n"));
        assert!(out_0.contains("==== deep\n"));

        // offset 1: root === (3), dir "sub" ==== (4), dir "deep" ===== (5), file ====== (6).
        assert!(out_1.contains("=== root\n"));
        assert!(out_1.contains("==== sub\n"));
        assert!(out_1.contains("===== deep\n"));
    }

    #[test]
    fn heading_offset_applies_to_milestones() {
        let mut task = make_task("Fix bug", "/proj/app/todo.typ", Some("/proj/app"));
        task.milestone = Some("Sprint 1 > Backend".to_string());
        let tasks = vec![task];

        let out_0 = format_typst_list(&tasks, 0, true, true);
        let out_3 = format_typst_list(&tasks, 3, true, true);

        // offset 0: milestone base = 3 + 0 + 1 = 4 (====), nested = 5 (=====).
        assert!(out_0.contains("==== Sprint 1\n"));
        assert!(out_0.contains("===== Backend\n"));

        // offset 3: milestone base = 3 + 3 + 0 + 1 = 7 (=======), nested = 8 (========).
        assert!(out_3.contains("======= Sprint 1\n"));
        assert!(out_3.contains("======== Backend\n"));
    }

    #[test]
    fn default_skips_path_and_milestone_headings() {
        let mut task = make_task("Fix bug", "/proj/root/sub/todo.typ", Some("/proj/root"));
        task.milestone = Some("Sprint 1".to_string());
        let tasks = vec![task];

        let out = format_typst_list(&tasks, 0, false, false);

        // File heading at base level (==), path only (no backtick filename).
        assert!(out.contains("== /proj/root/sub/todo.typ\n"));
        assert!(!out.contains("`todo.typ`"));
        // No root or directory headings.
        assert!(!out.contains("== root\n"));
        assert!(!out.contains("=== sub\n"));
        // No milestone headings.
        assert!(!out.contains("Sprint 1"));
        // Task is present.
        assert!(out.contains("Fix bug"));
    }

    #[test]
    fn path_headings_only() {
        let mut task = make_task("Fix bug", "/proj/root/sub/todo.typ", Some("/proj/root"));
        task.milestone = Some("Sprint 1".to_string());
        let tasks = vec![task];

        let out = format_typst_list(&tasks, 0, true, false);

        // Root and directory headings present.
        assert!(out.contains("== root\n"));
        assert!(out.contains("=== sub\n"));
        // File heading nested under dirs, with backtick filename.
        assert!(out.contains("==== `todo.typ`"));
        // No milestone headings.
        assert!(!out.contains("Sprint 1"));
        // Task is present.
        assert!(out.contains("Fix bug"));
    }

    #[test]
    fn milestone_headings_only() {
        let mut task = make_task("Fix bug", "/proj/app/todo.typ", Some("/proj/app"));
        task.milestone = Some("Sprint 1 > Backend".to_string());
        let tasks = vec![task];

        let out = format_typst_list(&tasks, 0, false, true);

        // No root heading.
        assert!(!out.contains("== app\n"));
        // File heading at base level (==), path only.
        assert!(out.contains("== /proj/app/todo.typ\n"));
        assert!(!out.contains("`todo.typ`"));
        // Milestone headings present, nested under file.
        assert!(out.contains("=== Sprint 1\n"));
        assert!(out.contains("==== Backend\n"));
        // Task is present.
        assert!(out.contains("Fix bug"));
    }

    #[test]
    fn blank_line_between_files() {
        let tasks = vec![
            make_task("Task A", "/proj/root/a.typ", Some("/proj/root")),
            make_task("Task B", "/proj/root/b.typ", Some("/proj/root")),
        ];

        let out = format_typst_list(&tasks, 0, false, false);

        // Blank line separates the two file groups.
        assert!(out.contains("Task A\n\n== /proj/root/b.typ\n"));
    }
}
