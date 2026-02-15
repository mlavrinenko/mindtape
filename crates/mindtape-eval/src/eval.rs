//! Evaluate a `.typ` file and extract tasks from the content tree.
//!
//! This module is the bridge between Typst evaluation and the task tracker.
//! It calls `typst_eval::eval()` to get a `Module`, then traverses the
//! resulting `Content` tree looking for `ListItem` elements whose body
//! text matches the checkbox convention (`[ ] ` / `[x] `).  Metadata
//! elements produced by `#due()` and `#id()` calls are extracted from
//! each item's body.

use std::fmt;
use std::ops::ControlFlow;

use comemo::Track;
use log::{debug, trace};
use thiserror::Error;
use typst::engine::{Route, Sink, Traced};
use typst::foundations::{Content, Datetime, Module, Value};
use typst::World;
use typst::ROUTINES;
use typst_library::introspection::MetadataElem;
use typst_library::model::{HeadingElem, ListItem};

/// Errors that can occur during Typst evaluation.
#[derive(Debug, Error)]
pub enum EvalError {
    /// The source file could not be read.
    #[error("file error: {0}")]
    File(String),

    /// Typst evaluation produced errors.
    #[error("eval error: {0}")]
    Eval(String),

    /// The world could not be constructed.
    #[error("{0}")]
    World(String),
}

impl EvalError {
    fn from_file_error(err: &typst::diag::FileError) -> Self {
        Self::File(err.to_string())
    }

    fn from_source_diagnostics(errors: &impl fmt::Debug) -> Self {
        Self::Eval(format!("{errors:?}"))
    }
}

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
    /// File dependencies (relative paths from project root).
    pub dependencies: Vec<std::path::PathBuf>,
}

/// Evaluate the world's main `.typ` file and return all tasks found in its
/// content tree.
///
/// # Errors
/// Returns `Err` if the source file cannot be read or Typst evaluation fails.
pub fn eval_file(world: &dyn World) -> Result<Vec<Task>, EvalError> {
    eval_file_full(world).map(|r| r.tasks)
}

/// Evaluate the world's main `.typ` file and return tasks, title, and bindings.
///
/// # Errors
/// Returns `Err` if the source file cannot be read or Typst evaluation fails.
pub fn eval_file_full(world: &dyn World) -> Result<EvalResult, EvalError> {
    let source = world
        .source(world.main())
        .map_err(|err| EvalError::from_file_error(&err))?;

    debug!("evaluating {}", source.id().vpath().as_rooted_path().display());

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
    .map_err(|err| EvalError::from_source_diagnostics(&err))?;

    // Extract bindings BEFORE content() consumes the module.
    let bindings = extract_bindings(module.scope());
    trace!("extracted {} bindings", bindings.len());

    let content: Content = module.content();

    let title = extract_file_title(&content);

    let mut tasks = Vec::new();
    collect_tasks(&content, &mut tasks);

    debug!("found {} tasks, {} bindings", tasks.len(), bindings.len());

    Ok(EvalResult {
        tasks,
        title,
        bindings,
        dependencies: Vec::new(), // Generic World trait doesn't expose dependencies
    })
}

/// Evaluate a `MindTapeWorld`'s main file and return tasks, title, bindings, and dependencies.
///
/// This variant works with concrete `MindTapeWorld` instances and can extract
/// cross-file dependencies discovered during evaluation.
///
/// # Errors
/// Returns `Err` if the source file cannot be read or Typst evaluation fails.
pub fn eval_file_full_with_deps(
    world: &crate::world::MindTapeWorld,
) -> Result<EvalResult, EvalError> {
    let source = world
        .source(world.main())
        .map_err(|err| EvalError::from_file_error(&err))?;

    debug!("evaluating (with deps) {}", source.id().vpath().as_rooted_path().display());

    let mut sink = Sink::new();
    let traced = Traced::default();
    let route = Route::default();

    // Cast to &dyn World for the track() method
    let world_dyn: &dyn World = world;

    let module: Module = typst_eval::eval(
        &ROUTINES,
        world_dyn.track(),
        traced.track(),
        sink.track_mut(),
        route.track(),
        &source,
    )
    .map_err(|err| EvalError::from_source_diagnostics(&err))?;

    // Extract bindings BEFORE content() consumes the module.
    let bindings = extract_bindings(module.scope());

    let content: Content = module.content();

    let title = extract_file_title(&content);

    let mut tasks = Vec::new();
    collect_tasks(&content, &mut tasks);

    let dependencies = world.get_dependencies()?;

    debug!(
        "found {} tasks, {} bindings, {} deps",
        tasks.len(), bindings.len(), dependencies.len()
    );

    Ok(EvalResult {
        tasks,
        title,
        bindings,
        dependencies,
    })
}

/// Recursively traverse `content` looking for `ListItem` nodes and
/// extract a `Task` from each one, assigning 0-based positions.
pub fn collect_tasks(content: &Content, tasks: &mut Vec<Task>) {
    let mut position: u32 = 0;
    let _ = content.traverse(&mut |node: Content| -> ControlFlow<()> {
        if let Some(item) = node.to_packed::<ListItem>()
            && let Some(mut task) = extract_task(item)
        {
            task.position = position;
            position += 1;
            tasks.push(task);
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
                if slice.len() == 2 && let Value::Str(key) = &slice[0] {
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

/// Format a `Datetime` as `YYYY-MM-DD`.
#[must_use]
pub fn format_date(dt: &Datetime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        dt.year().unwrap_or(0),
        dt.month().unwrap_or(0),
        dt.day().unwrap_or(0),
    )
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
            Value::Datetime(dt) => ("date", format!("\"{}\"", format_date(dt))),
            Value::None => ("none", "null".to_string()),
            _ => continue,
        };
        bindings.push((name.to_string(), vtype.to_string(), vjson));
    }
    bindings
}
