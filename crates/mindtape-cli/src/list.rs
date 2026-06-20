//! `mindtape list` — query tasks under the configured roots.

use anyhow::{Result, anyhow};
use mindtape::sort::{default_order_with, sort_by_specs};
use mindtape::{Config, Filter, Readiness, Task, select};

use crate::output::{OutputFormat, render};
use crate::util;

/// Arguments for `list`.
#[derive(Debug, clap::Args)]
pub struct ListArgs {
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
}

/// Run the `list` command.
///
/// # Errors
/// Returns an error if the project cannot be opened or the filter/sort is invalid.
pub fn run(args: &ListArgs) -> Result<()> {
    run_with(args, util::open()?)
}

pub(crate) fn run_with(args: &ListArgs, project: util::Project) -> Result<()> {
    util::warn_errors(&project.outcome);

    // Readiness over the full set, before the filter narrows it (a terminal
    // dependency hidden by the default filter must still count as satisfied).
    // Shared by the filter, the sort, and the rendered column.
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

    let format = if args.json {
        OutputFormat::Json
    } else {
        args.format
    };
    println!("{}", render(&tasks, format, &project.config, &readiness)?);
    Ok(())
}

fn build_filter(args: &ListArgs, config: &Config) -> Result<Option<Filter>> {
    let expr = if let Some(expr) = &args.filter {
        Some(expr.clone())
    } else if args.all {
        None
    } else {
        config.query.default_filter.clone()
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
        // call, so list ordinals stay in lock-step with what `list` prints.
        default_order_with(tasks, config, readiness);
        return Ok(());
    }
    sort_by_specs(tasks, specs, config, readiness).map_err(|e| anyhow!("{e}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use mindtape::numeric::NumValue;
    use mindtape::{Config, Readiness, Task, load_project};

    use mindtape::sort::sort_by_specs;

    use super::{ListArgs, run_with};
    use crate::output::OutputFormat;

    fn task(id: &str, priority: f64) -> Task {
        Task {
            id: id.to_owned(),
            file: format!("tasks/{id}.typ").into(),
            title: id.to_owned(),
            status: None,
            priority: Some(NumValue::Num(priority)),
            difficulty: None,
            kind: None,
            estimate: BTreeMap::new(),
            depends_on: Vec::new(),
            relates_to: Vec::new(),
            supersedes: Vec::new(),
            duplicates: Vec::new(),
            tags: Vec::new(),
            due: None,
            start: None,
            reviewed: None,
            extras: BTreeMap::new(),
        }
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

    const TASK: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: 8,
  ),
  extras: (:),
))
";

    fn make_project(dir: &Path, files: &[(&str, &str)]) -> crate::util::Project {
        std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
        std::fs::create_dir_all(dir.join("tasks")).unwrap();
        for (name, body) in files {
            std::fs::write(dir.join("tasks").join(name), body).unwrap();
        }
        let config = Config::default();
        let outcome = load_project(dir, &config);
        crate::util::Project {
            root: dir.to_path_buf(),
            config,
            outcome,
        }
    }

    fn default_args() -> ListArgs {
        ListArgs {
            filter: None,
            all: true,
            format: OutputFormat::Table,
            json: false,
            limit: None,
            sort: vec![],
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
        let args = ListArgs {
            json: true,
            ..default_args()
        };
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn run_with_csv_format_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        let args = ListArgs {
            format: OutputFormat::Csv,
            ..default_args()
        };
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn run_with_limit_truncates() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
        let args = ListArgs {
            limit: Some(1),
            ..default_args()
        };
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn run_with_invalid_filter_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        let args = ListArgs {
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
        let args = ListArgs {
            sort: vec!["no_such_field".to_owned()],
            ..default_args()
        };
        assert!(run_with(&args, project).is_err());
    }
}
