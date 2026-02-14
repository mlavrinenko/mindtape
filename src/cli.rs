use std::path::PathBuf;

use typst::foundations::Datetime;

use crate::eval::Task;
use crate::store::{FileView, IndexStats, TaskView};

/// Parsed CLI command.
pub enum Command {
    Eval(EvalArgs),
    Watch(WatchArgs),
    List(ListArgs),
    Status(QueryArgs),
    Files(QueryArgs),
}

pub struct EvalArgs {
    pub file: PathBuf,
    pub due: bool,
    pub limit: Option<usize>,
}

pub struct WatchArgs {
    /// Directory to watch (quick mode, no config file needed).
    pub path: Option<PathBuf>,
    /// Explicit config file path.
    pub config: Option<PathBuf>,
}

#[derive(Default)]
pub struct ListArgs {
    /// None = pending only (default), Some(true) = done, Some(false) = pending.
    /// Use `status_all` for showing everything.
    pub done: Option<bool>,
    pub status_all: bool,
    pub tag: Option<String>,
    pub due_before: Option<String>,
    pub file: Option<PathBuf>,
    pub folder: Option<PathBuf>,
    pub limit: Option<usize>,
    pub db: Option<PathBuf>,
    pub json: bool,
}

pub struct QueryArgs {
    pub db: Option<PathBuf>,
    pub json: bool,
}

const USAGE: &str = "\
Usage: mindtape <file.typ> [--due] [-N]
       mindtape list [--status done|pending|all] [--tag TAG] [--due-before DATE] [--file PATH] [--folder PREFIX] [-N] [--db PATH]
       mindtape status [--db PATH]
       mindtape files [--db PATH]
       mindtape watch [<path>] [--config <file>]";

/// # Errors
/// Returns `Err` with a usage message if the arguments are invalid or incomplete.
pub fn parse_args(args: &[String]) -> Result<Command, String> {
    let rest = args.get(1..).unwrap_or(&[]);
    match args.first().map(String::as_str) {
        Some("watch") => parse_watch_args(rest),
        Some("list") => parse_list_args(rest),
        Some("status") => parse_query_args(rest).map(Command::Status),
        Some("files") => parse_query_args(rest).map(Command::Files),
        _ => parse_eval_args(args).map(Command::Eval),
    }
}

fn parse_eval_args(args: &[String]) -> Result<EvalArgs, String> {
    let mut file: Option<PathBuf> = None;
    let mut due = false;
    let mut limit: Option<usize> = None;

    for arg in args {
        if arg == "--due" {
            due = true;
        } else if let Some(n) = arg.strip_prefix('-').and_then(|s| s.parse::<usize>().ok()) {
            limit = Some(n);
        } else {
            file = Some(PathBuf::from(arg));
        }
    }

    let file = file.ok_or_else(|| USAGE.to_string())?;

    Ok(EvalArgs { file, due, limit })
}

#[allow(clippy::indexing_slicing)]
fn parse_watch_args(args: &[String]) -> Result<Command, String> {
    let mut path: Option<PathBuf> = None;
    let mut config: Option<PathBuf> = None;
    let mut idx = 0;

    while idx < args.len() {
        if args[idx] == "--config" {
            idx += 1;
            config = Some(
                args.get(idx)
                    .map(PathBuf::from)
                    .ok_or_else(|| "--config requires a path argument".to_string())?,
            );
        } else if args[idx].starts_with('-') {
            return Err(format!("unknown watch flag: {}", args[idx]));
        } else {
            path = Some(PathBuf::from(&args[idx]));
        }
        idx += 1;
    }

    Ok(Command::Watch(WatchArgs { path, config }))
}

