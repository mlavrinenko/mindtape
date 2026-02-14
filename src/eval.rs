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
use typst_library::model::ListItem;

/// A task extracted from a Typst checklist item.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub title: String,
    pub done: bool,
    pub due: Option<Datetime>,
    pub tags: Vec<String>,
}

/// Evaluate the world's main `.typ` file and return all tasks found in its
/// content tree.
///
/// A "task" is a `ListItem` whose `plain_text()` starts with a checkbox
/// marker (`[ ] ` or `[x] `).  Due dates are extracted from `MetadataElem`
/// nodes whose value is an `Array` of the form `("due", <datetime>)`.
pub fn eval_file(world: &dyn World) -> Result<Vec<Task>, String> {
    // 1. Obtain the main source from the world.
    let source = world
        .source(world.main())
        .map_err(|e: typst::diag::FileError| e.to_string())?;

    // 2. Evaluate the source into a Module.
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
    .map_err(|errors| format!("{:?}", errors))?;

    // 3. Get the content tree (consumes the module).
    let content: Content = module.content();

    // 4. Walk the content tree collecting tasks.
    let mut tasks = Vec::new();
    collect_tasks(&content, &mut tasks);

    Ok(tasks)
}

/// Recursively traverse `content` looking for `ListItem` nodes and
/// extract a `Task` from each one.
pub fn collect_tasks(content: &Content, tasks: &mut Vec<Task>) {
    let _ = content.traverse(&mut |node: Content| -> ControlFlow<()> {
        if let Some(item) = node.to_packed::<ListItem>() {
            if let Some(task) = extract_task(item) {
                tasks.push(task);
            }
        }
        ControlFlow::Continue(())
    });
}

/// Try to interpret a single `ListItem` as a task.
///
/// Returns `None` if the item's text does not start with a checkbox marker.
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

    Some(Task { title, done, due, tags })
}
