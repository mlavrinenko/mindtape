//! `mindtape list` — query tasks under the configured roots.

use std::cmp::Ordering;

use anyhow::{Result, bail};
use mindtape::{Config, Filter, Task, select};

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

    let filter = build_filter(args, &project.config)?;
    let mut tasks = select(project.outcome.tasks, filter.as_ref())?;
    sort_tasks(&mut tasks, &args.sort)?;
    if let Some(limit) = args.limit {
        tasks.truncate(limit);
    }

    let format = if args.json {
        OutputFormat::Json
    } else {
        args.format
    };
    println!("{}", render(&tasks, format)?);
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

fn sort_tasks(tasks: &mut [Task], specs: &[String]) -> Result<()> {
    // Apply least-significant spec first so a stable sort yields the intended
    // precedence (first spec wins ties last).
    for spec in specs.iter().rev() {
        let (field, descending) = parse_spec(spec);
        validate_field(field)?;
        tasks.sort_by(|left, right| {
            let ordering = compare(left, right, field);
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }
    Ok(())
}

fn parse_spec(spec: &str) -> (&str, bool) {
    match spec.split_once(':') {
        Some((field, "desc")) => (field, true),
        Some((field, _)) => (field, false),
        None => (spec, false),
    }
}

fn validate_field(field: &str) -> Result<()> {
    const FIELDS: [&str; 10] = [
        "id",
        "title",
        "status",
        "priority",
        "grade",
        "kind",
        "due",
        "start",
        "estimate_hours",
        "file",
    ];
    if FIELDS.contains(&field) {
        Ok(())
    } else {
        bail!("unknown sort field: {field}")
    }
}

fn compare(left: &Task, right: &Task, field: &str) -> Ordering {
    if field == "estimate_hours" {
        return cmp_opt_num(left.estimate_hours, right.estimate_hours);
    }
    string_key(left, field).cmp(&string_key(right, field))
}

fn string_key(task: &Task, field: &str) -> String {
    match field {
        "id" => task.id.clone(),
        "title" => task.title.clone(),
        "status" => task
            .status
            .as_ref()
            .map_or_else(String::new, |s| s.name.clone()),
        "priority" => task.priority.clone().unwrap_or_default(),
        "grade" => task.grade.clone().unwrap_or_default(),
        "kind" => task.kind.clone().unwrap_or_default(),
        "due" => task.due.clone().unwrap_or_default(),
        "start" => task.start.clone().unwrap_or_default(),
        "file" => task.file.display().to_string(),
        _ => String::new(),
    }
}

fn cmp_opt_num(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::cmp::Ordering;
    use std::collections::BTreeMap;
    use std::path::Path;

    use mindtape::{Config, Task, load_project};

    use super::{ListArgs, cmp_opt_num, compare, parse_spec, run_with, sort_tasks, validate_field};
    use crate::output::OutputFormat;

    fn task(id: &str, priority: &str, estimate: Option<f64>) -> Task {
        Task {
            id: id.to_owned(),
            file: format!("tasks/{id}.typ").into(),
            title: id.to_owned(),
            status: None,
            priority: Some(priority.to_owned()),
            grade: None,
            kind: None,
            estimate_hours: estimate,
            depends_on: Vec::new(),
            relates_to: Vec::new(),
            tags: Vec::new(),
            due: None,
            start: None,
            extras: BTreeMap::new(),
        }
    }

    #[test]
    fn parse_spec_handles_direction() {
        assert_eq!(parse_spec("due:desc"), ("due", true));
        assert_eq!(parse_spec("due:asc"), ("due", false));
        assert_eq!(parse_spec("due"), ("due", false));
    }

    #[test]
    fn validate_field_known_and_unknown() {
        assert!(validate_field("priority").is_ok());
        assert!(validate_field("bogus").is_err());
    }

    #[test]
    fn sort_orders_ascending() {
        let mut tasks = vec![task("b", "low", None), task("a", "high", None)];
        sort_tasks(&mut tasks, &["priority:asc".to_owned()]).unwrap();
        assert_eq!(tasks.first().unwrap().priority.as_deref(), Some("high"));
    }

    #[test]
    fn cmp_opt_num_sorts_none_last() {
        assert_eq!(cmp_opt_num(Some(1.0), None), Ordering::Less);
        assert_eq!(cmp_opt_num(None, Some(1.0)), Ordering::Greater);
        assert_eq!(cmp_opt_num(Some(1.0), Some(2.0)), Ordering::Less);
    }

    #[test]
    fn compare_by_estimate_hours() {
        let low = task("a", "x", Some(2.0));
        let high = task("b", "x", Some(5.0));
        assert_eq!(compare(&low, &high, "estimate_hours"), Ordering::Less);
    }

    const TASK: &str = "#metadata((
  marker: \"mindtape.task\",
  fields: (
    title: \"Task X\",
    status: (name: \"wip\", date: datetime(year: 2026, month: 6, day: 1), note: none),
    priority: \"high\",
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
        crate::util::Project { config, outcome }
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
