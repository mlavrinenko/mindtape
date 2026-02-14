use std::path::PathBuf;

use clap::Parser;

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

#[must_use]
pub fn format_deps_csv(deps: &FileDependencies) -> String {
    let mut csv = String::from("type,file,target\n");

    for import in &deps.imports {
        csv.push_str(&format!(
            "imports,{},{}\n",
            deps.file_path.display(),
            import.display()
        ));
    }

    for imported_by in &deps.imported_by {
        csv.push_str(&format!(
            "imported_by,{},{}\n",
            deps.file_path.display(),
            imported_by.display()
        ));
    }

    csv
}

#[must_use]
pub fn format_all_deps_csv(all_deps: &[FileDependencies]) -> String {
    let mut csv = String::from("file,title,imports_count,imported_by_count\n");

    for deps in all_deps {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            deps.file_path.display(),
            deps.file_title.as_deref().unwrap_or(""),
            deps.imports.len(),
            deps.imported_by.len()
        ));
    }

    csv
}