#[allow(clippy::indexing_slicing)]
fn parse_list_args(args: &[String]) -> Result<Command, String> {
    let mut list = ListArgs::default();
    let mut idx = 0;

    while idx < args.len() {
        match args[idx].as_str() {
            "--status" => {
                idx += 1;
                let val = args.get(idx).ok_or("--status requires a value (done, pending, all)")?;
                match val.as_str() {
                    "done" => list.done = Some(true),
                    "pending" => list.done = Some(false),
                    "all" => list.status_all = true,
                    _ => return Err(format!("invalid --status value: {val} (use done, pending, or all)")),
                }
            }
            "--tag" => {
                idx += 1;
                list.tag = Some(
                    args.get(idx)
                        .ok_or("--tag requires a value")?
                        .clone(),
                );
            }
            "--due-before" => {
                idx += 1;
                list.due_before = Some(
                    args.get(idx)
                        .ok_or("--due-before requires a date (YYYY-MM-DD)")?
                        .clone(),
                );
            }
            "--file" => {
                idx += 1;
                list.file = Some(PathBuf::from(
                    args.get(idx).ok_or("--file requires a path")?,
                ));
            }
            "--folder" => {
                idx += 1;
                list.folder = Some(PathBuf::from(
                    args.get(idx).ok_or("--folder requires a path")?,
                ));
            }
            "--db" => {
                idx += 1;
                list.db = Some(PathBuf::from(
                    args.get(idx).ok_or("--db requires a path")?,
                ));
            }
            "--json" => {
                list.json = true;
            }
            other => {
                if let Some(n) = other.strip_prefix('-').and_then(|s| s.parse::<usize>().ok()) {
                    list.limit = Some(n);
                } else {
                    return Err(format!("unknown list argument: {other}"));
                }
            }
        }
        idx += 1;
    }

    Ok(Command::List(list))
}

#[allow(clippy::indexing_slicing)]
fn parse_query_args(args: &[String]) -> Result<QueryArgs, String> {
    let mut db: Option<PathBuf> = None;
    let mut json = false;
    let mut idx = 0;

    while idx < args.len() {
        if args[idx] == "--db" {
            idx += 1;
            db = Some(PathBuf::from(
                args.get(idx).ok_or("--db requires a path")?,
            ));
        } else if args[idx] == "--json" {
            json = true;
        } else {
            return Err(format!("unknown argument: {}", args[idx]));
        }
        idx += 1;
    }

    Ok(QueryArgs { db, json })
}

// ---------------------------------------------------------------------------
// Formatting: store query results
// ---------------------------------------------------------------------------

#[must_use]
pub fn format_task_view(task: &TaskView) -> String {
    let check = if task.is_done { "[x]" } else { "[ ]" };
    let due_part = task
        .due
        .as_ref()
        .map(|d| format!(" (due {d})"))
        .unwrap_or_default();
    let tag_part = if task.tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", task.tags.join(", "))
    };
    format!("- {check}{due_part} {}{tag_part}", task.title)
}

#[must_use]
pub fn format_file_view(file: &FileView) -> String {
    let title_part = file
        .title
        .as_ref()
        .map(|t| format!("  {t}"))
        .unwrap_or_default();
    let count = file.task_count;
    let noun = if count == 1 { "task" } else { "tasks" };
    format!("{} ({count} {noun}){title_part}", file.relative_path.display())
}

#[must_use]
pub fn format_stats(stats: &IndexStats) -> String {
    let mut lines = vec![
        format!("files:   {}", stats.file_count),
        format!("tasks:   {} ({} pending, {} done)", stats.task_count, stats.pending_count, stats.done_count),
    ];
    if let Some(ref ts) = stats.last_updated {
        lines.push(format!("updated: {ts}"));
    }
    lines.join("\n")
}

/// # Panics
/// Panics if `due_only` is true and a retained task has a `None` due date.
#[must_use]
pub fn filter_and_sort(tasks: Vec<Task>, due_only: bool, limit: Option<usize>) -> Vec<Task> {
    // Filter out completed tasks
    let mut tasks: Vec<_> = tasks.into_iter().filter(|t| !t.done).collect();

    // If due_only: keep only tasks with due dates, sort by due date ascending
    if due_only {
        tasks.retain(|t| t.due.is_some());
        #[allow(clippy::unwrap_used)]
        tasks.sort_by(|a, b| {
            let a_due = a.due.as_ref().unwrap();
            let b_due = b.due.as_ref().unwrap();
            due_sort_key(a_due).cmp(&due_sort_key(b_due))
        });
    }

    // Apply limit
    if let Some(n) = limit {
        tasks.truncate(n);
    }

    tasks
}

#[must_use]
pub fn format_task(task: &Task) -> String {
    match &task.due {
        Some(dt) => format!("- (due {}) {}", format_due(dt), task.title),
        None => format!("- {}", task.title),
    }
}

#[must_use]
pub fn format_due(dt: &Datetime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        dt.year().unwrap_or(0),
        dt.month().unwrap_or(0),
        dt.day().unwrap_or(0),
    )
}

