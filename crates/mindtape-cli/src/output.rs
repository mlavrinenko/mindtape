//! Rendering tasks as a table, JSON, or CSV.

use anyhow::Result;
use mindtape::config::Config;
use mindtape::numeric::{NumValue, format_num};
use mindtape::{CanonicalTask, Readiness, ReadyState, SCHEMA_VERSION, Task};
use serde_json::{Value, json};

use crate::handle::{check_len, make_handles};
use crate::ls_rules::Applied;
use crate::render::RenderCtx;
use crate::render::columns::{ColumnId, resolve_order};

/// Output format for `list` and `eval`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Aligned text table.
    #[default]
    Table,
    /// Pretty-printed JSON array.
    Json,
    /// Comma-separated values with a header row.
    Csv,
    /// One task file path per line, nothing else — for composing with
    /// another program (`rg TODO $(mt ls ready --format path)`).
    #[value(name = "path")]
    Path,
    /// The same list as `path`, NUL-delimited instead of newline-delimited,
    /// for `xargs -0` and paths holding whitespace.
    #[value(name = "path0")]
    Path0,
}

/// What `--limit` cut from a listing: the rows the caller was shown against
/// the rows the filter actually matched. Only ever constructed when the limit
/// bit, so `shown < total` holds by construction and a consumer never
/// re-derives whether anything was hidden — absent *is* "this listing is
/// complete" (`truth/tasks/ls-limit-flag.typ`). `skip` is the `--skip` this
/// run applied before `-n` (`0` when the flag was absent), carried along so
/// [`Self::next_skip`] can advance it rather than restate it
/// (`truth/tasks/add-skip-and-a-next-command-hint-to-ls-paging.typ`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Truncation {
    pub shown: usize,
    pub total: usize,
    pub skip: usize,
}

impl Truncation {
    /// The truncation `--limit`/`--skip` produced: `total` matched tasks,
    /// `skip` dropped before the cut, `limit` shown after it. `None` when no
    /// limit was given — `--skip` alone shows every remaining row, so it
    /// never truncates on its own — and `None` when this page reaches the
    /// end of the match set (`skip + shown >= total`), a listing that fits
    /// under its own cap is not truncated, and saying so would train the
    /// reader to skip the line that means something.
    #[must_use]
    pub fn new(limit: Option<usize>, skip: usize, total: usize) -> Option<Self> {
        let limit = limit?;
        let shown = limit.min(total.saturating_sub(skip));
        (skip + shown < total).then_some(Self { shown, total, skip })
    }

    /// The `--skip` value that continues from here: this page's `--skip`
    /// advanced past the rows it actually showed, not past `total` — so a
    /// caller that runs it next sees the next page, not a gap or a repeat.
    #[must_use]
    pub fn next_skip(&self) -> usize {
        self.skip + self.shown
    }

    /// The human footer line for a truncated table, labelled like
    /// [`crate::ls_rules::trailer`]'s `applied:` so the two read as one block
    /// under the rows. Plain — no `next:` command; that needs the parsed
    /// `LsArgs` this type does not carry, so `crate::ls::print_trailers`
    /// composes the full line for the one caller that has them.
    #[must_use]
    pub fn footer(self) -> String {
        format!("truncated: showing {} of {}", self.shown, self.total)
    }
}

/// The document-level envelope `mt ls --json` wraps its records in: which rule
/// and view won each facet ([`Applied`]), and what `--limit` cut
/// ([`Truncation`]). `mt eval --json` and the flip-hook table have neither
/// context and pass `None` to [`render`], keeping the bare array —
/// `schema_version` is an envelope marker, not a per-task field.
#[derive(Debug, Clone, Copy)]
pub struct Envelope<'a> {
    pub applied: &'a Applied,
    pub truncated: Option<Truncation>,
}

fn status_name(task: &Task) -> String {
    task.status
        .as_ref()
        .map_or_else(String::new, |status| status.name.clone())
}

