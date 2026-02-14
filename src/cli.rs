use std::path::PathBuf;

use typst::foundations::Datetime;

use crate::eval::Task;

/// Parsed CLI command.
pub enum Command {
    Eval(EvalArgs),
    Watch(WatchArgs),
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

pub fn parse_args(args: &[String]) -> Result<Command, String> {
    if args.first().map(|s| s.as_str()) == Some("watch") {
        return parse_watch_args(&args[1..]);
    }
    parse_eval_args(args).map(Command::Eval)
}

fn parse_eval_args(args: &[String]) -> Result<EvalArgs, String> {
    let mut file: Option<PathBuf> = None;
    let mut due = false;
    let mut limit: Option<usize> = None;

    for arg in args {
        if arg == "--due" {
            due = true;
        } else if arg.starts_with('-') && arg[1..].parse::<usize>().is_ok() {
            limit = Some(arg[1..].parse().unwrap());
        } else {
            file = Some(PathBuf::from(arg));
        }
    }

    let file = file.ok_or_else(|| "Usage: mindtape <file.typ> [--due] [-N]\n       mindtape watch [<path>] [--config <file>]".to_string())?;

    Ok(EvalArgs { file, due, limit })
}

fn parse_watch_args(args: &[String]) -> Result<Command, String> {
    let mut path: Option<PathBuf> = None;
    let mut config: Option<PathBuf> = None;
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--config" {
            i += 1;
            config = Some(
                args.get(i)
                    .map(PathBuf::from)
                    .ok_or_else(|| "--config requires a path argument".to_string())?,
            );
        } else if args[i].starts_with('-') {
            return Err(format!("unknown watch flag: {}", args[i]));
        } else {
            path = Some(PathBuf::from(&args[i]));
        }
        i += 1;
    }

    Ok(Command::Watch(WatchArgs { path, config }))
}

pub fn filter_and_sort(tasks: Vec<Task>, due_only: bool, limit: Option<usize>) -> Vec<Task> {
    // Filter out completed tasks
    let mut tasks: Vec<_> = tasks.into_iter().filter(|t| !t.done).collect();

    // If due_only: keep only tasks with due dates, sort by due date ascending
    if due_only {
        tasks.retain(|t| t.due.is_some());
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

pub fn format_task(task: &Task) -> String {
    match &task.due {
        Some(dt) => format!("- (due {}) {}", format_due(dt), task.title),
        None => format!("- {}", task.title),
    }
}

pub fn format_due(dt: &Datetime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        dt.year().unwrap_or(0),
        dt.month().unwrap_or(0),
        dt.day().unwrap_or(0),
    )
}

pub fn due_sort_key(dt: &Datetime) -> (i32, u8, u8) {
    (
        dt.year().unwrap_or(0),
        dt.month().unwrap_or(0),
        dt.day().unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> String {
        val.to_string()
    }

    fn make_task(title: &str, done: bool, due: Option<Datetime>) -> Task {
        Task { title: title.to_string(), done, due, tags: vec![], position: 0 }
    }

    fn ymd(y: i32, m: u8, d: u8) -> Datetime {
        Datetime::from_ymd(y, m, d).unwrap()
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
}
