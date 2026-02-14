use std::path::PathBuf;

use typst::foundations::Datetime;

use crate::eval::Task;
use crate::store::{AgendaView, FileView, IndexStats, SearchResults, TaskView};

/// Output format for query commands.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
}

fn parse_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "table" => Ok(OutputFormat::Table),
        "json" => Ok(OutputFormat::Json),
        "csv" => Ok(OutputFormat::Csv),
        _ => Err(format!("invalid --format value: {value} (use table, json, or csv)")),
    }
}

/// Parsed CLI command.
pub enum Command {
    Eval(EvalArgs),
    Watch(WatchArgs),
    List(ListArgs),
    Status(QueryArgs),
    Files(QueryArgs),
    Search(SearchArgs),
    Agenda(AgendaArgs),
}

pub struct SearchArgs {
    pub query: String,
    pub limit: Option<usize>,
    pub db: Option<PathBuf>,
    pub format: OutputFormat,
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
    pub format: OutputFormat,
}

pub struct QueryArgs {
    pub db: Option<PathBuf>,
    pub format: OutputFormat,
}

#[derive(Default)]
pub struct AgendaArgs {
    pub show_overdue: bool,
    pub show_today: bool,
    pub show_week: bool,
    pub limit: Option<usize>,
    pub db: Option<PathBuf>,
    pub format: OutputFormat,
}

const USAGE: &str = "\
Usage: mindtape <file.typ> [--due] [-N]
       mindtape list [--status done|pending|all] [--tag TAG] [--due-before DATE] [--file PATH] [--folder PREFIX] [-N] [--db PATH] [--format table|json|csv]
       mindtape search <keyword> [-N] [--db PATH] [--format table|json|csv]
       mindtape agenda [--overdue] [--today] [--week] [-N] [--db PATH] [--format table|json|csv]
       mindtape status [--db PATH] [--format table|json|csv]
       mindtape files [--db PATH] [--format table|json|csv]
       mindtape watch [<path>] [--config <file>]";

/// # Errors
/// Returns `Err` with a usage message if the arguments are invalid or incomplete.
pub fn parse_args(args: &[String]) -> Result<Command, String> {
    let rest = args.get(1..).unwrap_or(&[]);
    match args.first().map(String::as_str) {
        Some("watch") => parse_watch_args(rest),
        Some("list") => parse_list_args(rest),
        Some("search") => parse_search_args(rest),
        Some("agenda") => parse_agenda_args(rest),
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
                list.format = OutputFormat::Json;
            }
            "--format" => {
                idx += 1;
                let val = args.get(idx).ok_or("--format requires a value (table, json, or csv)")?;
                list.format = parse_format(val)?;
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
    let mut format = OutputFormat::default();
    let mut idx = 0;

    while idx < args.len() {
        if args[idx] == "--db" {
            idx += 1;
            db = Some(PathBuf::from(
                args.get(idx).ok_or("--db requires a path")?,
            ));
        } else if args[idx] == "--json" {
            format = OutputFormat::Json;
        } else if args[idx] == "--format" {
            idx += 1;
            let val = args.get(idx).ok_or("--format requires a value (table, json, or csv)")?;
            format = parse_format(val)?;
        } else {
            return Err(format!("unknown argument: {}", args[idx]));
        }
        idx += 1;
    }

    Ok(QueryArgs { db, format })
}

#[allow(clippy::indexing_slicing)]
fn parse_search_args(args: &[String]) -> Result<Command, String> {
    let mut query: Option<String> = None;
    let mut limit = None;
    let mut db = None;
    let mut format = OutputFormat::default();
    let mut idx = 0;

    while idx < args.len() {
        match args[idx].as_str() {
            "--db" => {
                idx += 1;
                db = Some(PathBuf::from(
                    args.get(idx).ok_or("--db requires a path")?,
                ));
            }
            "--json" => {
                format = OutputFormat::Json;
            }
            "--format" => {
                idx += 1;
                let val = args.get(idx).ok_or("--format requires a value (table, json, or csv)")?;
                format = parse_format(val)?;
            }
            other => {
                if let Some(num) = other.strip_prefix('-').and_then(|s| s.parse::<usize>().ok()) {
                    limit = Some(num);
                } else if query.is_none() {
                    query = Some(other.to_string());
                } else {
                    return Err(format!("unknown search argument: {other}"));
                }
            }
        }
        idx += 1;
    }

    let query = query.ok_or_else(|| "search requires a keyword argument".to_string())?;

    Ok(Command::Search(SearchArgs { query, limit, db, format }))
}

fn parse_agenda_args(args: &[String]) -> Result<Command, String> {
    let mut agenda = AgendaArgs::default();
    let mut idx = 0;

    while idx < args.len() {
        let Some(arg) = args.get(idx) else {
            break;
        };
        match arg.as_str() {
            "--overdue" => {
                agenda.show_overdue = true;
            }
            "--today" => {
                agenda.show_today = true;
            }
            "--week" => {
                agenda.show_week = true;
            }
            "--db" => {
                idx += 1;
                agenda.db = Some(PathBuf::from(
                    args.get(idx).ok_or("--db requires a path")?,
                ));
            }
            "--json" => {
                agenda.format = OutputFormat::Json;
            }
            "--format" => {
                idx += 1;
                let val = args.get(idx).ok_or("--format requires a value (table, json, or csv)")?;
                agenda.format = parse_format(val)?;
            }
            other => {
                if let Some(num) = other.strip_prefix('-').and_then(|s| s.parse::<usize>().ok()) {
                    agenda.limit = Some(num);
                } else {
                    return Err(format!("unknown agenda argument: {other}"));
                }
            }
        }
        idx += 1;
    }

    // If no specific sections requested, show all sections
    if !agenda.show_overdue && !agenda.show_today && !agenda.show_week {
        agenda.show_overdue = true;
        agenda.show_today = true;
        agenda.show_week = true;
    }

    Ok(Command::Agenda(agenda))
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

// ---------------------------------------------------------------------------
// Formatting: CSV output
// ---------------------------------------------------------------------------

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[must_use]
pub fn format_tasks_csv(tasks: &[TaskView]) -> String {
    let mut out = String::from("status,due,title,file,tags\n");
    for t in tasks {
        let status = if t.is_done { "done" } else { "pending" };
        let due = t.due.as_deref().unwrap_or("");
        let tags = t.tags.join(";");
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            status,
            due,
            csv_escape(&t.title),
            csv_escape(&t.file_path.to_string_lossy()),
            csv_escape(&tags),
        ));
    }
    out
}