/// Render `tasks` in the chosen `format`. `ctx` carries the config, the
/// readiness driving the table's dependency column, and the environment a
/// column's conditional visibility evaluates against; only `JSON` reads
/// `envelope` at all. When it is `Some`,
/// `JSON` wraps the task array in the `{"schema_version": ..., "tasks": ...,
/// "applied": ..., "truncated": ...}` envelope
/// (`truth/tasks/conditional-ls-rules.typ`,
/// `truth/tasks/emit-schema-version-in-the-ls-json-envelope.typ`,
/// `truth/tasks/ls-limit-flag.typ`); `None` (e.g. `mt eval --json`, which has
/// no rule/view context) keeps the bare array — `schema_version` is a
/// document-level envelope marker, not a per-task field, so a bare array
/// carries none.
///
/// `Table`, `CSV`, `Path` and `Path0` ignore `envelope`: the table's
/// attribution and truncation lines are printed under it by the caller
/// (`crate::ls`), and the three machine formats are record streams with
/// nowhere to put a document-level fact — no header, no colour, no
/// hyperlinks, no envelope, so a `csv` reader and `xargs -0` get records and
/// nothing else (`truth/tasks/emit-a-plain-path-list-from-ls.typ`).
///
/// # Errors
/// Returns an error if JSON or CSV serialization fails, or if a column's
/// `rules` condition fails to evaluate (`table` and `csv` only — the record
/// formats resolve no columns).
pub fn render(
    tasks: &[Task],
    format: OutputFormat,
    ctx: RenderCtx<'_>,
    envelope: Option<Envelope<'_>>,
) -> Result<String> {
    match format {
        OutputFormat::Table => crate::render::table::render(tasks, ctx),
        OutputFormat::Json => render_json(tasks, ctx.config, ctx.readiness, envelope),
        OutputFormat::Csv => render_csv(tasks, ctx),
        OutputFormat::Path => Ok(render_path(tasks)),
        OutputFormat::Path0 => Ok(render_path0(tasks)),
    }
}

