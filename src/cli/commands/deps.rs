use csv::Writer;

use crate::cli::format::csv_to_string;
use crate::cli::util::shorten_home;
use crate::store::FileDependencies;

#[must_use]
pub fn format_deps(deps: &FileDependencies, home: &str) -> String {
    let mut output = String::new();

    output.push_str(&format!("File: {}", shorten_home(&deps.file_path, home)));
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
                output.push_str(&format!("  - {}\n", shorten_home(import, home)));
            }
            if !deps.imported_by.is_empty() {
                output.push('\n');
            }
        }

        if !deps.imported_by.is_empty() {
            output.push_str("Imported by:\n");
            for imported_by in &deps.imported_by {
                output.push_str(&format!("  - {}\n", shorten_home(imported_by, home)));
            }
        }
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
