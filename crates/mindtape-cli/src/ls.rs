//! `mindtape ls` — query tasks under the configured roots.

use anyhow::{Result, anyhow};
use mindtape::sort::{default_order_with, sort_by_specs};
use mindtape::{Config, Filter, Readiness, Task, select};

use crate::output::{OutputFormat, render};
use crate::util;

/// Arguments for `ls`.
#[derive(Debug, clap::Args)]
pub struct LsArgs {
    /// Filter expression (defaults to the project's `query.default_filter`).
    pub filter: Option<String>,

    /// Ignore the configured default filter; show everything.
    #[arg(long)]
    pub all: bool,

    /// Output format.
    #[arg(long, value_enum, default_value_t)]
    pub format: OutputFormat,

    /// Shorthand for `--format json`.
    #[arg(long)]
    pub json: bool,

    /// Limit output to N tasks.
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    /// Sort spec(s), e.g. `--sort due:asc --sort priority:desc`.
    #[arg(long)]
    pub sort: Vec<String>,

    /// Hold the terminal in a key loop: type the `#` ordinal to highlight a task,
    /// Enter opens it in the editor (`go`). Table-only; requires a TTY.
    #[arg(short = 'i', long, conflicts_with = "json")]
    pub interactive: bool,
}

/// Run the `ls` command.
///
/// # Errors
/// Returns an error if the project cannot be opened or the filter/sort is invalid.
pub fn run(args: &LsArgs) -> Result<()> {
    run_with(args, util::open()?)
}

pub(crate) fn run_with(args: &LsArgs, project: util::Project) -> Result<()> {
    util::warn_errors(&project.outcome);
    if args.interactive {
        return crate::ls_interactive::run(args, project);
    }
    let config = project.config.clone();
    let (tasks, readiness) = select_view(args, project)?;

    let format = if args.json {
        OutputFormat::Json
    } else {
        args.format
    };
    println!("{}", render(&tasks, format, &config, &readiness)?);
    Ok(())
}

/// Compute the displayed view: apply the filter, select, sort, and truncate.
///
/// Consumes `project.outcome.tasks`. Readiness is computed over the full set —
/// before the filter narrows it — so a terminal dependency hidden by the default
/// filter still counts as satisfied; it is shared by the filter, the sort, and
/// the rendered column. Both the one-shot render and the interactive loop call
/// this so their lists (and thus the `#` ordinals) match exactly.
///
/// # Errors
/// Returns an error if the filter or sort spec is invalid.
pub(crate) fn select_view(args: &LsArgs, project: util::Project) -> Result<(Vec<Task>, Readiness)> {
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let filter = build_filter(args, &project.config)?;
    let mut tasks = select(
        project.outcome.tasks,
        filter.as_ref(),
        &project.config,
        &readiness,
    )?;
    sort_tasks(&mut tasks, &args.sort, &project.config, &readiness)?;
    if let Some(limit) = args.limit {
        tasks.truncate(limit);
    }
    Ok((tasks, readiness))
}

fn build_filter(args: &LsArgs, config: &Config) -> Result<Option<Filter>> {
    let expr = if let Some(expr) = &args.filter {
        Some(expr.clone())
    } else if args.all {
        None
    } else {
        config.ls.default_filter.clone()
    };
    match expr {
        Some(expr) => Ok(Some(Filter::parse(&expr)?)),
        None => Ok(None),
    }
}

fn sort_tasks(
    tasks: &mut [Task],
    specs: &[String],
    config: &Config,
    readiness: &Readiness,
) -> Result<()> {
    if specs.is_empty() {
        // default_order_with honours config.query.default_sort when set, then
        // falls back to the built-in chain. Handle resolution uses the same
        // call, so ls ordinals stay in lock-step with what `ls` prints.
        default_order_with(tasks, config, readiness);
        return Ok(());
    }
    sort_by_specs(tasks, specs, config, readiness).map_err(|e| anyhow!("{e}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::Path;

    use mindtape::numeric::NumValue;
    use mindtape::test_support::{TASK_SRC as TASK, bare_task};
    use mindtape::{Config, Readiness, Task};

    use mindtape::sort::sort_by_specs;

    use super::{LsArgs, run_with};
    use crate::output::OutputFormat;

    fn task(id: &str, priority: f64) -> Task {
        let mut t = bare_task(id);
        t.priority = Some(NumValue::Num(priority));
        t
    }

    #[test]
    fn sort_by_specs_orders_priority_numerically() {
        // Ascending: lower number first. Lexicographic "8" < "2" is false,
        // so this only passes with numeric ordering.
        let mut tasks = vec![task("b", 8.0), task("a", 2.0)];
        sort_by_specs(
            &mut tasks,
            &["priority:asc".to_owned()],
            &Config::default(),
            &Readiness::empty(),
        )
        .unwrap();
        assert_eq!(tasks.first().unwrap().id, "a");
    }

    #[test]
    fn sort_by_specs_rejects_unknown_field() {
        let mut tasks = vec![task("a", 1.0)];
        assert!(
            sort_by_specs(
                &mut tasks,
                &["no_such_field".to_owned()],
                &Config::default(),
                &Readiness::empty(),
            )
            .is_err()
        );
    }

    fn make_project(dir: &Path, files: &[(&str, &str)]) -> crate::util::Project {
        crate::test_util::make_project(dir, files, Config::default())
    }

    fn default_args() -> LsArgs {
        LsArgs {
            filter: None,
            all: true,
            format: OutputFormat::Table,
            json: false,
            limit: None,
            sort: vec![],
            interactive: false,
        }
    }

    #[test]
    fn run_with_table_format_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        assert!(run_with(&default_args(), project).is_ok());
    }

    #[test]
    fn run_with_json_flag_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        let args = LsArgs {
            json: true,
            ..default_args()
        };
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn run_with_csv_format_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        let args = LsArgs {
            format: OutputFormat::Csv,
            ..default_args()
        };
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn run_with_limit_truncates() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
        let args = LsArgs {
            limit: Some(1),
            ..default_args()
        };
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn run_with_invalid_filter_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        let args = LsArgs {
            filter: Some("!!bad!!".to_owned()),
            all: false,
            ..default_args()
        };
        assert!(run_with(&args, project).is_err());
    }

    #[test]
    fn run_with_invalid_sort_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        let args = LsArgs {
            sort: vec!["no_such_field".to_owned()],
            ..default_args()
        };
        assert!(run_with(&args, project).is_err());
    }
}
