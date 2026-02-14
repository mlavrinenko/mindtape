//! Typst evaluation and task extraction for `MindTape`.
//!
//! This crate evaluates `.typ` files using `typst-eval` and extracts
//! tasks from the content tree. It provides the `MindTapeWorld` (a
//! `typst::World` implementation) and task extraction functions.

mod eval;
pub mod world;

// Re-export eval's public API at crate root.
pub use eval::{
    collect_tasks, eval_file, eval_file_full, eval_file_full_with_deps, extract_bindings,
    extract_file_title, extract_task, EvalError, EvalResult, Task,
};
