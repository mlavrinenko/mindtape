//! Typst evaluation and task extraction for `MindTape`.
//!
//! This crate evaluates `.typ` files using `typst-eval` and extracts
//! tasks from the content tree. It provides the `MindTapeWorld` (a
//! `typst::World` implementation) and task extraction functions.

mod eval;
pub mod world;
pub mod write;

// Re-export eval's public API at crate root.
pub use eval::{
    collect_tasks, eval_file, eval_file_full, eval_file_full_with_deps, extract_bindings,
    extract_file_title, extract_task, format_date, has_mindtape_import, EvalError, EvalResult,
    Task,
};

// Re-export write's public API at crate root.
pub use write::{
    add_task_tag, load_source, remove_task_due, remove_task_tag, set_task_due,
    toggle_task_checkbox, WriteError,
};
