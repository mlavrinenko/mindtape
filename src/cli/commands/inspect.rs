use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::Serialize;

use crate::cli::format::{format_file_view, format_files_csv, format_stats, format_stats_csv};
use crate::cli::util::{open_query_db, print_json, resolve_query_db_path};
use crate::cli::{OutputFormat, QueryOpts};
use crate::store::{FileDependencies, FileView, IndexStats, Store};

use super::deps::{format_all_deps, format_all_deps_csv, format_deps, format_deps_csv};

/// Which section(s) to display.
#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum Section {
    Status,
    Files,
    Deps,
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

        match format {
            OutputFormat::Json => print_json(&InspectJson {
                status: stats,
                files,
                deps,
            }),
            OutputFormat::Csv => print_sections(&stats, &files, &deps, SectionFmt::Csv),
            OutputFormat::Table => {
                if stats.is_some() {
                    eprintln!("Database: {}", db_path.display());
                }
                print_sections(&stats, &files, &deps, SectionFmt::Table)
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

#[derive(Clone, Copy)]
enum SectionFmt {
    Table,
    Csv,
}

fn print_sections(
    stats: &Option<IndexStats>,
    files: &Option<Vec<FileView>>,
    deps: &Option<DepsOutput>,
    fmt: SectionFmt,
) -> Result<()> {
    let mut parts: Vec<(&str, String)> = Vec::new();

    match fmt {
        SectionFmt::Table => {
            if let Some(st) = stats {
                parts.push(("Status", format!("{}\n", format_stats(st))));
            }
            if let Some(fi) = files {
                let body = if fi.is_empty() {
                    "no indexed files\n".to_string()
                } else {
                    fi.iter()
                        .map(|file| format!("{}\n", format_file_view(file)))
                        .collect()
                };
                parts.push(("Files", body));
            }
            if let Some(dep_data) = deps {
                let body = match dep_data {
                    DepsOutput::Single(single) => format_deps(single),
                    DepsOutput::All(all) if all.is_empty() => "no dependencies\n".to_string(),
                    DepsOutput::All(all) => format_all_deps(all),
                };
                parts.push(("Dependencies", body));
            }
        }
        SectionFmt::Csv => {
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
        }
    }

    let multi = parts.len() > 1;
    let header = match fmt {
        SectionFmt::Table if multi => |name| format!("=== {name} ===\n"),
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
