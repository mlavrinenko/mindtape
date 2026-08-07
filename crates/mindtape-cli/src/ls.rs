//! `mt ls` — query tasks under the configured roots.

use anstyle::{Effects, Style};
use anyhow::{Result, anyhow};
use mindtape::sort::{default_order_with, sort_by_specs};
use mindtape::{Config, Filter, Readiness, Task, select};

use crate::ls_rules::{self, Applied};
use crate::output::{OutputFormat, render};
use crate::render::color::{Painter, is_color_active};
use crate::util;

/// Arguments for `ls`.
#[derive(Debug, clap::Args)]
pub struct LsArgs {
    /// Filter expression. Omitted applies the project's default view (`[ls]`
    /// `default-view`, else `[views.default]`, else match-all); an empty string
    /// (`''`) lists everything; a bare `@view` applies that view's filter, sort,
    /// and columns.
    pub filter: Option<String>,

    #[command(flatten)]
    pub flags: LsFlags,
}

/// Output-shaping flags shared verbatim between `ls` and `find`
/// (`crate::find`, sugar over `contains()`): everything but the filter
/// itself. Flattened into both so a flag can never exist on one and not the
/// other — the drift `crate::fields` was created to end after `filter.rs` and
/// `sort.rs` disagreed about which fields exist (see
/// `truth/tasks/mt-find-as-sugar-over-contains.typ`).
#[derive(Debug, Clone, clap::Args)]
pub struct LsFlags {
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
    if args.flags.interactive {
        return crate::ls_interactive::run(args, project);
    }
    let (tasks, readiness, applied, config) = resolve_and_select(args, project)?;

    let format = if args.flags.json {
        OutputFormat::Json
    } else {
        args.flags.format
    };
    let body = render(&tasks, format, &config, &readiness, Some(&applied))?;
    if matches!(format, OutputFormat::Path0) {
        // Self-terminated: every path (including the last) already carries
        // its own trailing NUL, so a `println!`-supplied newline would land
        // inside the final `xargs -0` argument instead of after it.
        print!("{body}");
    } else {
        println!("{body}");
    }
    if matches!(format, OutputFormat::Table)
        && let Some(line) = ls_rules::trailer(&applied)
    {
        let painter = Painter::new(is_color_active());
        println!(
            "{}",
            painter.paint(&line, Style::new().effects(Effects::DIMMED))
        );
    }
    Ok(())
}

/// Resolve the effective bundle ([`ls_rules::resolve`]), apply it, select and
/// sort the tasks, and truncate. Consumes `project.outcome.tasks`. Readiness
/// is computed over the full set — before the filter narrows it — so a
/// terminal dependency hidden by the effective filter still counts as
/// satisfied; it is shared by the filter, the sort, and the rendered column.
/// Both the one-shot render and the interactive loop build on this (via
/// [`select_view`]), so their lists (and thus the `#` ordinals) match exactly.
///
/// # Errors
/// Returns an error if the resolved filter or sort spec is invalid.
fn resolve_and_select(
    args: &LsArgs,
    project: util::Project,
) -> Result<(Vec<Task>, Readiness, Applied, Config)> {
    let (bundle, applied) = ls_rules::resolve(&project, args)?;
    let mut config = project.config.clone();
    if !bundle.columns.is_empty() {
        config.ls.columns_order = bundle.columns.clone();
    }
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let filter = Filter::parse(&bundle.filter)?;
    let mut tasks = select(
        project.outcome.tasks,
        Some(&filter),
        &project.config,
        &readiness,
    )?;
    sort_tasks(&mut tasks, &bundle.sort, &project.config, &readiness)?;
    if let Some(limit) = args.flags.limit {
        tasks.truncate(limit);
    }
    Ok((tasks, readiness, applied, config))
}