#[must_use]
pub fn due_sort_key(dt: &Datetime) -> (i32, u8, u8) {
    (
        dt.year().unwrap_or(0),
        dt.month().unwrap_or(0),
        dt.day().unwrap_or(0),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn s(val: &str) -> String {
        val.to_string()
    }

    fn make_task(title: &str, done: bool, due: Option<Datetime>) -> Task {
        Task { title: title.to_string(), done, due, tags: vec![], position: 0 }
    }

    fn ymd(year: i32, month: u8, day: u8) -> Datetime {
        Datetime::from_ymd(year, month, day).unwrap()
    }

    // --- parse_args: eval (backwards compat) ---

    #[test]
    fn parse_args_file_only() {
        let cmd = parse_args(&[s("foo.typ")]).unwrap();
        let Command::Eval(args) = cmd else { panic!("expected Eval") };
        assert_eq!(args.file, PathBuf::from("foo.typ"));
        assert!(!args.due);
        assert_eq!(args.limit, None);
    }

    #[test]
    fn parse_args_with_due() {
        let cmd = parse_args(&[s("f.typ"), s("--due")]).unwrap();
        let Command::Eval(args) = cmd else { panic!("expected Eval") };
        assert!(args.due);
    }

    #[test]
    fn parse_args_with_limit() {
        let cmd = parse_args(&[s("f.typ"), s("-3")]).unwrap();
        let Command::Eval(args) = cmd else { panic!("expected Eval") };
        assert_eq!(args.limit, Some(3));
    }

    #[test]
    fn parse_args_all_flags() {
        let cmd = parse_args(&[s("f.typ"), s("--due"), s("-5")]).unwrap();
        let Command::Eval(args) = cmd else { panic!("expected Eval") };
        assert_eq!(args.file, PathBuf::from("f.typ"));
        assert!(args.due);
        assert_eq!(args.limit, Some(5));
    }

    #[test]
    fn parse_args_no_file() {
        assert!(parse_args(&[s("--due")]).is_err());
    }

    #[test]
    fn parse_args_empty() {
        assert!(parse_args(&[]).is_err());
    }

    // --- parse_args: watch ---

    #[test]
    fn parse_watch_bare() {
        let cmd = parse_args(&[s("watch")]).unwrap();
        let Command::Watch(args) = cmd else { panic!("expected Watch") };
        assert_eq!(args.path, None);
        assert_eq!(args.config, None);
    }

    #[test]
    fn parse_watch_with_path() {
        let cmd = parse_args(&[s("watch"), s(".")]).unwrap();
        let Command::Watch(args) = cmd else { panic!("expected Watch") };
        assert_eq!(args.path, Some(PathBuf::from(".")));
    }

    #[test]
    fn parse_watch_with_config() {
        let cmd = parse_args(&[s("watch"), s("--config"), s("my.toml")]).unwrap();
        let Command::Watch(args) = cmd else { panic!("expected Watch") };
        assert_eq!(args.config, Some(PathBuf::from("my.toml")));
        assert_eq!(args.path, None);
    }

    #[test]
    fn parse_watch_config_missing_value() {
        let result = parse_args(&[s("watch"), s("--config")]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_watch_unknown_flag() {
        let result = parse_args(&[s("watch"), s("--verbose")]);
        assert!(result.is_err());
    }

    // --- format_due ---

    #[test]
    fn format_due_normal() {
        assert_eq!(format_due(&ymd(2026, 3, 15)), "2026-03-15");
    }

    #[test]
    fn format_due_zero_padded() {
        assert_eq!(format_due(&ymd(2026, 1, 5)), "2026-01-05");
    }

    // --- due_sort_key ---

    #[test]
    fn due_sort_key_extracts_components() {
        assert_eq!(due_sort_key(&ymd(2026, 3, 1)), (2026, 3, 1));
    }

    // --- format_task ---

    #[test]
    fn format_task_with_due() {
        let task = make_task("Buy milk", false, Some(ymd(2026, 3, 1)));
        assert_eq!(format_task(&task), "- (due 2026-03-01) Buy milk");
    }

    #[test]
    fn format_task_without_due() {
        let task = make_task("Buy milk", false, None);
        assert_eq!(format_task(&task), "- Buy milk");
    }

    // --- filter_and_sort ---

    #[test]
    fn filter_and_sort_removes_done() {
        let tasks = vec![
            make_task("a", false, None),
            make_task("b", true, None),
            make_task("c", false, None),
        ];
        let result = filter_and_sort(tasks, false, None);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "a");
        assert_eq!(result[1].title, "c");
    }

    #[test]
    fn filter_and_sort_due_only() {
        let tasks = vec![
            make_task("a", false, Some(ymd(2026, 3, 1))),
            make_task("b", false, None),
            make_task("c", false, Some(ymd(2026, 1, 1))),
        ];
        let result = filter_and_sort(tasks, true, None);
        assert_eq!(result.len(), 2);
        // Should also sort by due date ascending
        assert_eq!(result[0].title, "c");
        assert_eq!(result[1].title, "a");
    }

    #[test]
    fn filter_and_sort_sorts_by_due() {
        let tasks = vec![
            make_task("later", false, Some(ymd(2026, 6, 1))),
            make_task("sooner", false, Some(ymd(2026, 2, 1))),
            make_task("middle", false, Some(ymd(2026, 4, 1))),
        ];
        let result = filter_and_sort(tasks, true, None);
        assert_eq!(result[0].title, "sooner");
        assert_eq!(result[1].title, "middle");
        assert_eq!(result[2].title, "later");
    }

    #[test]
    fn filter_and_sort_with_limit() {
        let tasks = vec![
            make_task("a", false, None),
            make_task("b", false, None),
            make_task("c", false, None),
        ];
        let result = filter_and_sort(tasks, false, Some(1));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "a");
    }

    #[test]
    fn filter_and_sort_limit_none_returns_all() {
        let tasks = vec![
            make_task("a", false, None),
            make_task("b", false, None),
        ];
        let result = filter_and_sort(tasks, false, None);
        assert_eq!(result.len(), 2);
    }

    // --- parse_args: list ---

    #[test]
    fn parse_list_bare() {
        let cmd = parse_args(&[s("list")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.done, None);
        assert!(!args.status_all);
        assert_eq!(args.tag, None);
        assert_eq!(args.limit, None);
        assert_eq!(args.db, None);
    }

    #[test]
    fn parse_list_status_done() {
        let cmd = parse_args(&[s("list"), s("--status"), s("done")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.done, Some(true));
    }

    #[test]
    fn parse_list_status_pending() {
        let cmd = parse_args(&[s("list"), s("--status"), s("pending")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.done, Some(false));
    }

    #[test]
    fn parse_list_status_all() {
        let cmd = parse_args(&[s("list"), s("--status"), s("all")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert!(args.status_all);
    }

    #[test]
    fn parse_list_status_invalid() {
        let result = parse_args(&[s("list"), s("--status"), s("xyz")]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_list_with_tag() {
        let cmd = parse_args(&[s("list"), s("--tag"), s("work")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.tag, Some("work".to_string()));
    }

    #[test]
    fn parse_list_with_due_before() {
        let cmd = parse_args(&[s("list"), s("--due-before"), s("2026-03-01")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.due_before, Some("2026-03-01".to_string()));
    }

    #[test]
    fn parse_list_with_file() {
        let cmd = parse_args(&[s("list"), s("--file"), s("todo.typ")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.file, Some(PathBuf::from("todo.typ")));
    }

    #[test]
    fn parse_list_with_folder() {
        let cmd = parse_args(&[s("list"), s("--folder"), s("notes/")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.folder, Some(PathBuf::from("notes/")));
    }

    #[test]
    fn parse_list_with_limit() {
        let cmd = parse_args(&[s("list"), s("-5")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.limit, Some(5));
    }

    #[test]
    fn parse_list_with_db() {
        let cmd = parse_args(&[s("list"), s("--db"), s("/tmp/test.db")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.db, Some(PathBuf::from("/tmp/test.db")));
    }

    #[test]
    fn parse_list_all_flags() {
        let cmd = parse_args(&[
            s("list"), s("--status"), s("done"), s("--tag"), s("work"),
            s("--due-before"), s("2026-03-01"), s("--file"), s("t.typ"),
            s("-3"), s("--db"), s("x.db"),
        ]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.done, Some(true));
        assert_eq!(args.tag, Some("work".to_string()));
        assert_eq!(args.due_before, Some("2026-03-01".to_string()));
        assert_eq!(args.file, Some(PathBuf::from("t.typ")));
        assert_eq!(args.limit, Some(3));
        assert_eq!(args.db, Some(PathBuf::from("x.db")));
    }

    #[test]
    fn parse_list_unknown_flag() {
        let result = parse_args(&[s("list"), s("--verbose")]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_list_missing_tag_value() {
        let result = parse_args(&[s("list"), s("--tag")]);
        assert!(result.is_err());
    }

    // --- parse_args: status / files ---

    #[test]
    fn parse_status_bare() {
        let cmd = parse_args(&[s("status")]).unwrap();
        let Command::Status(args) = cmd else { panic!("expected Status") };
        assert_eq!(args.db, None);
    }

    #[test]
    fn parse_status_with_db() {
        let cmd = parse_args(&[s("status"), s("--db"), s("x.db")]).unwrap();
        let Command::Status(args) = cmd else { panic!("expected Status") };
        assert_eq!(args.db, Some(PathBuf::from("x.db")));
    }

    #[test]
    fn parse_files_bare() {
        let cmd = parse_args(&[s("files")]).unwrap();
        let Command::Files(args) = cmd else { panic!("expected Files") };
        assert_eq!(args.db, None);
    }

    #[test]
    fn parse_files_with_db() {
        let cmd = parse_args(&[s("files"), s("--db"), s("x.db")]).unwrap();
        let Command::Files(args) = cmd else { panic!("expected Files") };
        assert_eq!(args.db, Some(PathBuf::from("x.db")));
    }

    #[test]
    fn parse_status_unknown_flag() {
        let result = parse_args(&[s("status"), s("--verbose")]);
        assert!(result.is_err());
    }

    // --- --json flag ---

    #[test]
    fn parse_list_with_json() {
        let cmd = parse_args(&[s("list"), s("--json")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert!(args.json);
    }

    #[test]
    fn parse_list_without_json() {
        let cmd = parse_args(&[s("list")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert!(!args.json);
    }

    #[test]
    fn parse_status_with_json() {
        let cmd = parse_args(&[s("status"), s("--json")]).unwrap();
        let Command::Status(args) = cmd else { panic!("expected Status") };
        assert!(args.json);
    }

    #[test]
    fn parse_files_with_json() {
        let cmd = parse_args(&[s("files"), s("--json")]).unwrap();
        let Command::Files(args) = cmd else { panic!("expected Files") };
        assert!(args.json);
    }

    // --- format_task_view ---

    #[test]
    fn format_task_view_pending_with_due() {
        let task = TaskView {
            title: "Buy milk".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: Some("2026-03-01".to_string()),
            tags: vec![],
        };
        assert_eq!(format_task_view(&task), "- [ ] (due 2026-03-01) Buy milk");
    }

    #[test]
    fn format_task_view_done_no_due() {
        let task = TaskView {
            title: "Done thing".to_string(),
            is_done: true,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: None,
            tags: vec![],
        };
        assert_eq!(format_task_view(&task), "- [x] Done thing");
    }

    #[test]
    fn format_task_view_with_tags() {
        let task = TaskView {
            title: "Task".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("t.typ"),
            file_title: None,
            due: None,
            tags: vec!["work".to_string(), "urgent".to_string()],
        };
        assert_eq!(format_task_view(&task), "- [ ] Task [work, urgent]");
    }

    // --- format_file_view ---

    #[test]
    fn format_file_view_with_title() {
        let file = FileView {
            relative_path: PathBuf::from("notes/todo.typ"),
            title: Some("My Tasks".to_string()),
            task_count: 5,
            updated_at: "2026-01-01".to_string(),
        };
        assert_eq!(format_file_view(&file), "notes/todo.typ (5 tasks)  My Tasks");
    }

    #[test]
    fn format_file_view_no_title_singular() {
        let file = FileView {
            relative_path: PathBuf::from("t.typ"),
            title: None,
            task_count: 1,
            updated_at: "2026-01-01".to_string(),
        };
        assert_eq!(format_file_view(&file), "t.typ (1 task)");
    }

    // --- format_stats ---

    #[test]
    fn format_stats_with_data() {
        let stats = IndexStats {
            file_count: 3,
            task_count: 10,
            done_count: 4,
            pending_count: 6,
            last_updated: Some("2026-01-15 12:00:00".to_string()),
        };
        let output = format_stats(&stats);
        assert!(output.contains("files:   3"));
        assert!(output.contains("tasks:   10 (6 pending, 4 done)"));
        assert!(output.contains("updated: 2026-01-15 12:00:00"));
    }

    #[test]
    fn format_stats_empty() {
        let stats = IndexStats {
            file_count: 0,
            task_count: 0,
            done_count: 0,
            pending_count: 0,
            last_updated: None,
        };
        let output = format_stats(&stats);
        assert!(output.contains("files:   0"));
        assert!(!output.contains("updated:"));
    }
}
