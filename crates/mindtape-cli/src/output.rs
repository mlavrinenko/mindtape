//! Rendering tasks as a table, JSON, or CSV.

use anyhow::Result;
use comfy_table::Table;
use comfy_table::presets::UTF8_FULL;
use mindtape::Task;
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
pub fn render(tasks: &[Task], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Table => Ok(render_table(tasks)),
        OutputFormat::Json => render_json(tasks),
        OutputFormat::Csv => render_csv(tasks),
    }
}

fn render_table(tasks: &[Task]) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(["ID", "STATUS", "PRIORITY", "DUE", "TITLE"]);
    for task in tasks {
        table.add_row([
            task.id.clone(),
            status_name(task),
            task.priority.clone().unwrap_or_default(),
            task.due.clone().unwrap_or_default(),
            task.title.clone(),
        ]);
    }
    table.to_string()
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
            task.priority.as_deref().unwrap_or_default(),
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
    map.insert("priority".to_owned(), opt(task.priority.as_deref()));
    map.insert("grade".to_owned(), opt(task.grade.as_deref()));
    map.insert("kind".to_owned(), opt(task.kind.as_deref()));
    map.insert(
        "estimate_hours".to_owned(),
        task.estimate_hours
            .map_or(Value::Null, |hours| json!(hours)),
    );
    map.insert("depends_on".to_owned(), json!(task.depends_on));
    map.insert("relates_to".to_owned(), json!(task.relates_to));
    map.insert("tags".to_owned(), json!(task.tags));
    map.insert("due".to_owned(), opt(task.due.as_deref()));
    map.insert("start".to_owned(), opt(task.start.as_deref()));
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

    use mindtape::{Status, Task};

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
            priority: Some("high".to_owned()),
            grade: None,
            kind: None,
            estimate_hours: Some(8.0),
            depends_on: Vec::new(),
            relates_to: Vec::new(),
            tags: vec!["x".to_owned()],
            due: None,
            start: None,
            extras: BTreeMap::new(),
        }
    }

    #[test]
    fn table_contains_fields() {
        let out = render(&[sample()], OutputFormat::Table).unwrap();
        assert!(out.contains("t1"));
        assert!(out.contains("Build"));
        assert!(out.contains("wip"));
    }

    #[test]
    fn json_exposes_core_fields() {
        let out = render(&[sample()], OutputFormat::Json).unwrap();
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value[0]["id"], "t1");
        assert_eq!(value[0]["status"]["name"], "wip");
        assert_eq!(value[0]["estimate_hours"], 8.0);
    }

    #[test]
    fn csv_has_header_and_row() {
        let out = render(&[sample()], OutputFormat::Csv).unwrap();
        assert!(out.contains("id,status,priority,due,title"));
        assert!(out.contains("t1,wip,high,,Build"));
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