/// Compute the displayed view: [`resolve_and_select`], keeping only the task
/// list and readiness. The interactive loop ([`crate::ls_interactive`]) calls
/// this so it matches the one-shot render exactly.
///
/// # Errors
/// Returns an error if the resolved filter or sort spec is invalid.
pub(crate) fn select_view(args: &LsArgs, project: util::Project) -> Result<(Vec<Task>, Readiness)> {
    let (tasks, readiness, _applied, _config) = resolve_and_select(args, project)?;
    Ok((tasks, readiness))
}

/// Apply `specs` (empty means the built-in default order); shared by the
/// `ls` render path and [`crate::util`]'s handle-ordinal resolution, so both
/// order tasks identically for any given resolved bundle.
pub(crate) fn sort_tasks(
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

    use super::{LsArgs, resolve_and_select, run_with};
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
            flags: crate::ls::LsFlags {
                format: OutputFormat::Table,
                json: false,
                limit: None,
                sort: vec![],
                columns: None,
                interactive: false,
            },
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
        let mut args = default_args();
        args.flags.json = true;
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn run_with_csv_format_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("x.typ", TASK)]);
        let mut args = default_args();
        args.flags.format = OutputFormat::Csv;
        assert!(run_with(&args, project).is_ok());
    }

    #[test]
    fn run_with_limit_truncates() {
        let dir = tempfile::tempdir().unwrap();
        let project = make_project(dir.path(), &[("a.typ", TASK), ("b.typ", TASK)]);
        let mut args = default_args();
        args.flags.limit = Some(1);
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
        let mut args = default_args();
        args.flags.sort = vec!["no_such_field".to_owned()];
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
        let mut args = default_args();
        args.flags.columns = Some("status,title".to_owned());
        assert!(run_with(&args, view_project(dir.path())).is_ok());
    }

    #[test]
    fn columns_at_view_runs() {
        let dir = tempfile::tempdir().unwrap();
        let mut args = default_args();
        args.flags.columns = Some("@hot".to_owned());
        assert!(run_with(&args, view_project(dir.path())).is_ok());
    }

    #[test]
    fn unknown_column_errors() {
        let dir = tempfile::tempdir().unwrap();
        let mut args = default_args();
        args.flags.columns = Some("bogus".to_owned());
        assert!(run_with(&args, view_project(dir.path())).is_err());
    }

    #[test]
    fn sort_at_view_expands() {
        let dir = tempfile::tempdir().unwrap();
        let mut args = default_args();
        args.flags.sort = vec!["@hot".to_owned()];
        assert!(run_with(&args, view_project(dir.path())).is_ok());
    }

    #[test]
    fn default_view_drives_bare_listing() {
        // A `[views.default]` view supplies the filter for a bare `mt ls`.
        // TASK is `wip` (non-terminal), so a `terminal` default view hides it.
        let dir = tempfile::tempdir().unwrap();
        let config = mindtape::parse_config("[views.default]\nfilter = 'terminal'\n").unwrap();
        let project = crate::test_util::make_project(dir.path(), &[("x.typ", TASK)], config);
        let (tasks, _) = super::select_view(&default_args(), project).unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn resolve_and_select_applies_view_columns() {
        let dir = tempfile::tempdir().unwrap();
        let project = view_project(dir.path());
        let args = LsArgs {
            filter: Some("@hot".to_owned()),
            ..default_args()
        };
        let (_, _, _, config_out) = resolve_and_select(&args, project).unwrap();
        assert_eq!(config_out.ls.columns_order, vec!["status", "title"]);
    }

    #[test]
    fn explicit_empty_filter_bypasses_default_view() {
        // `mt ls ''` is match-all: the default view is ignored entirely.
        let dir = tempfile::tempdir().unwrap();
        let config = mindtape::parse_config("[views.default]\nfilter = 'terminal'\n").unwrap();
        let project = crate::test_util::make_project(dir.path(), &[("x.typ", TASK)], config);
        let args = LsArgs {
            filter: Some(String::new()),
            ..default_args()
        };
        let (tasks, _) = super::select_view(&args, project).unwrap();
        assert_eq!(tasks.len(), 1);
    }
}
