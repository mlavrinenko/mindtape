//! Rendering tasks as a table, JSON, or CSV.

use anyhow::Result;
use mindtape::config::Config;
use mindtape::numeric::{NumValue, format_num};
use mindtape::{CanonicalTask, Readiness, ReadyState, Task};
use serde_json::{Value, json};

use crate::handle::{DEFAULT_CHECK_LEN, make_handles};
use crate::ls_rules::Applied;
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
}

fn status_name(task: &Task) -> String {
    task.status
        .as_ref()
        .map_or_else(String::new, |status| status.name.clone())
}

/// Render `tasks` in the chosen `format`. `readiness` drives the table's
/// dependency column; `Table` and `CSV` ignore `applied`. `applied` is `mt
/// ls`'s resolved bundle attribution (`crate::ls_rules`): when `Some`, `JSON`
/// wraps the task array in the `{"tasks": ..., "applied": ...}` envelope
/// (`truth/tasks/conditional-ls-rules.typ`); `None` (e.g. `mt eval --json`,
/// which has no rule/view context) keeps the bare array.
///
/// # Errors
/// Returns an error if JSON or CSV serialization fails.
pub fn render(
    tasks: &[Task],
    format: OutputFormat,
    config: &Config,
    readiness: &Readiness,
    applied: Option<&Applied>,
) -> Result<String> {
    match format {
        OutputFormat::Table => Ok(crate::render::table::render(tasks, config, readiness)),
        OutputFormat::Json => render_json(tasks, config, applied),
        OutputFormat::Csv => render_csv(tasks, config, readiness),
    }
}

fn render_json(tasks: &[Task], config: &Config, applied: Option<&Applied>) -> Result<String> {
    let array: Vec<Value> = tasks
        .iter()
        .map(|task| task_to_json(task, config))
        .collect();
    let body = match applied {
        Some(applied) => json!({
            "tasks": array,
            "applied": applied_to_json(applied),
        }),
        None => Value::Array(array),
    };
    Ok(serde_json::to_string_pretty(&body)?)
}

/// `mt ls --json`'s `applied` envelope field: which rule (if any) fired, and
/// each facet's winning source plus effective value.
fn applied_to_json(applied: &Applied) -> Value {
    json!({
        "rule": applied.rule,
        "facets": {
            "filter": {"from": applied.filter_from, "value": applied.filter_value},
            "sort": {"from": applied.sort_from, "value": applied.sort_value},
            "columns": {"from": applied.columns_from, "value": applied.columns_value},
        },
    })
}

/// Render CSV over the same resolved column set/order the table renderer
/// uses ([`resolve_order`]: `[ls] columns-order` + per-column `Visibility`
/// against [`mindtape::LS_COLUMNS`]), so `--format csv` and the default table
/// never disagree on which columns appear or in what sequence.
///
/// Column *mapping decision*: CSV column values are the plain/raw field
/// (e.g. the status legend's raw name, the numeric priority score) rather
/// than the table's coloured/labelled cell text — matching CSV's prior
/// behaviour and staying friendly to non-interactive consumers. The one
/// deliberate divergence is the `#` column: the table's handle
/// (`<ordinal><check>`) is ephemeral per render, so CSV emits the task's
/// stable `id` under an `id` header instead.
///
/// Fields with no [`mindtape::LS_COLUMNS`] entry — `kind`, `estimate`, `tags`,
/// derived tags, and any `x-<ns>` extra — have no column to drive from and so
/// are not represented here, same as the table. `--format json` is the
/// escape hatch: it always emits every field.
fn render_csv(tasks: &[Task], config: &Config, readiness: &Readiness) -> Result<String> {
    let handles = make_handles(tasks, DEFAULT_CHECK_LEN);
    let columns = resolve_order(config, &handles, readiness);
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(columns.iter().copied().map(csv_header))?;
    for (_, task) in &handles {
        let row: Vec<String> = columns
            .iter()
            .map(|&id| csv_value(id, task, config, readiness))
            .collect();
        writer.write_record(row)?;
    }
    let bytes = writer.into_inner()?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// CSV header text for a resolved column. Diverges from [`ColumnId::header`]
/// only for `Handle`: `id` describes the CSV cell's content (the task's
/// stable id), where the table's `#` describes an ephemeral ordinal.
fn csv_header(id: ColumnId) -> &'static str {
    match id {
        ColumnId::Handle => "id",
        ColumnId::Status => "status",
        ColumnId::Ready => "ready",
        ColumnId::Priority => "priority",
        ColumnId::Due => "due",
        ColumnId::Title => "title",
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
        ColumnId::Title => task.title.clone(),
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
