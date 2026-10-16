//! `mt ls` — query tasks under the configured roots.
//!
//! # Files
//!
//! - `rules.rs` — `[[ls.rules]]` cascade resolution for a bare `mt ls`.
//! - `rules_applied.rs` — `Applied`, which source won each facet.
//! - `interactive/` — the `--interactive` terminal picker.

pub(crate) mod interactive;
pub(crate) mod rules;
pub(crate) mod rules_applied;

use crate::diag::Result;
use anstyle::{Effects, Style};

use crate::diag::{Diag, Hint, Kind};
use mindtape::sort::{default_order_with, sort_by_specs};
use mindtape::{ConditionEnv, Config, Filter, Readiness, StemHashes, Task, select};

use self::rules::Applied;
use crate::render::RenderCtx;
use crate::render::color::{Painter, is_color_active};
use crate::render::output::{Envelope, OutputFormat, Truncation, render};
use crate::util;

/// Arguments for `ls`.
#[derive(Debug, clap::Args)]
pub struct LsArgs {
    /// Which tasks to list: a filter expression, or `@view` for a saved view.
    ///
    /// Without it, you get your project's default listing. `''` lists every
    /// task.
    pub filter: Option<String>,

    #[command(flatten)]
    pub flags: LsFlags,
}

/// Output-shaping flags shared verbatim between `ls` and `find`
/// (`crate::cmd::find`, sugar over `contains()`): everything but the filter
/// itself. Flattened into both so a flag can never exist on one and not the
/// other — the drift `crate::model::fields` was created to end after `filter/mod.rs` and
/// `sort.rs` disagreed about which fields exist (see
/// `truth/tasks/mt-find-as-sugar-over-contains.typ`).
#[derive(Debug, Clone, clap::Args)]
pub struct LsFlags {
    /// Output format.
    #[arg(long, value_enum, default_value_t)]
    pub format: OutputFormat,

    /// Same as `--format json`.
    #[arg(long)]
    pub json: bool,

    /// Show at most N tasks, after filtering and sorting.
    ///
    /// A table says how many it left out.
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    /// Skip the first N rows after sorting.
    ///
    /// For page two of `-n 20`, use `--skip 20`. When the table is cut off,
    /// its footer shows the `--skip` for the next page.
    #[arg(long)]
    pub skip: Option<usize>,

    /// Sort by a field, such as `--sort due:asc`.
    ///
    /// Repeat it to break ties, or pass `@view` to sort as that view does.
    #[arg(long)]
    pub sort: Vec<String>,

    /// The columns to show, comma-separated, or `@view` for a view's columns.
    #[arg(long, value_name = "COLS")]
    pub columns: Option<String>,

    /// Pick a task from the table and open it in your editor.
    ///
    /// Type a task's hash to highlight it, and press Enter to open it. Needs a
    /// terminal.

    #[arg(short = 'i', long, conflicts_with = "json")]
    pub interactive: bool,
}

/// Run the `ls` command.
///
/// # Errors
/// Returns an error if the project cannot be opened or the filter/sort is invalid.
pub fn run(args: &LsArgs) -> Result<()> {
    run_with(args, util::open_warned()?)
}

pub(crate) fn run_with(args: &LsArgs, project: util::Project) -> Result<()> {
    if args.flags.interactive {
        return crate::cmd::ls::interactive::run(args, project);
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
    print_body(&body, format);
    if matches!(format, OutputFormat::Table) {
        print_footer(args, &listing);
    }
    Ok(())
}

/// Print a rendered listing with the newline its format calls for.
fn print_body(body: &str, format: OutputFormat) {
    // An empty `path` listing is no bytes, not a lone newline, so `wc -l`
    // counts 0 and a `while read` loop never runs. `path0` is self-terminated:
    // every path (including the last) already carries its own trailing NUL, so
    // a `println!`-supplied newline would land inside the final `xargs -0`
    // argument instead of after it.
    if body.is_empty() {
        return;
    }
    if matches!(format, OutputFormat::Path0) {
        print!("{body}");
    } else {
        println!("{body}");
    }
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

/// The dim lines under a rendered table: where the listing came from and how
/// many of the project's tasks it shows, then, when `--limit` hid rows, how
/// many more matched and the command that fetches the next page. Table only:
/// every other format is a record stream a reader parses, so a trailing prose
/// line there is corruption, not information.
fn print_footer(args: &LsArgs, listing: &Listing) {
    let painter = Painter::new(is_color_active());
    let dim = Style::new().effects(Effects::DIMMED);
    let next_page = listing.truncated.map(|cut| {
        let next = next_command_hint(args, cut.next_skip());
        format!("{} · next page: {next}", cut.footer())
    });
    let footer = rules::footer(&listing.applied, listing.tasks.len(), listing.project_total);
    let lines = std::iter::once(footer).chain(next_page);
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
/// visibility evaluates against, the stem hashes of every loaded task,
/// what `--limit` cut (`None` when the listing is complete), and how many
/// tasks the project holds before any filter.
pub(crate) struct Listing {
    pub tasks: Vec<Task>,
    pub readiness: Readiness,
    pub applied: Applied,
    pub config: Config,
    pub env: ConditionEnv,
    pub hashes: StemHashes,
    pub truncated: Option<Truncation>,
    pub project_total: usize,
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

/// Resolve the effective bundle ([`cmd::ls::rules::resolve`]), apply it, select and
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
    let env = rules::build_env(&project, &format_name(args));
    let (bundle, applied) = rules::resolve(&project, args, &env)?;
    let mut config = project.config.clone();
    if !bundle.columns.is_empty() {
        config.ls.columns_order = bundle.columns.clone();
    }
    let readiness = Readiness::compute(&project.outcome.tasks, &project.config);
    let project_total = project.outcome.tasks.len();
    let hashes = util::stem_hashes(&project.outcome.tasks, &project.config);
    // The command line's filter is a usage error when it fails; a view's or
    // a rule's stays as the config reports it.
    let typed = |err| match &args.filter {
        Some(_) => crate::diag::filter(err, Some(&bundle.filter)),
        None => Diag::from(err),
    };
    let filter = Filter::parse(&bundle.filter).map_err(typed)?;
    let tasks = select(
        project.outcome.tasks,
        Some(&filter),
        &project.config,
        &readiness,
    );
    let mut tasks = tasks.map_err(typed)?;
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
        project_total,
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
    for spec in specs {
        check_sort_spec(spec)?;
    }
    sort_by_specs(tasks, specs, config, readiness).map_err(|msg| Diag::verbatim(Kind::Usage, msg))
}

/// `spec` (`field`, `field:asc` or `field:desc`) as a usage error naming the
/// part that is wrong, with the nearest field for a misspelt one.
fn check_sort_spec(spec: &str) -> std::result::Result<(), Diag> {
    let (field, direction) = spec
        .split_once(':')
        .map_or((spec, None), |(field, direction)| (field, Some(direction)));
    if !mindtape::sort::is_sort_field(field) {
        let diag = Diag::usage("unknown sort field").value(field);
        let known = mindtape::sort::sort_fields();
        return Err(diag.did_you_mean_or(
            field,
            &known,
            Hint::new("sort by a task field such as priority or due"),
        ));
    }
    match direction {
        None | Some("asc" | "desc") => Ok(()),
        Some(other) => Err(Diag::usage("unknown sort direction")
            .value(other)
            .hint(Hint::new("use asc or desc"))),
    }
}

#[cfg(test)]
mod tests;
