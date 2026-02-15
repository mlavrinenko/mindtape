use std::path::PathBuf;

use clap::Parser;
use csv::Writer;

use crate::cli::format::csv_to_string;
use crate::cli::QueryOpts;
use crate::store::FileDependencies;

/// Show file dependencies.
#[derive(Parser, Debug)]
pub struct DepsArgs {
    /// Show dependencies for a specific file
    #[arg(long)]
    pub file: Option<PathBuf>,

    #[command(flatten)]
    pub query: QueryOpts,
}

#[must_use]
pub fn format_deps(deps: &FileDependencies) -> String {
    let mut output = String::new();

    output.push_str(&format!("File: {}", deps.file_path.display()));
    if let Some(title) = &deps.file_title {
        output.push_str(&format!(" ({title})"));
    }
    output.push_str("\n\n");

    if deps.imports.is_empty() && deps.imported_by.is_empty() {
        output.push_str("  No dependencies.\n");
    } else {
        if !deps.imports.is_empty() {
            output.push_str("Imports:\n");
            for import in &deps.imports {
                output.push_str(&format!("  - {}\n", import.display()));
            }
            if !deps.imported_by.is_empty() {
                output.push('\n');
            }
        }

        if !deps.imported_by.is_empty() {
            output.push_str("Imported by:\n");
            for imported_by in &deps.imported_by {
                output.push_str(&format!("  - {}\n", imported_by.display()));
            }
        }
    }

    output
}

#[must_use]
pub fn format_all_deps(all_deps: &[FileDependencies]) -> String {
    if all_deps.is_empty() {
        return "No files indexed.\n".to_string();
    }

    let mut output = String::new();
    for deps in all_deps {
        output.push_str(&format!("{}", deps.file_path.display()));
        if let Some(title) = &deps.file_title {
            output.push_str(&format!(" ({title})"));
        }

        if !deps.imports.is_empty() || !deps.imported_by.is_empty() {
            output.push_str(&format!(
                " — imports: {}, imported by: {}",
                deps.imports.len(),
                deps.imported_by.len()
            ));
        }
        output.push('\n');
    }
    output
}

/// Formats file dependencies as CSV
///
/// # Errors
/// Returns error if CSV writing fails (unlikely with in-memory writer)
pub fn format_deps_csv(deps: &FileDependencies) -> Result<String, csv::Error> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(["type", "file", "target"])?;

    for import in &deps.imports {
        wtr.write_record([
            "imports",
            &deps.file_path.to_string_lossy(),
            &import.to_string_lossy(),
        ])?;
    }

    for imported_by in &deps.imported_by {
        wtr.write_record([
            "imported_by",
            &deps.file_path.to_string_lossy(),
            &imported_by.to_string_lossy(),
        ])?;
    }

    csv_to_string(wtr)
}

/// Formats all file dependencies as CSV
///
/// # Errors
/// Returns error if CSV writing fails (unlikely with in-memory writer)
pub fn format_all_deps_csv(all_deps: &[FileDependencies]) -> Result<String, csv::Error> {
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(["file", "title", "imports_count", "imported_by_count"])?;

    for deps in all_deps {
        wtr.write_record([
            deps.file_path.to_string_lossy().as_ref(),
            deps.file_title.as_deref().unwrap_or(""),
            &deps.imports.len().to_string(),
            &deps.imported_by.len().to_string(),
        ])?;
    }

    csv_to_string(wtr)
}
