use anyhow::{Context, Result};

use crate::cli::format::{format_file_view, format_files_csv};
use crate::cli::util::{open_query_db, print_json, resolve_query_db_path};
use crate::cli::{FilesArgs, OutputFormat};
use crate::store::Store;

impl FilesArgs {
    /// Run the files command.
    ///
    /// # Errors
    /// Returns error if database open or query fails, or if JSON/CSV formatting fails.
    pub fn run(&self) -> Result<()> {
        let format = self.query.output_format();
        let db_path = resolve_query_db_path(self.query.db.as_deref());
        let store = open_query_db(&db_path)?;

        let files = store.list_files().context("failed to list files")?;

        match format {
            OutputFormat::Json => print_json(&files)?,
            OutputFormat::Csv => print!("{}", format_files_csv(&files)?),
            OutputFormat::Table => {
                if files.is_empty() {
                    eprintln!("no indexed files");
                    return Ok(());
                }
                for file in &files {
                    println!("{}", format_file_view(file));
                }
            }
        }
        Ok(())
    }
}
