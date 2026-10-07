//! `mt ls` — query tasks under the configured roots.

use anstyle::{Effects, Style};
use anyhow::{Result, anyhow};
use mindtape::sort::{default_order_with, sort_by_specs};
use mindtape::{ConditionEnv, Config, Filter, Readiness, StemHashes, Task, select};

use crate::ls_rules::{self, Applied};
use crate::output::{Envelope, OutputFormat, Truncation, render};
use crate::render::RenderCtx;
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
/// other — the drift `crate::model::fields` was created to end after `filter/mod.rs` and
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

    /// Drop N rows after the sort, before `-n` — page two of a `-n 20`
    /// listing is `--skip 20`. A truncated table's footer and `--json`'s
    /// `truncated.next_skip` already name the `--skip` that continues from
    /// this run, advanced past what it showed; pass that value rather than
    /// adding to this one by hand.
    #[arg(long)]
    pub skip: Option<usize>,

    /// Sort spec(s), e.g. `--sort due:asc --sort priority:desc`. A `@view` spec
    /// splices that view's sort chain.
    #[arg(long)]
    pub sort: Vec<String>,

    /// Columns to show: a comma list of column names, or `@view` for a view's
    /// column set. Overrides `[ls] columns-order` for this run.
    #[arg(long, value_name = "COLS")]
    pub columns: Option<String>,

    /// Hold the terminal in a key loop: type a `slug#` hash to highlight a task,
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
    let retired = listing.config.retired_refs();
    let envelope = Envelope {
        applied: &listing.applied,
        truncated: listing.truncated,
        retired: &retired,
    };
    let body = render(&listing.tasks, format, listing.render_ctx(), Some(envelope))?;
    if matches!(format, OutputFormat::Path0) {
        // Self-terminated: every path (including the last) already carries
        // its own trailing NUL, so a `println!`-supplied newline would land
        // inside the final `xargs -0` argument instead of after it.
        print!("{body}");
    } else {
        println!("{body}");
    }
    if matches!(format, OutputFormat::Table) {
        print_trailers(args, &listing);
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

/// The dim lines under a rendered table: the `applied:` attribution, then —
/// when `--limit` hid rows — a `truncated:` line naming what was cut and the
/// command that fetches the next page. Table only — every other format is a
/// record stream a reader parses, so a trailing prose line there is
/// corruption, not information.
fn print_trailers(args: &LsArgs, listing: &Listing) {
    let painter = Painter::new(is_color_active());
    let dim = Style::new().effects(Effects::DIMMED);
    let truncated_line = listing.truncated.map(|cut| {
        let next = next_command_hint(args, cut.next_skip());
        format!("truncated: {} of {} — next: {next}", cut.shown, cut.total)
    });
    let lines = ls_rules::trailer(&listing.applied)
        .into_iter()
        .chain(truncated_line);
    for line in lines {
        println!("{}", painter.paint(&line, dim));
    }
}

/// The command that fetches the next page after a truncated listing: this
/// run's own filter and flags, `--skip` set to `next_skip` — this page's
/// `--skip` advanced past what it showed, never restated from an existing
/// `--skip` on the command line, which `next_skip` already accounts for.
/// Built from the parsed [`LsArgs`] rather than raw `argv`, so it is exact
/// regardless of shell aliasing, flag order, or how `@name`/`-w` selected the
/// worktree, and correct whether or not this run itself carried `--skip`.
fn next_command_hint(args: &LsArgs, next_skip: usize) -> String {
    let mut parts = vec!["mt".to_owned(), "ls".to_owned()];
    if let Some(filter) = &args.filter {
        parts.push(shell_quote(filter));
    }
    if args.flags.json {
        parts.push("--json".to_owned());
    } else if !matches!(args.flags.format, OutputFormat::Table) {
        parts.push("--format".to_owned());
        parts.push(format_name(args));
    }
    // `--limit` is always `Some` here: `Truncation` only fires when it was.
    if let Some(limit) = args.flags.limit {
        parts.push("-n".to_owned());
        parts.push(limit.to_string());
    }
    for sort in &args.flags.sort {
        parts.push("--sort".to_owned());
        parts.push(shell_quote(sort));
    }
    if let Some(columns) = &args.flags.columns {
        parts.push("--columns".to_owned());
        parts.push(shell_quote(columns));
    }
    parts.push("--skip".to_owned());
    parts.push(next_skip.to_string());
    parts.join(" ")
}

/// Single-quote `value` for the shell, the way the rest of this reference
/// already writes a filter argument (`mt ls ''`): safe whether or not
/// quoting is needed, so the hint never has to decide. An embedded `'` closes
/// the quote, escapes itself, then reopens it (`'\''`) rather than requiring
/// nothing to escape, which a filter expression's own `"..."` strings do not
/// promise.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// The resolved listing one `mt ls` run prints: the selected tasks, the
/// readiness and attribution they were selected under, the config the render
/// reads (view columns folded in), the environment a column's conditional
/// visibility evaluates against, the stem hashes of every loaded task, and
/// what `--limit` cut — `None` when the listing is complete.
pub(crate) struct Listing {
    pub tasks: Vec<Task>,
    pub readiness: Readiness,
    pub applied: Applied,
    pub config: Config,
    pub env: ConditionEnv,
    pub hashes: StemHashes,
    pub truncated: Option<Truncation>,
}

impl Listing {
    /// What a renderer reads, borrowed from this listing so the
    /// one-shot render and the interactive loop cannot resolve columns
    /// against different facts.
    pub(crate) fn render_ctx(&self) -> RenderCtx<'_> {
        RenderCtx {
            config: &self.config,
            readiness: &self.readiness,
            env: &self.env,
            hashes: &self.hashes,
        }
    }
}

