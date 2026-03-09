use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use comfy_table::Table;
use comfy_table::presets::ASCII_BORDERS_ONLY_CONDENSED;
use serde::Serialize;

use crate::cli::format::{
    format_file_errors_csv, format_files_csv, format_stats, format_stats_csv,
};
use crate::cli::util::{home_dir, open_query_db, print_json, resolve_query_db_path, short_path};
use crate::cli::{OutputFormat, QueryOpts};
use crate::store::{FileDependencies, FileError, FileView, IndexStats, Store};

use super::deps::{format_all_deps_csv, format_deps, format_deps_csv};

/// Which section(s) to display.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum Section {
    Status,
    Files,
    Deps,
    Errors,
}

/// Inspect the index: statistics, files, and dependencies.
#[derive(Parser, Debug)]
pub struct InspectArgs {
    /// Sections to display (omit for all)
    #[arg(long)]
    pub with: Vec<Section>,

    /// Show dependencies for a specific file
    #[arg(long)]
    pub file: Option<PathBuf>,

    #[command(flatten)]
    pub query: QueryOpts,
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
enum DepsOutput {
    Single(FileDependencies),
    All(Vec<FileDependencies>),
}

/// Combined JSON output.
#[derive(Serialize)]
struct InspectJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<IndexStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<Vec<FileView>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deps: Option<DepsOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<FileError>>,
}

impl InspectArgs {
    /// Run the inspect command.
    ///
    /// # Errors
    /// Returns error if database open or query fails, or if formatting fails.
    pub fn run(&self) -> Result<()> {
        let format = self.query.output_format();
        let db_path = resolve_query_db_path(self.query.db.as_deref());
        let store = open_query_db(&db_path)?;

        let stats = self
            .wants(Section::Status)
            .then(|| store.get_stats().context("failed to query stats"))
            .transpose()?;
        let files = self
            .wants(Section::Files)
            .then(|| store.list_files().context("failed to list files"))
            .transpose()?;
        let deps = self
            .wants(Section::Deps)
            .then(|| self.query_deps(&store))
            .transpose()?;
        let errors = self
            .wants(Section::Errors)
            .then(|| {
                store
                    .list_file_errors()
                    .context("failed to list file errors")
            })
            .transpose()?;

        match format {
            OutputFormat::Json => print_json(&InspectJson {
                status: stats,
                files,
                deps,
                errors,
            }),
            OutputFormat::Csv => print_sections(&stats, &files, &deps, &errors, SectionFmt::Csv),
            OutputFormat::Table | OutputFormat::Typst => {
                let sfmt = if format == OutputFormat::Typst {
                    SectionFmt::Typst
                } else {
                    SectionFmt::Table
                };
                if stats.is_some() {
                    eprintln!("Database: {}", short_path(&db_path));
                }
                print_sections(&stats, &files, &deps, &errors, sfmt)
            }
        }
    }

    fn wants(&self, section: Section) -> bool {
        self.with.is_empty() || self.with.contains(&section)
    }