#[must_use]
pub fn format_files_csv(files: &[FileView]) -> String {
    let mut out = String::from("path,title,tasks,updated_at\n");
    for f in files {
        let title = f.title.as_deref().unwrap_or("");
        out.push_str(&format!(
            "{},{},{},{}\n",
            csv_escape(&f.relative_path.to_string_lossy()),
            csv_escape(title),
            f.task_count,
            csv_escape(&f.updated_at),
        ));
    }
    out
}

#[must_use]
pub fn format_stats_csv(stats: &IndexStats) -> String {
    let mut out = String::from("metric,value\n");
    out.push_str(&format!("files,{}\n", stats.file_count));
    out.push_str(&format!("tasks,{}\n", stats.task_count));
    out.push_str(&format!("done,{}\n", stats.done_count));
    out.push_str(&format!("pending,{}\n", stats.pending_count));
    if let Some(ref ts) = stats.last_updated {
        out.push_str(&format!("last_updated,{}\n", csv_escape(ts)));
    }
    out
}

// ---------------------------------------------------------------------------
// Formatting: search results
// ---------------------------------------------------------------------------

#[must_use]
pub fn format_search_results(results: &SearchResults) -> String {
    let mut out = String::new();

    if !results.tasks.is_empty() {
        out.push_str(&format!("Tasks ({}):\n", results.tasks.len()));
        let mut current_file = String::new();
        for task in &results.tasks {
            let file_str = task.file_path.to_string_lossy();
            if file_str != current_file {
                out.push_str(&format!("  {file_str}\n"));
                current_file = file_str.to_string();
            }
            out.push_str(&format!("    {}\n", format_task_view(task)));
        }
    }

    if !results.bindings.is_empty() {
        if !results.tasks.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("Bindings ({}):\n", results.bindings.len()));
        for binding in &results.bindings {
            out.push_str(&format!(
                "  {} = {}  ({})\n",
                binding.name,
                binding.value,
                binding.file_path.display(),
            ));
        }
    }

    if results.tasks.is_empty() && results.bindings.is_empty() {
        out.push_str("no matches found\n");
    }

    out
}