/// Resolve the effective bundle ([`ls_rules::resolve`]), apply it, select and
/// sort the tasks, and truncate. Consumes `project.outcome.tasks`. Readiness
/// is computed over the full set — before the filter narrows it — so a
/// terminal dependency hidden by the effective filter still counts as
/// satisfied; it is shared by the filter, the sort, and the rendered column.
///
/// `--skip` drops rows next, then `--limit` cuts what remains — both after
/// the sort, so together they compose with `--sort` and a view's own sort
/// chain rather than racing them. The match count is taken before either
/// applies, which is what makes the reported `total` the number a bigger
/// `--limit` (or a `--skip` of `0`) would show; `shown` and `next_skip` are
/// read off what actually printed, so paging past the end reports no
/// truncation rather than a next page with nothing on it. Both the one-shot
/// render and the interactive loop build on this, so their lists match
/// exactly. The stem hashes cover every loaded task, not only the rows kept.
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
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let hashes = util::stem_hashes(&project.outcome.tasks, &project.config);
    let filter = Filter::parse(&bundle.filter)?;
    let mut tasks = select(
        project.outcome.tasks,
        Some(&filter),
        &project.config,
        &readiness,
    )?;
    sort_tasks(&mut tasks, &bundle.sort, &project.config, &readiness)?;
    let total = tasks.len();
    let skip = args.flags.skip.unwrap_or(0);
    if skip > 0 {
        tasks.drain(..skip.min(tasks.len()));
    }
    let truncated = Truncation::new(args.flags.limit, skip, total);
    if let Some(limit) = args.flags.limit {
        tasks.truncate(limit);
    }
    Ok(Listing {
        tasks,
        readiness,
        applied,
        config,
        // The same environment `[[ls.rules]]` just resolved against, carried
        // through to the render so a condition means one thing per run
        // whether it picked the bundle or a column's visibility.
        env,
        hashes,
        truncated,
    })
}

/// Apply `specs` (empty means the built-in default order); shared by the
/// `ls` render path and the flip hooks' listings.
pub(crate) fn sort_tasks(
    tasks: &mut [Task],
    specs: &[String],
    config: &Config,
    readiness: &Readiness,
) -> Result<()> {
    if specs.is_empty() {
        // default_order_with honours config.query.default_sort when set, then
        // falls back to the built-in chain.
        default_order_with(tasks, config, readiness);
        return Ok(());
    }
    sort_by_specs(tasks, specs, config, readiness).map_err(|e| anyhow!("{e}"))
}

#[cfg(test)]
#[path = "ls_tests.rs"]
mod tests;
