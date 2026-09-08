//! `mt ls` — query tasks under the configured roots.

use anstyle::{Effects, Style};
use anyhow::{Result, anyhow};
use mindtape::sort::{default_order_with, sort_by_specs};
use mindtape::{Config, Filter, Readiness, Task, select};

use crate::ls_columns::apply_column_rules;
use crate::ls_rules::{self, Applied};
use crate::output::{Envelope, OutputFormat, Truncation, render};
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

    /// Limit output to N tasks. Applied after the filter and the sort, so it
    /// composes with `--sort` and a named view; a table says what it cut,
    /// `--json` reports it in the envelope, and the record formats (`csv`,
    /// `path`, `path0`) stay records only.
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
    let listing = resolve_and_select(args, project)?;

    let format = effective_format(args);
    let envelope = Envelope {
        applied: &listing.applied,
        truncated: listing.truncated,
    };
    let body = render(
        &listing.tasks,
        format,
        &listing.config,
        &listing.readiness,
        Some(envelope),
    )?;
    if matches!(format, OutputFormat::Path0) {
        // Self-terminated: every path (including the last) already carries
        // its own trailing NUL, so a `println!`-supplied newline would land
        // inside the final `xargs -0` argument instead of after it.
        print!("{body}");
    } else {
        println!("{body}");
    }
    if matches!(format, OutputFormat::Table) {
        print_trailers(&listing);
    }
    Ok(())
}

/// The format this run renders in: `--json` is shorthand for `--format json`
/// and wins, otherwise `--format` (which has its own default).
fn effective_format(args: &LsArgs) -> OutputFormat {
    if args.flags.json {
        OutputFormat::Json
    } else {
        args.flags.format
    }
}

/// The effective format's config-facing name, backing `format()` in a
/// condition. Read off clap's own value enum rather than a second `match`, so
/// a new `--format` variant cannot arrive with a name the condition language
/// disagrees about.
fn format_name(args: &LsArgs) -> String {
    clap::ValueEnum::to_possible_value(&effective_format(args))
        .map_or_else(String::new, |value| value.get_name().to_owned())
}

/// The dim lines under a rendered table: the `applied:` attribution, then the
/// `truncated:` count when `--limit` hid rows. Table only — every other format
/// is a record stream a reader parses, so a trailing prose line there is
/// corruption, not information.
fn print_trailers(listing: &Listing) {
    let painter = Painter::new(is_color_active());
    let dim = Style::new().effects(Effects::DIMMED);
    let lines = ls_rules::trailer(&listing.applied)
        .into_iter()
        .chain(listing.truncated.map(Truncation::footer));
    for line in lines {
        println!("{}", painter.paint(&line, dim));
    }
}

/// The resolved listing one `mt ls` run prints: the selected tasks, the
/// readiness and attribution they were selected under, the config the render
/// reads (view columns folded in), and what `--limit` cut — `None` when the
/// listing is complete.
pub(crate) struct Listing {
    pub tasks: Vec<Task>,
    pub readiness: Readiness,
    pub applied: Applied,
    pub config: Config,
    pub truncated: Option<Truncation>,
}

/// Resolve the effective bundle ([`ls_rules::resolve`]), apply it, select and
/// sort the tasks, and truncate. Consumes `project.outcome.tasks`. Readiness
/// is computed over the full set — before the filter narrows it — so a
/// terminal dependency hidden by the effective filter still counts as
/// satisfied; it is shared by the filter, the sort, and the rendered column.
///
/// `--limit` is applied last, over the filtered and sorted list, so it
/// composes with `--sort` and a view's own sort chain rather than racing
/// them; the match count is taken before the cut, which is what makes the
/// reported `total` the number the caller would see with a bigger `--limit`.
/// Both the one-shot render and the interactive loop build on this, so their
/// lists (and thus the `#` ordinals) match exactly.
///
/// # Errors
/// Returns an error if the resolved filter or sort spec is invalid.
pub(crate) fn resolve_and_select(args: &LsArgs, project: util::Project) -> Result<Listing> {
    let env = ls_rules::build_env(&project, &format_name(args));
    let (bundle, applied) = ls_rules::resolve(&project, args, &env)?;
    let mut config = project.config.clone();
    if !bundle.columns.is_empty() {
        config.ls.columns_order = bundle.columns.clone();
    }
    // Conditional per-column visibility, resolved against the same environment
    // `[[ls.rules]]` just used, so a condition means one thing per run.
    apply_column_rules(&mut config, &env)?;
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let filter = Filter::parse(&bundle.filter)?;
    let mut tasks = select(
        project.outcome.tasks,
        Some(&filter),
        &project.config,
        &readiness,
    )?;
    sort_tasks(&mut tasks, &bundle.sort, &project.config, &readiness)?;
    let truncated = Truncation::new(args.flags.limit, tasks.len());
    if let Some(limit) = args.flags.limit {
        tasks.truncate(limit);
    }
    Ok(Listing {
        tasks,
        readiness,
        applied,
        config,
        truncated,
    })
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
#[path = "ls_tests.rs"]
mod tests;