#[must_use]
pub fn format_search_csv(results: &SearchResults) -> String {
    let mut out = String::from("type,name,value,file\n");
    for task in &results.tasks {
        let status = if task.is_done { "done" } else { "pending" };
        out.push_str(&format!(
            "task,{},{},{}\n",
            csv_escape(&task.title),
            status,
            csv_escape(&task.file_path.to_string_lossy()),
        ));
    }
    for binding in &results.bindings {
        out.push_str(&format!(
            "binding,{},{},{}\n",
            csv_escape(&binding.name),
            csv_escape(&binding.value),
            csv_escape(&binding.file_path.to_string_lossy()),
        ));
    }
    out
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
pub fn format_agenda(agenda: &AgendaView, show_overdue: bool, show_today: bool, show_week: bool) -> String {
    let mut output = String::new();

    if show_overdue && !agenda.overdue.is_empty() {
        output.push_str("# Overdue\n\n");
        for task in &agenda.overdue {
            output.push_str(&format!("  {}\n", format_task_view(task)));
        }
        output.push('\n');
    }

    if show_today && !agenda.today.is_empty() {
        output.push_str("# Today\n\n");
        for task in &agenda.today {
            output.push_str(&format!("  {}\n", format_task_view(task)));
        }
        output.push('\n');
    }

    if show_week && !agenda.this_week.is_empty() {
        output.push_str("# This Week\n\n");
        for task in &agenda.this_week {
            output.push_str(&format!("  {}\n", format_task_view(task)));
        }
        output.push('\n');
    }

    if output.is_empty() {
        output.push_str("No tasks in agenda.\n");
    }

    output
}

#[must_use]
pub fn format_agenda_csv(agenda: &AgendaView) -> String {
    let mut csv = String::from("section,done,due,title,tags,file\n");

    for task in &agenda.overdue {
        csv.push_str(&format!(
            "overdue,{},{},{},{},{}\n",
            task.is_done,
            task.due.as_deref().unwrap_or(""),
            csv_escape(&task.title),
            task.tags.join(";"),
            task.file_path.display(),
        ));
    }

    for task in &agenda.today {
        csv.push_str(&format!(
            "today,{},{},{},{},{}\n",
            task.is_done,
            task.due.as_deref().unwrap_or(""),
            csv_escape(&task.title),
            task.tags.join(";"),
            task.file_path.display(),
        ));
    }

    for task in &agenda.this_week {
        csv.push_str(&format!(
            "this_week,{},{},{},{},{}\n",
            task.is_done,
            task.due.as_deref().unwrap_or(""),
            csv_escape(&task.title),
            task.tags.join(";"),
            task.file_path.display(),
        ));
    }

    csv
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
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn parse_list_without_json() {
        let cmd = parse_args(&[s("list")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.format, OutputFormat::Table);
    }

    #[test]
    fn parse_list_format_csv() {
        let cmd = parse_args(&[s("list"), s("--format"), s("csv")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.format, OutputFormat::Csv);
    }

    #[test]
    fn parse_list_format_table() {
        let cmd = parse_args(&[s("list"), s("--format"), s("table")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.format, OutputFormat::Table);
    }

    #[test]
    fn parse_list_format_json() {
        let cmd = parse_args(&[s("list"), s("--format"), s("json")]).unwrap();
        let Command::List(args) = cmd else { panic!("expected List") };
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn parse_list_format_invalid() {
        let result = parse_args(&[s("list"), s("--format"), s("xml")]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_status_with_json() {
        let cmd = parse_args(&[s("status"), s("--json")]).unwrap();
        let Command::Status(args) = cmd else { panic!("expected Status") };
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn parse_status_format_csv() {
        let cmd = parse_args(&[s("status"), s("--format"), s("csv")]).unwrap();
        let Command::Status(args) = cmd else { panic!("expected Status") };
        assert_eq!(args.format, OutputFormat::Csv);
    }

    #[test]
    fn parse_files_with_json() {
        let cmd = parse_args(&[s("files"), s("--json")]).unwrap();
        let Command::Files(args) = cmd else { panic!("expected Files") };
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn parse_files_format_csv() {
        let cmd = parse_args(&[s("files"), s("--format"), s("csv")]).unwrap();
        let Command::Files(args) = cmd else { panic!("expected Files") };
        assert_eq!(args.format, OutputFormat::Csv);
    }

    // --- parse_args: search ---

    #[test]
    fn parse_search_basic() {
        let cmd = parse_args(&[s("search"), s("milk")]).unwrap();
        let Command::Search(args) = cmd else { panic!("expected Search") };
        assert_eq!(args.query, "milk");
        assert_eq!(args.limit, None);
        assert_eq!(args.db, None);
        assert_eq!(args.format, OutputFormat::Table);
    }

    #[test]
    fn parse_search_with_flags() {
        let cmd = parse_args(&[
            s("search"), s("task"), s("-5"), s("--db"), s("x.db"), s("--format"), s("json"),
        ]).unwrap();
        let Command::Search(args) = cmd else { panic!("expected Search") };
        assert_eq!(args.query, "task");
        assert_eq!(args.limit, Some(5));
        assert_eq!(args.db, Some(PathBuf::from("x.db")));
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn parse_search_json_shorthand() {
        let cmd = parse_args(&[s("search"), s("q"), s("--json")]).unwrap();
        let Command::Search(args) = cmd else { panic!("expected Search") };
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn parse_search_missing_query() {
        let result = parse_args(&[s("search")]);
        assert!(result.is_err());
    }

    // --- parse_agenda_args ---

    #[test]
    fn parse_agenda_default() {
        let cmd = parse_args(&[s("agenda")]).unwrap();
        let Command::Agenda(args) = cmd else { panic!("expected Agenda") };
        assert!(args.show_overdue);
        assert!(args.show_today);
        assert!(args.show_week);
        assert_eq!(args.limit, None);
        assert_eq!(args.db, None);
        assert_eq!(args.format, OutputFormat::Table);
    }

    #[test]
    fn parse_agenda_specific_sections() {
        let cmd = parse_args(&[s("agenda"), s("--overdue"), s("--today")]).unwrap();
        let Command::Agenda(args) = cmd else { panic!("expected Agenda") };
        assert!(args.show_overdue);
        assert!(args.show_today);
        assert!(!args.show_week);
    }

    #[test]
    fn parse_agenda_with_flags() {
        let cmd = parse_args(&[
            s("agenda"), s("--week"), s("-10"), s("--db"), s("a.db"), s("--format"), s("csv"),
        ]).unwrap();
        let Command::Agenda(args) = cmd else { panic!("expected Agenda") };
        assert!(!args.show_overdue);
        assert!(!args.show_today);
        assert!(args.show_week);
        assert_eq!(args.limit, Some(10));
        assert_eq!(args.db, Some(PathBuf::from("a.db")));
        assert_eq!(args.format, OutputFormat::Csv);
    }

    #[test]
    fn parse_agenda_json_shorthand() {
        let cmd = parse_args(&[s("agenda"), s("--json")]).unwrap();
        let Command::Agenda(args) = cmd else { panic!("expected Agenda") };
        assert_eq!(args.format, OutputFormat::Json);
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

    // --- CSV formatters ---

    #[test]
    fn format_tasks_csv_basic() {
        let tasks = vec![TaskView {
            title: "Buy milk".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: Some("2026-03-01".to_string()),
            tags: vec!["shop".to_string()],
        }];
        let csv = format_tasks_csv(&tasks);
        assert!(csv.starts_with("status,due,title,file,tags\n"));
        assert!(csv.contains("pending,2026-03-01,Buy milk,todo.typ,shop\n"));
    }

    #[test]
    fn format_tasks_csv_escapes_commas() {
        let tasks = vec![TaskView {
            title: "Buy eggs, milk".to_string(),
            is_done: true,
            position: 0,
            file_path: PathBuf::from("t.typ"),
            file_title: None,
            due: None,
            tags: vec![],
        }];
        let csv = format_tasks_csv(&tasks);
        assert!(csv.contains("done,,\"Buy eggs, milk\",t.typ,\n"));
    }

    #[test]
    fn format_files_csv_basic() {
        let files = vec![FileView {
            relative_path: PathBuf::from("notes/todo.typ"),
            title: Some("My Tasks".to_string()),
            task_count: 5,
            updated_at: "2026-01-01".to_string(),
        }];
        let csv = format_files_csv(&files);
        assert!(csv.starts_with("path,title,tasks,updated_at\n"));
        assert!(csv.contains("notes/todo.typ,My Tasks,5,2026-01-01\n"));
    }

    #[test]
    fn format_stats_csv_basic() {
        let stats = IndexStats {
            file_count: 3,
            task_count: 10,
            done_count: 4,
            pending_count: 6,
            last_updated: Some("2026-01-15".to_string()),
        };
        let csv = format_stats_csv(&stats);
        assert!(csv.starts_with("metric,value\n"));
        assert!(csv.contains("files,3\n"));
        assert!(csv.contains("tasks,10\n"));
        assert!(csv.contains("done,4\n"));
        assert!(csv.contains("pending,6\n"));
        assert!(csv.contains("last_updated,2026-01-15\n"));
    }

    #[test]
    fn csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn csv_escape_with_comma() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
    }

    #[test]
    fn csv_escape_with_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }
}
