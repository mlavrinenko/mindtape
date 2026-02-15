pub mod commands;
pub mod format;
pub mod util;

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

pub use commands::agenda::AgendaArgs;
pub use commands::check::CheckArgs;
pub use commands::deps::DepsArgs;
pub use commands::list::ListArgs;
pub use commands::search::SearchArgs;
pub use commands::set::SetArgs;
pub use commands::watch::WatchArgs;

// Re-export per-command formatting so main.rs can use cli::format_* as before.
pub use commands::agenda::{format_agenda, format_agenda_csv};
pub use commands::deps::{format_all_deps, format_all_deps_csv, format_deps, format_deps_csv};
pub use commands::eval::{due_sort_key, filter_and_sort, format_due, format_task};
pub use commands::search::{format_search_csv, format_search_results};
pub use format::{
    format_file_view, format_files_csv, format_stats, format_stats_csv, format_task_view,
    format_tasks_csv,
};

/// Output format for query commands.
#[derive(Debug, Clone, Copy, PartialEq, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
}

/// Shared query options flattened into commands that query the database.
#[derive(Parser, Debug, Default)]
pub struct QueryOpts {
    /// Path to `SQLite` database
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value_t)]
    pub format: OutputFormat,

    /// Shorthand for --format json
    #[arg(long)]
    pub json: bool,
}

impl QueryOpts {
    /// Resolve the effective output format (`--json` overrides `--format`).
    #[must_use]
    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            self.format
        }
    }
}

#[derive(Parser)]
#[command(name = "mindtape", about = "File-based task tracker using Typst")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Typst file to evaluate (shorthand for eval mode)
    pub file: Option<PathBuf>,

    /// Show only tasks with due dates, sorted by date
    #[arg(long)]
    pub due: bool,

    /// Limit output to N items
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Command {
    /// Watch and index folders
    Watch(WatchArgs),
    /// List tasks from the index
    List(ListArgs),
    /// Search tasks and bindings
    Search(SearchArgs),
    /// Show agenda (overdue, today, this week)
    Agenda(AgendaArgs),
    /// Show index statistics
    Status(StatusArgs),
    /// List indexed files
    Files(FilesArgs),
    /// Show file dependencies
    Deps(DepsArgs),
    /// Toggle a task's checkbox
    Check(CheckArgs),
    /// Update task properties (due date, tags)
    Set(SetArgs),
}

/// Args for the `status` subcommand (query-only).
#[derive(Parser, Debug)]
pub struct StatusArgs {
    #[command(flatten)]
    pub query: QueryOpts,
}

/// Args for the `files` subcommand (query-only).
#[derive(Parser, Debug)]
pub struct FilesArgs {
    #[command(flatten)]
    pub query: QueryOpts,
}

