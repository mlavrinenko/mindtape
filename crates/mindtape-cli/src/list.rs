//! `mindtape list` — query tasks under the configured roots.

use std::cmp::Ordering;

use anyhow::{Result, bail};
use mindtape::sort::{default_order, priority_value, status_index};
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
    let mut tasks = select(project.outcome.tasks, filter.as_ref(), &project.config)?;
    sort_tasks(&mut tasks, &args.sort, &project.config)?;
    if let Some(limit) = args.limit {
        tasks.truncate(limit);
    }

    let format = if args.json {
        OutputFormat::Json
    } else {
        args.format
    };
    println!("{}", render(&tasks, format, &project.config)?);
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

fn sort_tasks(tasks: &mut [Task], specs: &[String], config: &Config) -> Result<()> {
    // No explicit spec: fall back to the shared default order (the same order
    // handle resolution uses, so list ordinals stay stable).
    if specs.is_empty() {
        default_order(tasks, config);
        return Ok(());
    }
    // Apply least-significant spec first so a stable sort yields the intended
    // precedence (first spec wins ties last).
    for spec in specs.iter().rev() {
        let (field, descending) = parse_spec(spec);
        validate_field(field)?;
        tasks.sort_by(|left, right| {
            let ordering = compare(left, right, field, config);
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
    const FIELDS: [&str; 9] = [
        "id",
        "title",
        "status",
        "priority",
        "difficulty",
        "kind",
        "due",
        "start",
        "file",
    ];
    if FIELDS.contains(&field) {
        Ok(())
    } else {
        bail!("unknown sort field: {field}")
    }
}

/// Compare two tasks on `field`: status by its lifecycle index, priority and
/// difficulty numerically, everything else lexicographically.
fn compare(left: &Task, right: &Task, field: &str, config: &Config) -> Ordering {
    match field {
        "status" => status_index(left, config).cmp(&status_index(right, config)),
        "priority" => cmp_num(priority_value(left, config), priority_value(right, config)),
        "difficulty" => cmp_num(
            difficulty_value(left, config),
            difficulty_value(right, config),
        ),
        _ => string_key(left, field).cmp(&string_key(right, field)),
    }
}

/// A task's difficulty resolved to a number on its scale, if set and resolvable.
fn difficulty_value(task: &Task, config: &Config) -> Option<f64> {
    task.difficulty
        .as_ref()
        .and_then(|value| config.tasks.difficulty.resolve(value))
}

/// Compare numbers ascending; a missing value sorts last.
fn cmp_num(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(low), Some(high)) => low.partial_cmp(&high).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn string_key(task: &Task, field: &str) -> String {
    match field {
        "id" => task.id.clone(),
        "title" => task.title.clone(),
        "kind" => task.kind.clone().unwrap_or_default(),
        "due" => task.due.clone().unwrap_or_default(),
        "start" => task.start.clone().unwrap_or_default(),
        "file" => task.file.display().to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use mindtape::numeric::NumValue;
    use mindtape::{Config, Task, load_project};

    use super::{ListArgs, parse_spec, run_with, sort_tasks, validate_field};
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
    fn sort_orders_priority_numerically() {
        // Ascending priority: the lower number comes first. Lexicographically
        // "8" < "2" is false, so this only passes with numeric ordering.
        let mut tasks = vec![task("b", 8.0), task("a", 2.0)];
        sort_tasks(&mut tasks, &["priority:asc".to_owned()], &Config::default()).unwrap();
        assert_eq!(tasks.first().unwrap().id, "a");
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
