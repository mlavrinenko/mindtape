//! `mt ls` — query tasks under the configured roots.

use anyhow::{Result, anyhow};
use mindtape::sort::{default_order_with, sort_by_specs};
use mindtape::{Config, Filter, LS_COLUMNS, Readiness, Task, ViewSpec, select};

use crate::output::{OutputFormat, render};
use crate::util;

/// Arguments for `ls`.
#[derive(Debug, clap::Args)]
pub struct LsArgs {
    /// Filter expression. Omitted uses the project's `[ls] default-filter`;
    /// an empty string (`''`) lists everything; a bare `@view` applies that
    /// view's filter, sort, and columns.
    pub filter: Option<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t)]
    pub format: OutputFormat,

    /// Shorthand for `--format json`.
    #[arg(long)]
    pub json: bool,

    /// Limit output to N tasks.
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    /// Sort spec(s), e.g. `--sort due:asc --sort priority:desc`. A `@view` spec
    /// splices that view's sort chain.
    #[arg(long)]
    pub sort: Vec<String>,

    /// Columns to show: a comma list of column names, or `@view` for a view's
    /// column set. Overrides `[ls] columns-order` for this run.
    #[arg(long, value_name = "COLS")]
    pub columns: Option<String>,

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
    let mut config = project.config.clone();
    if let Some(cols) = effective_columns(args, &config)? {
        config.ls.columns_order = cols;
    }
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
    let sort = effective_sort(args, &project.config)?;
    sort_tasks(&mut tasks, &sort, &project.config, &readiness)?;
    if let Some(limit) = args.limit {
        tasks.truncate(limit);
    }
    Ok((tasks, readiness))
}

/// Resolve the filter to apply. Omitted uses `[ls] default-filter`; an explicit
/// expression (including empty, which is match-all, and a bare `@view`) is used
/// verbatim.
fn build_filter(args: &LsArgs, config: &Config) -> Result<Option<Filter>> {
    let expr = match &args.filter {
        Some(expr) => Some(expr.clone()),
        None => config.ls.default_filter.clone(),
    };
    match expr {
        Some(expr) => Ok(Some(Filter::parse(&expr)?)),
        None => Ok(None),
    }
}

/// The view a bare `@name` filter argument names (bundle mode), if any. `None`
/// when the filter is absent or is a compound expression rather than a lone
/// view reference. Errors when the named view does not exist.
fn bundle_view<'a>(args: &LsArgs, config: &'a Config) -> Result<Option<&'a ViewSpec>> {
    let Some(filter) = &args.filter else {
        return Ok(None);
    };
    let Some(name) = bare_view_ref(filter) else {
        return Ok(None);
    };
    let spec = config
        .views
        .get(name)
        .ok_or_else(|| anyhow!("unknown view: {name}"))?;
    Ok(Some(spec))
}

/// A filter argument that is exactly one `@name` token (bundle mode), yielding
/// the bare name; `None` for a compound expression.
fn bare_view_ref(filter: &str) -> Option<&str> {
    let name = filter.trim().strip_prefix('@')?;
    let ok = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    ok.then_some(name)
}

/// The sort chain to apply: explicit `--sort` wins, else a bundle view's sort,
/// else empty (the default order). Any `@view` spec is spliced to that view's
/// sort chain.
fn effective_sort(args: &LsArgs, config: &Config) -> Result<Vec<String>> {
    let base = if !args.sort.is_empty() {
        args.sort.clone()
    } else if let Some(view) = bundle_view(args, config)? {
        view.sort.clone()
    } else {
        Vec::new()
    };
    let mut out = Vec::with_capacity(base.len());
    for spec in &base {
        if let Some(name) = spec.strip_prefix('@') {
            let view = config
                .views
                .get(name)
                .ok_or_else(|| anyhow!("unknown view: {name}"))?;
            out.extend(view.sort.iter().cloned());
        } else {
            out.push(spec.clone());
        }
    }
    Ok(out)
}

/// The column override to apply: explicit `--columns` wins, else a bundle view's
/// columns, else `None` (keep `[ls] columns-order`).
fn effective_columns(args: &LsArgs, config: &Config) -> Result<Option<Vec<String>>> {
    if let Some(spec) = &args.columns {
        return Ok(Some(resolve_columns(spec, config)?));
    }
    match bundle_view(args, config)? {
        Some(view) if !view.columns.is_empty() => Ok(Some(view.columns.clone())),
        _ => Ok(None),
    }
}

/// Resolve a `--columns` spec (a comma list of column names, or `@view`) to a
/// validated list of `[ls]` column names.
fn resolve_columns(spec: &str, config: &Config) -> Result<Vec<String>> {
    let names: Vec<String> = if let Some(name) = spec.strip_prefix('@') {
        let view = config
            .views
            .get(name)
            .ok_or_else(|| anyhow!("unknown view: {name}"))?;
        view.columns.clone()
    } else {
        spec.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    };
    for name in &names {
        if !LS_COLUMNS.contains(&name.as_str()) {
            return Err(anyhow!(
                "unknown column `{name}`, expected one of {}",
                LS_COLUMNS.join(", ")
            ));
        }
    }
    Ok(names)
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
            format: OutputFormat::Table,
            json: false,
            limit: None,
            sort: vec![],
            columns: None,
            interactive: false,
        }
    }

    fn config_with_views() -> Config {
        mindtape::parse_config(
            "[views.hot]\nfilter = 'priority > 5'\nsort = ['priority:desc']\n\
             columns = ['status', 'title']\n\
             [views.broken]\nfilter = 'blocked'\n",
        )
        .unwrap()
    }

    fn view_project(dir: &Path) -> crate::util::Project {
        crate::test_util::make_project(dir, &[("x.typ", TASK)], config_with_views())
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

    #[test]
    fn empty_filter_lists_everything() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        let args = LsArgs {
            filter: Some(String::new()),
            ..default_args()
        };
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn bundle_view_reference_runs() {
        let dir = tempfile::tempdir().unwrap();
        let args = LsArgs {
            filter: Some("@hot".to_owned()),
            ..default_args()
        };
        assert!(run_with(&args, view_project(dir.path())).is_ok());
    }

    #[test]
    fn unknown_bundle_view_errors() {
        let dir = tempfile::tempdir().unwrap();
        let args = LsArgs {
            filter: Some("@ghost".to_owned()),
            ..default_args()
        };
        assert!(run_with(&args, view_project(dir.path())).is_err());
    }

    #[test]
    fn columns_flag_selects_named_columns() {
        let dir = tempfile::tempdir().unwrap();
        let args = LsArgs {
            columns: Some("status,title".to_owned()),
            ..default_args()
        };
        assert!(run_with(&args, view_project(dir.path())).is_ok());
    }

    #[test]
    fn columns_at_view_runs() {
        let dir = tempfile::tempdir().unwrap();
        let args = LsArgs {
            columns: Some("@hot".to_owned()),
            ..default_args()
        };
        assert!(run_with(&args, view_project(dir.path())).is_ok());
    }

    #[test]
    fn unknown_column_errors() {
        let dir = tempfile::tempdir().unwrap();
        let args = LsArgs {
            columns: Some("bogus".to_owned()),
            ..default_args()
        };
        assert!(run_with(&args, view_project(dir.path())).is_err());
    }

    #[test]
    fn sort_at_view_expands() {
        let dir = tempfile::tempdir().unwrap();
        let args = LsArgs {
            sort: vec!["@hot".to_owned()],
            ..default_args()
        };
        assert!(run_with(&args, view_project(dir.path())).is_ok());
    }
}