    fn query_deps(&self, store: &impl Store) -> Result<DepsOutput> {
        if let Some(file) = &self.file {
            let deps = store
                .get_file_dependencies(file)
                .context("failed to query dependencies")?
                .ok_or_else(|| anyhow::anyhow!("file not found in index: {}", file.display()))?;
            Ok(DepsOutput::Single(deps))
        } else {
            let all = store
                .list_file_dependencies()
                .context("failed to list dependencies")?;
            Ok(DepsOutput::All(all))
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SectionFmt {
    Table,
    Typst,
    Csv,
}

fn print_sections(
    stats: &Option<IndexStats>,
    files: &Option<Vec<FileView>>,
    deps: &Option<DepsOutput>,
    errors: &Option<Vec<FileError>>,
    fmt: SectionFmt,
) -> Result<()> {
    let parts = match fmt {
        SectionFmt::Table | SectionFmt::Typst => build_human_parts(stats, files, deps, errors, fmt),
        SectionFmt::Csv => build_csv_parts(stats, files, deps, errors)?,
    };

    let multi = parts.len() > 1;
    let header = match fmt {
        SectionFmt::Table if multi => |name| format!("=== {name} ===\n"),
        SectionFmt::Typst if multi => |name| format!("== {name}\n"),
        SectionFmt::Csv if multi => |name| format!("# section: {name}\n"),
        _ => |_| String::new(),
    };

    for (idx, (name, body)) in parts.iter().enumerate() {
        if idx > 0 {
            println!();
        }
        print!("{}{body}", header(name));
    }
    Ok(())
}

fn build_human_parts<'a>(
    stats: &Option<IndexStats>,
    files: &Option<Vec<FileView>>,
    deps: &Option<DepsOutput>,
    errors: &Option<Vec<FileError>>,
    fmt: SectionFmt,
) -> Vec<(&'a str, String)> {
    let home = home_dir();
    let mut parts: Vec<(&str, String)> = Vec::new();

    if let Some(st) = stats {
        parts.push(("Status", format!("{}\n", format_stats(st))));
    }
    if let Some(fi) = files {
        let body = if fi.is_empty() {
            "no indexed files\n".to_string()
        } else {
            format!("{}\n", build_files_table(fi, &home))
        };
        parts.push(("Files", body));
    }
    if let Some(dep_data) = deps {
        let body = match dep_data {
            DepsOutput::Single(single) => format_deps(single, &home),
            DepsOutput::All(all) if all.is_empty() => "no dependencies\n".to_string(),
            DepsOutput::All(all) => format!("{}\n", build_deps_table(all, &home)),
        };
        parts.push(("Dependencies", body));
    }
    if let Some(errs) = errors {
        let body = if errs.is_empty() {
            "no errors\n".to_string()
        } else if fmt == SectionFmt::Typst {
            format_errors_typst(errs, &home)
        } else {
            format!("{}\n", build_errors_table(errs, &home))
        };
        parts.push(("Errors", body));
    }
    parts
}

fn build_csv_parts<'a>(
    stats: &Option<IndexStats>,
    files: &Option<Vec<FileView>>,
    deps: &Option<DepsOutput>,
    errors: &Option<Vec<FileError>>,
) -> Result<Vec<(&'a str, String)>> {
    let mut parts: Vec<(&str, String)> = Vec::new();

    if let Some(st) = stats {
        parts.push(("status", format_stats_csv(st)?));
    }
    if let Some(fi) = files {
        parts.push(("files", format_files_csv(fi)?));
    }
    if let Some(dep_data) = deps {
        let csv = match dep_data {
            DepsOutput::Single(single) => format_deps_csv(single)?,
            DepsOutput::All(all) => format_all_deps_csv(all)?,
        };
        parts.push(("deps", csv));
    }
    if let Some(errs) = errors {
        parts.push(("errors", format_file_errors_csv(errs)?));
    }
    Ok(parts)
}

// ---------------------------------------------------------------------------
// Table builders
// ---------------------------------------------------------------------------

fn new_table() -> Table {
    let mut table = Table::new();
    table.load_preset(ASCII_BORDERS_ONLY_CONDENSED);
    table
}

fn build_files_table(files: &[FileView], home: &str) -> String {
    let mut table = new_table();
    table.set_header(vec!["Path", "Tasks", "Title"]);
    for f in files {
        let title = f.title.as_deref().unwrap_or("");
        let path = crate::cli::util::shorten_home(&f.file_path, home);
        table.add_row(vec![&path, &f.task_count.to_string(), title]);
    }
    table.to_string()
}

fn build_deps_table(all_deps: &[FileDependencies], home: &str) -> String {
    let mut table = new_table();
    table.set_header(vec!["Path", "Title", "Imports", "Imported by"]);
    for d in all_deps {
        let title = d.file_title.as_deref().unwrap_or("");
        let path = crate::cli::util::shorten_home(&d.file_path, home);
        table.add_row(vec![
            &path,
            title,
            &d.imports.len().to_string(),
            &d.imported_by.len().to_string(),
        ]);
    }
    table.to_string()
}

fn build_errors_table(errors: &[FileError], home: &str) -> String {
    let mut table = new_table();
    table.set_header(vec!["Path", "Error"]);
    for e in errors {
        let path = crate::cli::util::shorten_home(&e.file_path, home);
        table.add_row(vec![&path, &e.error]);
    }
    table.to_string()
}

fn format_errors_typst(errors: &[FileError], home: &str) -> String {
    let mut out = String::new();
    for e in errors {
        let path = crate::cli::util::shorten_home(&e.file_path, home);
        out.push_str(&format!("- `{path}`\n  ```\n  {}\n  ```\n", e.error));
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn files_table_renders() {
        let files = vec![FileView {
            file_path: PathBuf::from("notes/todo.typ"),
            watch_root: None,
            title: Some("My Tasks".to_string()),
            task_count: 5,
            updated_at: "2026-01-01".to_string(),
        }];
        let out = build_files_table(&files, "/nonexistent");
        assert!(out.contains("notes/todo.typ"));
        assert!(out.contains("5"));
        assert!(out.contains("My Tasks"));
        assert!(out.contains("Path"));
    }

    #[test]
    fn files_table_shortens_home() {
        let files = vec![FileView {
            file_path: PathBuf::from("/home/user/notes/todo.typ"),
            watch_root: None,
            title: None,
            task_count: 1,
            updated_at: "2026-01-01".to_string(),
        }];
        let out = build_files_table(&files, "/home/user");
        assert!(out.contains("~/notes/todo.typ"));
        assert!(!out.contains("/home/user"));
    }

    #[test]
    fn deps_table_renders() {
        let deps = vec![FileDependencies {
            file_path: PathBuf::from("a.typ"),
            file_title: Some("File A".to_string()),
            imports: vec![PathBuf::from("b.typ")],
            imported_by: vec![],
        }];
        let out = build_deps_table(&deps, "");
        assert!(out.contains("a.typ"));
        assert!(out.contains("File A"));
        assert!(out.contains("1"));
    }

    #[test]
    fn errors_table_renders() {
        let errors = vec![FileError {
            file_path: PathBuf::from("/home/user/broken.typ"),
            watch_root: None,
            error: "eval error: undefined variable".to_string(),
            updated_at: "2026-03-01".to_string(),
        }];
        let out = build_errors_table(&errors, "/home/user");
        assert!(out.contains("~/broken.typ"));
        assert!(out.contains("eval error: undefined variable"));
    }

    #[test]
    fn errors_typst_format() {
        let errors = vec![FileError {
            file_path: PathBuf::from("/home/user/pad/notes/preorders.typ"),
            watch_root: None,
            error: "eval error: undefined variable".to_string(),
            updated_at: "2026-03-01".to_string(),
        }];
        let out = format_errors_typst(&errors, "/home/user");
        assert!(out.contains("- `~/pad/notes/preorders.typ`"));
        assert!(out.contains("  ```"));
        assert!(out.contains("  eval error: undefined variable"));
    }
}
