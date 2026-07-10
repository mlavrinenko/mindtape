//! Rendering tasks as a table, JSON, or CSV.

use anyhow::Result;
use mindtape::config::Config;
use mindtape::link::Link;
use mindtape::numeric::{NumValue, format_num};
use mindtape::{Readiness, ReadyState, Task};
use serde_json::{Map, Value, json};
use typst_harvest::HVal;

use crate::handle::{DEFAULT_CHECK_LEN, make_handles};
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
/// dependency column; `JSON` and `CSV` ignore it.
///
/// # Errors
/// Returns an error if JSON or CSV serialization fails.
pub fn render(
    tasks: &[Task],
    format: OutputFormat,
    config: &Config,
    readiness: &Readiness,
) -> Result<String> {
    match format {
        OutputFormat::Table => Ok(crate::render::table::render(tasks, config, readiness)),
        OutputFormat::Json => render_json(tasks, config),
        OutputFormat::Csv => render_csv(tasks, config, readiness),
    }
}

fn render_json(tasks: &[Task], config: &Config) -> Result<String> {
    let array: Vec<Value> = tasks
        .iter()
        .map(|task| task_to_json(task, config))
        .collect();
    Ok(serde_json::to_string_pretty(&Value::Array(array))?)
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

/// Project one task to a JSON object. `config` resolves framework-scored
/// priorities to their numeric score.
pub fn task_to_json(task: &Task, config: &Config) -> Value {
    let mut map = Map::new();
    map.insert("id".to_owned(), json!(task.id));
    map.insert("file".to_owned(), json!(task.file.display().to_string()));
    map.insert("title".to_owned(), json!(task.title));
    map.insert("status".to_owned(), status_to_json(task));
    map.insert(
        "priority".to_owned(),
        priority_to_json(task.priority.as_ref(), config),
    );
    map.insert(
        "difficulty".to_owned(),
        num_to_json(task.difficulty.as_ref()),
    );
    map.insert("kind".to_owned(), opt(task.kind.as_deref()));
    map.insert("estimate".to_owned(), json!(task.estimate));
    map.insert(
        "links".to_owned(),
        Value::Array(task.links.iter().map(link_to_json).collect()),
    );
    map.insert("tags".to_owned(), json!(task.tags));
    map.insert("derived_tags".to_owned(), json!(task.derived_tags));
    map.insert("due".to_owned(), opt(task.due.as_deref()));
    map.insert("start".to_owned(), opt(task.start.as_deref()));
    map.insert("reviewed".to_owned(), opt(task.reviewed.as_deref()));
    let extras = task
        .extras
        .iter()
        .map(|(key, value)| (key.clone(), hval_to_json(value)))
        .collect();
    map.insert("extras".to_owned(), Value::Object(extras));
    Value::Object(map)
}

/// One `links:` edge as `{task, kind, note}` — the JSON shape for `mt ls
/// --format json` / `mt eval --json`.
fn link_to_json(link: &Link) -> Value {
    json!({
        "task": link.task,
        "kind": link.kind,
        "note": link.note,
    })
}

fn status_to_json(task: &Task) -> Value {
    match &task.status {
        None => Value::Null,
        Some(status) => json!({
            "name": status.name,
            "date": status.date,
            "note": status.note,
        }),
    }
}

fn opt(value: Option<&str>) -> Value {
    value.map_or(Value::Null, |text| json!(text))
}

/// JSON for a numeric-field value: a number stays numeric, a label is a string.
/// A framework value is unreachable here (priority routes through
/// [`priority_to_json`]); it falls back to its name.
fn num_to_json(value: Option<&NumValue>) -> Value {
    match value {
        Some(NumValue::Num(num)) => json!(num),
        Some(NumValue::Label(label)) => json!(label),
        Some(framework @ NumValue::Framework { .. }) => json!(framework.as_display()),
        None => Value::Null,
    }
}

/// JSON for a priority value: a framework value resolves to its numeric score
/// (null when unresolvable, e.g. outside a project); else as a numeric field.
fn priority_to_json(value: Option<&NumValue>, config: &Config) -> Value {
    match value {
        Some(framework @ NumValue::Framework { .. }) => config
            .tasks
            .priority
            .resolve(framework)
            .map_or(Value::Null, |score| json!(score)),
        other => num_to_json(other),
    }
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

fn hval_to_json(value: &HVal) -> Value {
    match value {
        HVal::Null => Value::Null,
        HVal::Bool(flag) => json!(flag),
        HVal::Int(num) => json!(num),
        HVal::Float(num) => json!(num),
        HVal::Str(text) | HVal::Date(text) => json!(text),
        HVal::Array(items) => Value::Array(items.iter().map(hval_to_json).collect()),
        HVal::Dict(map) => Value::Object(
            map.iter()
                .map(|(key, val)| (key.clone(), hval_to_json(val)))
                .collect(),
        ),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use std::collections::BTreeMap;

    use mindtape::numeric::NumValue;
    use mindtape::test_support::bare_task;
    use mindtape::{Status, Task};

    use mindtape::{Config, Readiness};

    use super::{HVal, OutputFormat, render, task_to_json};

    fn sample() -> Task {
        let mut t = bare_task("t1");
        t.title = "Build".to_owned();
        t.status = Some(Status {
            name: "wip".to_owned(),
            date: Some("2026-06-14".to_owned()),
            note: None,
        });
        t.priority = Some(NumValue::Num(8.0));
        t.estimate = BTreeMap::from([("hours".to_owned(), 8.0)]);
        t.tags = vec!["x".to_owned()];
        // Populated so `due` stays significant under the default `auto`
        // column visibility — several csv tests below assert on its presence.
        t.due = Some("2026-07-01".to_owned());
        t
    }

    #[test]
    fn table_contains_fields() {
        let out = render(
            &[sample()],
            OutputFormat::Table,
            &Config::default(),
            &Readiness::empty(),
        )
        .unwrap();
        assert!(out.contains("Build"));
        assert!(out.contains("wip"));
    }

    #[test]
    fn table_has_handle_column() {
        let out = render(
            &[sample()],
            OutputFormat::Table,
            &Config::default(),
            &Readiness::empty(),
        )
        .unwrap();
        // The header row must have "#" and the first task is ordinal 1.
        assert!(out.contains('#'));
        assert!(out.contains("1"));
    }

    #[test]
    fn json_does_not_contain_handle() {
        let out = render(
            &[sample()],
            OutputFormat::Json,
            &Config::default(),
            &Readiness::empty(),
        )
        .unwrap();
        assert!(
            !out.contains("\"handle\""),
            "handle must not appear in JSON"
        );
        // id is still present
        assert!(out.contains("\"id\""));
    }

    #[test]
    fn json_exposes_core_fields() {
        let out = render(
            &[sample()],
            OutputFormat::Json,
            &Config::default(),
            &Readiness::empty(),
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value[0]["id"], "t1");
        assert_eq!(value[0]["status"]["name"], "wip");
        assert_eq!(value[0]["estimate"]["hours"], 8.0);
    }

    #[test]
    fn csv_has_header_and_row() {
        let out = render(
            &[sample()],
            OutputFormat::Csv,
            &Config::default(),
            &Readiness::empty(),
        )
        .unwrap();
        assert!(out.contains("id,status,priority,due,title"));
        assert!(out.contains("t1,wip,8,2026-07-01,Build"));
    }

    #[test]
    fn csv_columns_order_selects_and_orders() {
        // A custom `[ls] columns-order` must drive CSV exactly like the table
        // renderer: only the listed columns, in the given sequence.
        let mut config = Config::default();
        config.ls.columns_order = vec!["title".to_owned(), "status".to_owned()];
        let out = render(&[sample()], OutputFormat::Csv, &config, &Readiness::empty()).unwrap();
        let mut lines = out.lines();
        assert_eq!(lines.next().unwrap(), "title,status");
        assert_eq!(lines.next().unwrap(), "Build,wip");
    }

    #[test]
    fn csv_includes_ready_when_blocked() {
        // `ready` is one of `LS_COLUMNS`: like the table, CSV must surface it
        // once a task is contested, not just the historical fixed 5 columns.
        let mut blocked = sample();
        blocked.id = "t2".to_owned();
        blocked.file = "tasks/t2.typ".into();
        blocked.links = vec![mindtape::test_support::dep_edge("missing.typ")];
        let tasks = vec![sample(), blocked];
        let readiness = Readiness::compute(&tasks, &Config::default());
        let out = render(&tasks, OutputFormat::Csv, &Config::default(), &readiness).unwrap();
        assert_eq!(
            out.lines().next().unwrap(),
            "id,status,ready,priority,due,title"
        );
        assert!(out.contains("t2,wip,blocked,8,2026-07-01,Build"));
    }

    #[test]
    fn csv_omits_fields_without_ls_column() {
        // `kind`, `estimate`, `tags`, and `x-<ns>` extras have no `LS_COLUMNS`
        // entry, so — like the table — CSV does not represent them; only
        // `--format json` exposes every field. This pins the documented
        // limitation from `render_csv`'s doc comment as a regression guard.
        let mut task = sample();
        task.kind = Some("bug-report".to_owned());
        let out = render(
            &[task],
            OutputFormat::Csv,
            &Config::default(),
            &Readiness::empty(),
        )
        .unwrap();
        assert_eq!(out.lines().next().unwrap(), "id,status,priority,due,title");
        assert!(
            !out.contains("bug-report"),
            "kind has no ls column and must not leak into csv"
        );
    }

    #[test]
    fn extras_arrays_and_dicts_serialize() {
        let mut task = sample();
        task.extras.insert(
            "x-ns".to_owned(),
            HVal::Array(vec![HVal::Int(1), HVal::Str("a".to_owned())]),
        );
        let json = task_to_json(&task, &Config::default());
        assert_eq!(json["extras"]["x-ns"][0], 1);
        assert_eq!(json["extras"]["x-ns"][1], "a");
    }

    #[test]
    fn json_exposes_links_as_task_kind_note() {
        let mut task = sample();
        task.links = vec![mindtape::test_support::noted_edge(
            "epic.typ", "related", "why",
        )];
        let json = task_to_json(&task, &Config::default());
        assert_eq!(json["links"][0]["task"], "epic.typ");
        assert_eq!(json["links"][0]["kind"], "related");
        assert_eq!(json["links"][0]["note"], "why");
    }
}
