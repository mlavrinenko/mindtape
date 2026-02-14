//! Evaluate a `.typ` file and extract tasks from the content tree.
//!
//! This module is the bridge between Typst evaluation and the task tracker.
//! It calls `typst_eval::eval()` to get a `Module`, then traverses the
//! resulting `Content` tree looking for `ListItem` elements whose body
//! text matches the checkbox convention (`[ ] ` / `[x] `).  Metadata
//! elements produced by `#due()` and `#id()` calls are extracted from
//! each item's body.

use std::ops::ControlFlow;

use comemo::Track;
use typst::engine::{Route, Sink, Traced};
use typst::foundations::{Content, Datetime, Module, Value};
use typst::World;
use typst::ROUTINES;
use typst_library::introspection::MetadataElem;
use typst_library::model::{HeadingElem, ListItem};

/// A task extracted from a Typst checklist item.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub title: String,
    pub done: bool,
    pub due: Option<Datetime>,
    pub tags: Vec<String>,
    pub position: u32,
}

/// Full result from evaluating a `.typ` file.
#[derive(Debug, Clone)]
pub struct EvalResult {
    pub tasks: Vec<Task>,
    pub title: Option<String>,
    /// Bindings as `(name, value_type, value_json)` tuples.
    pub bindings: Vec<(String, String, String)>,
}

/// Evaluate the world's main `.typ` file and return all tasks found in its
/// content tree.
///
/// # Errors
/// Returns `Err` if the source file cannot be read or Typst evaluation fails.
pub fn eval_file(world: &dyn World) -> Result<Vec<Task>, String> {
    eval_file_full(world).map(|r| r.tasks)
}

/// Evaluate the world's main `.typ` file and return tasks, title, and bindings.
///
/// # Errors
/// Returns `Err` if the source file cannot be read or Typst evaluation fails.
pub fn eval_file_full(world: &dyn World) -> Result<EvalResult, String> {
    let source = world
        .source(world.main())
        .map_err(|e: typst::diag::FileError| e.to_string())?;

    let mut sink = Sink::new();
    let traced = Traced::default();
    let route = Route::default();

    let module: Module = typst_eval::eval(
        &ROUTINES,
        world.track(),
        traced.track(),
        sink.track_mut(),
        route.track(),
        &source,
    )
    .map_err(|errors| format!("{errors:?}"))?;

    // Extract bindings BEFORE content() consumes the module.
    let bindings = extract_bindings(module.scope());

    let content: Content = module.content();

    let title = extract_file_title(&content);

    let mut tasks = Vec::new();
    collect_tasks(&content, &mut tasks);

    Ok(EvalResult { tasks, title, bindings })
}

/// Recursively traverse `content` looking for `ListItem` nodes and
/// extract a `Task` from each one, assigning 0-based positions.
pub fn collect_tasks(content: &Content, tasks: &mut Vec<Task>) {
    let mut position: u32 = 0;
    let _ = content.traverse(&mut |node: Content| -> ControlFlow<()> {
        if let Some(item) = node.to_packed::<ListItem>() {
            if let Some(mut task) = extract_task(item) {
                task.position = position;
                position += 1;
                tasks.push(task);
            }
        }
        ControlFlow::Continue(())
    });
}

/// Try to interpret a single `ListItem` as a task.
///
/// Returns `None` if the item's text does not start with a checkbox marker.
#[allow(clippy::indexing_slicing)]
pub fn extract_task(item: &ListItem) -> Option<Task> {
    let body: &Content = &item.body;
    let text = body.plain_text();

    // Parse checkbox prefix.
    let (done, title) = if let Some(rest) = text.strip_prefix("[x] ")
        .or_else(|| text.strip_prefix("[X] "))
    {
        (true, rest.trim().to_string())
    } else if let Some(rest) = text.strip_prefix("[ ] ") {
        (false, rest.trim().to_string())
    } else {
        // Not a checkbox item — skip.
        return None;
    };

    // Walk the body content looking for MetadataElem nodes produced by
    // `#due()`, `#id()`, and `#tag()`.
    let mut due: Option<Datetime> = None;
    let mut tags: Vec<String> = Vec::new();

    let _ = body.traverse(&mut |node: Content| -> ControlFlow<()> {
        if let Some(meta) = node.to_packed::<MetadataElem>() {
            let value = meta.value.clone();
            if let Value::Array(arr) = value {
                let slice = arr.as_slice();
                if slice.len() == 2 {
                    if let Value::Str(key) = &slice[0] {
                        match key.as_str() {
                            "due" => {
                                if let Value::Datetime(dt) = &slice[1] {
                                    due = Some(*dt);
                                }
                            }
                            "tag" => {
                                if let Value::Str(name) = &slice[1] {
                                    tags.push(name.to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        ControlFlow::Continue(())
    });

    Some(Task { title, done, due, tags, position: 0 })
}

/// Extract the title from the first heading in the content tree.
pub fn extract_file_title(content: &Content) -> Option<String> {
    let mut title = None;
    let _ = content.traverse(&mut |node: Content| -> ControlFlow<()> {
        if let Some(heading) = node.to_packed::<HeadingElem>() {
            title = Some(heading.body.plain_text().trim().to_string());
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    });
    title
}

/// Extract `#let` bindings from a module's scope as `(name, value_type, value_json)`.
///
/// Skips functions and other non-data values.
pub fn extract_bindings(scope: &typst::foundations::Scope) -> Vec<(String, String, String)> {
    let mut bindings = Vec::new();
    for (name, binding) in scope.iter() {
        let value = binding.read();
        let (vtype, vjson) = match value {
            Value::Str(str_val) => ("string", format!("\"{}\"", str_val.as_str())),
            Value::Int(int_val) => ("int", int_val.to_string()),
            Value::Float(float_val) => ("float", float_val.to_string()),
            Value::Bool(bool_val) => ("bool", bool_val.to_string()),
            Value::Datetime(dt) => ("date", format!(
                "\"{:04}-{:02}-{:02}\"",
                dt.year().unwrap_or(0),
                dt.month().unwrap_or(0),
                dt.day().unwrap_or(0),
            )),
            Value::None => ("none", "null".to_string()),
            _ => continue,
        };
        bindings.push((name.to_string(), vtype.to_string(), vjson));
    }
    bindings
}
