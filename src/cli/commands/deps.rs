use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use csv::Writer;

use crate::cli::format::csv_to_string;
use crate::cli::util::{open_query_db, print_json, resolve_query_db_path};
use crate::cli::{OutputFormat, QueryOpts};
use crate::store::{FileDependencies, Store};

/// Show file dependencies.
#[derive(Parser, Debug)]
pub struct DepsArgs {
    /// Show dependencies for a specific file
    #[arg(long)]
    pub file: Option<PathBuf>,

    #[command(flatten)]
    pub query: QueryOpts,
}

impl DepsArgs {
    /// Run the deps command.
    ///
    /// # Errors
    /// Returns error if database open or query fails, or if JSON/CSV formatting fails.
    pub fn run(&self) -> Result<()> {
        let format = self.query.output_format();
        let db_path = resolve_query_db_path(self.query.db.as_deref());
        let store = open_query_db(&db_path)?;

        if let Some(file) = &self.file {
            let deps = store
                .get_file_dependencies(file)
                .context("failed to query dependencies")?
                .ok_or_else(|| {
                    anyhow::anyhow!("file not found in index: {}", file.display())
                })?;

            match format {
                OutputFormat::Json => print_json(&deps)?,
                OutputFormat::Csv => print!("{}", format_deps_csv(&deps)?),
                OutputFormat::Table => print!("{}", format_deps(&deps)),
            }
        } else {
            let all_deps = store
                .list_file_dependencies()
                .context("failed to list dependencies")?;

            match format {
                OutputFormat::Json => print_json(&all_deps)?,
                OutputFormat::Csv => print!("{}", format_all_deps_csv(&all_deps)?),
                OutputFormat::Table => print!("{}", format_all_deps(&all_deps)),
            }
        }
        Ok(())
    }
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