/// `path` format: one task path per line, joined (not trailing) — the caller
/// prints this through `println!`, which supplies the final newline the same
/// way every other format already relies on it.
fn render_path(tasks: &[Task]) -> String {
    tasks
        .iter()
        .map(|task| crate::render::file_path::cell_text(&task.file))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `path0` format: each task path is its own NUL-terminated record, including
/// the last — unlike [`render_path`], the caller must print this verbatim
/// (`print!`, not `println!`), since a stray trailing newline after the final
/// NUL would land inside the last `xargs -0` argument.
fn render_path0(tasks: &[Task]) -> String {
    tasks
        .iter()
        .map(|task| format!("{}\0", crate::render::file_path::cell_text(&task.file)))
        .collect()
}

fn render_json(
    tasks: &[Task],
    config: &Config,
    readiness: &Readiness,
    envelope: Option<Envelope<'_>>,
) -> Result<String> {
    let array: Vec<Value> = tasks
        .iter()
        .map(|task| {
            let mut canonical = CanonicalTask::from_task(task, config);
            canonical.why = readiness.blockers(task).to_vec();
            serde_json::to_value(canonical).unwrap_or(Value::Null)
        })
        .collect();
    let body = match envelope {
        Some(envelope) => json!({
            "schema_version": SCHEMA_VERSION,
            "tasks": array,
            "applied": applied_to_json(envelope.applied),
            "truncated": truncated_to_json(envelope.truncated),
        }),
        None => Value::Array(array),
    };
    Ok(serde_json::to_string_pretty(&body)?)
}

/// `mt ls --json`'s `truncated` envelope field: `null` on a complete listing,
/// else `shown`/`total` plus `next_skip` — the `--skip` value that continues
/// from here, so a caller pages forward by copying a field rather than adding
/// `shown` to whatever `--skip` it already passed. Always present, so a
/// consumer reads one key rather than testing for its existence — an
/// additive envelope key no `mt import` or `jq` consumer can trip over, hence
/// no `SCHEMA_VERSION` bump.
fn truncated_to_json(truncated: Option<Truncation>) -> Value {
    truncated.map_or(
        Value::Null,
        |cut| json!({"shown": cut.shown, "total": cut.total, "next_skip": cut.next_skip()}),
    )
}

/// `mt ls --json`'s `applied` envelope field: which rule (if any) fired,
/// whether anything of it survived, and each facet's winning source plus
/// effective value.
///
/// `superseded` is how a consumer tells a rule that matched and lost every
/// facet from one that never matched — both leave the facets pointing
/// elsewhere, and only the flag separates them. Additive, so no
/// `SCHEMA_VERSION` bump.
fn applied_to_json(applied: &Applied) -> Value {
    json!({
        "rule": applied.rule,
        "superseded": applied.rule_superseded,
        "facets": {
            "filter": {"from": applied.filter_from, "value": applied.filter_value},
            "sort": {"from": applied.sort_from, "value": applied.sort_value},
            "columns": {"from": applied.columns_from, "value": applied.columns_value},
        },
    })
}

/// Render CSV over the same resolved column set/order the table renderer
/// uses ([`resolve_order`]: `[ls] columns-order` + each column's `when`
/// against [`mindtape::LS_COLUMNS`]), so `--format csv` and the default table
/// never disagree on which columns appear or in what sequence.
///
/// Column *mapping decision*: CSV column values are the plain/raw field
/// (e.g. the status legend's raw name, the numeric priority score) rather
/// than the table's coloured/labelled cell text — matching CSV's prior
/// behaviour and staying friendly to non-interactive consumers. Two columns
/// diverge from the table on purpose, and for one reason: where the table
/// shows something only the render that produced it can resolve, CSV emits
/// the stable value a program can keep.
///
/// - `#`: the table's handle (`<ordinal><check>`) is ephemeral per render, so
///   CSV emits the task's stable `id` under an `id` header instead.
/// - `file`: the table's cell is relative to the process's working directory
///   ([`crate::render::file_path::cell_text`]), so one task reads
///   `tasks/a.typ` from the root and `../tasks/a.typ` from `tasks/`. CSV
///   emits `task.file` — the project-root-relative path, byte-identical to
///   `--format json`'s `file` key and to what the `path` filter field
///   compares, so the two machine formats never disagree by cwd
///   (`truth/tasks/csv-and-json-disagree-about-what-the-file-field-means.typ`).
///   Rendering that path from where the reader stands is `--format path`'s
///   whole job, and the table's.
///
/// Fields with no [`mindtape::LS_COLUMNS`] entry — `kind`, `estimate`, `tags`,
/// derived tags, and any `x-<ns>` extra — have no column to drive from and so
/// are not represented here, same as the table. `--format json` is the
/// escape hatch: it always emits every field.
fn render_csv(tasks: &[Task], ctx: RenderCtx<'_>) -> Result<String> {
    let handles = make_handles(tasks, check_len(ctx.config));
    let columns = resolve_order(&handles, ctx)?;
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(columns.iter().copied().map(csv_header))?;
    for (_, task) in &handles {
        let row: Vec<String> = columns
            .iter()
            .map(|&id| csv_value(id, task, ctx.config, ctx.readiness))
            .collect();
        writer.write_record(row)?;
    }
    let bytes = writer.into_inner()?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// CSV header text for a resolved column: its [`ColumnId::key`], except
/// for `Handle`: `id` describes the CSV cell's content (the task's
/// stable id), where the table's `#` describes an ephemeral ordinal.
fn csv_header(id: ColumnId) -> &'static str {
    match id {
        ColumnId::Handle => "id",
        other => other.key(),
    }
}

/// CSV cell text for a resolved column: the plain/raw field value (see the
/// mapping decision on [`render_csv`]), not the table's styled [`CellSpec`].
///
/// [`CellSpec`]: crate::render::field::CellSpec
fn csv_value(id: ColumnId, task: &Task, config: &Config, readiness: &Readiness) -> String {
    match id {
        ColumnId::Handle => task.id.clone(),
        ColumnId::Status => status_name(task),
        ColumnId::Ready => ready_text(readiness.state(task)).to_owned(),
        ColumnId::Priority => priority_display(task.priority.as_ref(), config),
        ColumnId::Due => task.due.clone().unwrap_or_default(),
        ColumnId::Assignee => task.assignee.clone().unwrap_or_default(),
        ColumnId::Title => task.title.clone(),
        ColumnId::Why => mindtape::waits_on(readiness.blockers(task)),
        ColumnId::Children => task.children.to_string(),
        ColumnId::Progress => task.progress().map(|p| p.to_string()).unwrap_or_default(),
        ColumnId::File => task.file.display().to_string(),
    }
}

/// Plain readiness text for a CSV cell: settled tasks render blank, matching
/// the table's `ready` column significance (hidden unless something is
/// blocked).
fn ready_text(state: ReadyState) -> &'static str {
    match state {
        ReadyState::Ready => "ready",
        ReadyState::Blocked => "blocked",
        ReadyState::Settled => "",
    }
}

/// Project one task to a JSON object via the canonical schema
/// ([`CanonicalTask`]) — the single shape shared with `mt import`. `config`
/// resolves a framework priority's read-side score.
pub fn task_to_json(task: &Task, config: &Config) -> Value {
    let canonical = CanonicalTask::from_task(task, config);
    serde_json::to_value(canonical).unwrap_or(Value::Null)
}

/// The plain display string for a priority value: a framework value resolves to
/// its numeric score; else the numeric-field display (empty when absent).
fn priority_display(value: Option<&NumValue>, config: &Config) -> String {
    match value {
        Some(framework @ NumValue::Framework { .. }) => config
            .tasks
            .priority
            .resolve(framework)
            .map(format_num)
            .unwrap_or_default(),
        other => other.map(NumValue::as_display).unwrap_or_default(),
    }
}

#[cfg(test)]
#[path = "output_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "output_tests_envelope.rs"]
mod tests_envelope;