/// Rewrite `-3` to `-n 3` so clap can parse the `-N` shorthand.
#[must_use]
pub fn preprocess_args(raw: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(raw.len() + 1);
    for arg in raw {
        if let Some(rest) = arg.strip_prefix('-')
            && !rest.is_empty()
            && rest.chars().all(|c| c.is_ascii_digit())
        {
            out.push("-n".to_string());
            out.push(rest.to_string());
            continue;
        }
        out.push(arg);
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn preprocess_rewrites_dash_number() {
        let args = vec![
            "mindtape".to_string(),
            "list".to_string(),
            "-5".to_string(),
        ];
        let result = preprocess_args(args);
        assert_eq!(result, vec!["mindtape", "list", "-n", "5"]);
    }

    #[test]
    fn preprocess_leaves_other_args_alone() {
        let args = vec![
            "mindtape".to_string(),
            "list".to_string(),
            "--tag".to_string(),
            "work".to_string(),
        ];
        let result = preprocess_args(args.clone());
        assert_eq!(result, args);
    }

    #[test]
    fn preprocess_does_not_rewrite_double_dash() {
        let args = vec!["mindtape".to_string(), "--3".to_string()];
        let result = preprocess_args(args.clone());
        assert_eq!(result, args);
    }

    #[test]
    fn query_opts_json_overrides_format() {
        let opts = QueryOpts {
            db: None,
            format: OutputFormat::Table,
            json: true,
        };
        assert_eq!(opts.output_format(), OutputFormat::Json);
    }

    #[test]
    fn query_opts_format_csv() {
        let opts = QueryOpts {
            db: None,
            format: OutputFormat::Csv,
            json: false,
        };
        assert_eq!(opts.output_format(), OutputFormat::Csv);
    }

    // --- Clap parse smoke tests ---

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(preprocess_args(args.iter().map(ToString::to_string).collect())).unwrap()
    }

    #[test]
    fn parse_eval_file() {
        let cli = parse(&["mindtape", "foo.typ"]);
        assert!(cli.command.is_none());
        assert_eq!(cli.file, Some(PathBuf::from("foo.typ")));
    }

    #[test]
    fn parse_eval_file_with_due() {
        let cli = parse(&["mindtape", "foo.typ", "--due"]);
        assert!(cli.command.is_none());
        assert!(cli.due);
    }

    #[test]
    fn parse_eval_file_with_limit() {
        let cli = parse(&["mindtape", "foo.typ", "-3"]);
        assert!(cli.command.is_none());
        assert_eq!(cli.limit, Some(3));
    }

    #[test]
    fn parse_list_bare() {
        let cli = parse(&["mindtape", "list"]);
        assert!(matches!(cli.command, Some(Command::List(_))));
    }

    #[test]
    fn parse_watch_bare() {
        let cli = parse(&["mindtape", "watch"]);
        assert!(matches!(cli.command, Some(Command::Watch(_))));
    }

    #[test]
    fn parse_search_keyword() {
        let cli = parse(&["mindtape", "search", "milk"]);
        let Some(Command::Search(args)) = cli.command else {
            panic!("expected Search");
        };
        assert_eq!(args.keyword, "milk");
    }

    #[test]
    fn parse_agenda_sections() {
        let cli = parse(&["mindtape", "agenda", "--overdue", "--today"]);
        let Some(Command::Agenda(args)) = cli.command else {
            panic!("expected Agenda");
        };
        assert!(args.overdue);
        assert!(args.today);
        assert!(!args.week);
    }

    #[test]
    fn parse_check_task_id() {
        let cli = parse(&["mindtape", "check", "abc-123"]);
        let Some(Command::Check(args)) = cli.command else {
            panic!("expected Check");
        };
        assert_eq!(args.task_id, "abc-123");
    }

    #[test]
    fn parse_status_json() {
        let cli = parse(&["mindtape", "status", "--json"]);
        let Some(Command::Status(args)) = cli.command else {
            panic!("expected Status");
        };
        assert_eq!(args.query.output_format(), OutputFormat::Json);
    }

    #[test]
    fn parse_list_with_limit_shorthand() {
        let cli = parse(&["mindtape", "list", "-5"]);
        let Some(Command::List(args)) = cli.command else {
            panic!("expected List");
        };
        assert_eq!(args.limit, Some(5));
    }

    #[test]
    fn parse_deps_with_file() {
        let cli = parse(&["mindtape", "deps", "--file", "todo.typ"]);
        let Some(Command::Deps(args)) = cli.command else {
            panic!("expected Deps");
        };
        assert_eq!(args.file, Some(PathBuf::from("todo.typ")));
    }

    #[test]
    fn parse_set_due() {
        let cli = parse(&["mindtape", "set", "abc-123", "--due", "2026-03-15"]);
        let Some(Command::Set(args)) = cli.command else {
            panic!("expected Set");
        };
        assert_eq!(args.task_id, "abc-123");
        assert_eq!(args.due, Some("2026-03-15".to_string()));
        assert!(!args.no_due);
    }

    #[test]
    fn parse_set_no_due() {
        let cli = parse(&["mindtape", "set", "abc-123", "--no-due"]);
        let Some(Command::Set(args)) = cli.command else {
            panic!("expected Set");
        };
        assert!(args.no_due);
        assert!(args.due.is_none());
    }

    #[test]
    fn parse_set_tags() {
        let cli = parse(&[
            "mindtape", "set", "abc-123",
            "--add-tag", "work", "--remove-tag", "old",
        ]);
        let Some(Command::Set(args)) = cli.command else {
            panic!("expected Set");
        };
        assert_eq!(args.add_tag, vec!["work"]);
        assert_eq!(args.remove_tag, vec!["old"]);
    }

    #[test]
    fn parse_set_combined() {
        let cli = parse(&[
            "mindtape", "set", "*37f8",
            "--due", "2026-04-01",
            "--add-tag", "urgent",
            "--add-tag", "work",
        ]);
        let Some(Command::Set(args)) = cli.command else {
            panic!("expected Set");
        };
        assert_eq!(args.task_id, "*37f8");
        assert_eq!(args.due, Some("2026-04-01".to_string()));
        assert_eq!(args.add_tag, vec!["urgent", "work"]);
    }
}
