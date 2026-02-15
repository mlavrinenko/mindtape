use clap::Parser;
use csv::Writer;

use crate::cli::format::{csv_to_string, format_task_view};
use crate::store::SearchResults;

use crate::cli::QueryOpts;

/// Search tasks and bindings.
#[derive(Parser, Debug)]
pub struct SearchArgs {
    /// Search keyword
    pub keyword: String,

    /// Limit output to N items
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,

    #[command(flatten)]
    pub query: QueryOpts,
}

#[must_use]
pub fn format_search_results(results: &SearchResults) -> String {
    let mut out = String::new();

    if !results.tasks.is_empty() {
        out.push_str(&format!("Tasks ({}):\n", results.tasks.len()));
        let mut current_file = String::new();
        for task in &results.tasks {
            let file_str = task.file_path.to_string_lossy();
            if file_str != current_file {
                out.push_str(&format!("  {file_str}\n"));
                current_file = file_str.to_string();
            }
            out.push_str(&format!("    {}\n", format_task_view(task)));
        }
    }

    if !results.bindings.is_empty() {
        if !results.tasks.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("Bindings ({}):\n", results.bindings.len()));
        for binding in &results.bindings {
            out.push_str(&format!(
                "  {} = {}  ({})\n",
                binding.name,
                binding.value,
                binding.file_path.display(),
            ));
        }
    }

    if results.tasks.is_empty() && results.bindings.is_empty() {
        out.push_str("no matches found\n");
    }

    out
}

/// Formats search results as CSV
///
/// # Errors
/// Returns error if CSV writing fails (unlikely with in-memory writer)
pub fn format_search_csv(results: &SearchResults) -> Result<String, csv::Error> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(["type", "name", "value", "file"])?;
    for task in &results.tasks {
        let status = if task.is_done { "done" } else { "pending" };
        wtr.write_record(&[
            "task",
            &task.title,
            status,
            &task.file_path.to_string_lossy(),
        ])?;
    }
    for binding in &results.bindings {
        wtr.write_record(&[
            "binding",
            &binding.name,
            &binding.value,
            &binding.file_path.to_string_lossy(),
        ])?;
    }
    csv_to_string(wtr)
}
