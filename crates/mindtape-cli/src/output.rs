//! Rendering tasks as a table, JSON, or CSV.

use anyhow::Result;
use mindtape::Task;
use mindtape::config::Config;
use mindtape::numeric::NumValue;
use serde_json::{Map, Value, json};
use typst_harvest::HVal;

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

/// Render `tasks` in the chosen `format`.
///
/// # Errors
/// Returns an error if JSON or CSV serialization fails.
pub fn render(tasks: &[Task], format: OutputFormat, config: &Config) -> Result<String> {
    match format {
        OutputFormat::Table => Ok(crate::render::table::render(tasks, config)),
        OutputFormat::Json => render_json(tasks),
        OutputFormat::Csv => render_csv(tasks),
    }
}

fn render_json(tasks: &[Task]) -> Result<String> {
    let array: Vec<Value> = tasks.iter().map(task_to_json).collect();
    Ok(serde_json::to_string_pretty(&Value::Array(array))?)
}

fn render_csv(tasks: &[Task]) -> Result<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(["id", "status", "priority", "due", "title"])?;
    for task in tasks {
        writer.write_record([
            task.id.as_str(),
            &status_name(task),
            &num_display(task.priority.as_ref()),
            task.due.as_deref().unwrap_or_default(),
            task.title.as_str(),
        ])?;
    }
    let bytes = writer.into_inner()?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Project one task to a JSON object.
pub fn task_to_json(task: &Task) -> Value {
    let mut map = Map::new();
    map.insert("id".to_owned(), json!(task.id));
    map.insert("file".to_owned(), json!(task.file.display().to_string()));
    map.insert("title".to_owned(), json!(task.title));
    map.insert("status".to_owned(), status_to_json(task));
    map.insert("priority".to_owned(), num_to_json(task.priority.as_ref()));
    map.insert(
        "difficulty".to_owned(),
        num_to_json(task.difficulty.as_ref()),
    );
    map.insert("kind".to_owned(), opt(task.kind.as_deref()));
    map.insert("estimate".to_owned(), json!(task.estimate));
    map.insert("depends_on".to_owned(), json!(task.depends_on));
    map.insert("relates_to".to_owned(), json!(task.relates_to));
    map.insert("supersedes".to_owned(), json!(task.supersedes));
    map.insert("duplicates".to_owned(), json!(task.duplicates));
    map.insert("tags".to_owned(), json!(task.tags));
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
fn num_to_json(value: Option<&NumValue>) -> Value {
    match value {
        Some(NumValue::Num(num)) => json!(num),
        Some(NumValue::Label(label)) => json!(label),
        None => Value::Null,
    }
}

/// The plain display string for a numeric-field value (empty when absent).
fn num_display(value: Option<&NumValue>) -> String {
    value.map(NumValue::as_display).unwrap_or_default()
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
    use mindtape::{Status, Task};

    use mindtape::Config;

    use super::{HVal, OutputFormat, render, task_to_json};

    fn sample() -> Task {
        Task {
            id: "t1".to_owned(),
            file: "tasks/t1.typ".into(),
            title: "Build".to_owned(),
            status: Some(Status {
                name: "wip".to_owned(),
                date: Some("2026-06-14".to_owned()),
                note: None,
            }),
            priority: Some(NumValue::Num(8.0)),
            difficulty: None,
            kind: None,
            estimate: BTreeMap::from([("hours".to_owned(), 8.0)]),
            depends_on: Vec::new(),
            relates_to: Vec::new(),
            supersedes: Vec::new(),
            duplicates: Vec::new(),
            tags: vec!["x".to_owned()],
            due: None,
            start: None,
            reviewed: None,
            extras: BTreeMap::new(),
        }
    }

    #[test]
    fn table_contains_fields() {
        let out = render(&[sample()], OutputFormat::Table, &Config::default()).unwrap();
        assert!(out.contains("Build"));
        assert!(out.contains("wip"));
    }

    #[test]
    fn table_has_handle_column() {
        let out = render(&[sample()], OutputFormat::Table, &Config::default()).unwrap();
        // The header row must have "#" and the first task is ordinal 1.
        assert!(out.contains('#'));
        assert!(out.contains("1"));
    }

    #[test]
    fn json_does_not_contain_handle() {
        let out = render(&[sample()], OutputFormat::Json, &Config::default()).unwrap();
        assert!(
            !out.contains("\"handle\""),
            "handle must not appear in JSON"
        );
        // id is still present
        assert!(out.contains("\"id\""));
    }

    #[test]
    fn json_exposes_core_fields() {
        let out = render(&[sample()], OutputFormat::Json, &Config::default()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value[0]["id"], "t1");
        assert_eq!(value[0]["status"]["name"], "wip");
        assert_eq!(value[0]["estimate"]["hours"], 8.0);
    }

    #[test]
    fn csv_has_header_and_row() {
        let out = render(&[sample()], OutputFormat::Csv, &Config::default()).unwrap();
        assert!(out.contains("id,status,priority,due,title"));
        assert!(out.contains("t1,wip,8,,Build"));
    }

    #[test]
    fn extras_arrays_and_dicts_serialize() {
        let mut task = sample();
        task.extras.insert(
            "x-ns".to_owned(),
            HVal::Array(vec![HVal::Int(1), HVal::Str("a".to_owned())]),
        );
        let json = task_to_json(&task);
        assert_eq!(json["extras"]["x-ns"][0], 1);
        assert_eq!(json["extras"]["x-ns"][1], "a");
    }
}
