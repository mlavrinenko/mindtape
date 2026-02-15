use anyhow::{Context, Result};

use crate::cli::format::{format_stats, format_stats_csv};
use crate::cli::util::{open_query_db, print_json, resolve_query_db_path};
use crate::cli::{OutputFormat, StatusArgs};
use crate::store::Store;

impl StatusArgs {
    /// Run the status command.
    ///
    /// # Errors
    /// Returns error if database open or query fails, or if JSON/CSV formatting fails.
    pub fn run(&self) -> Result<()> {
        let format = self.query.output_format();
        let db_path = resolve_query_db_path(self.query.db.as_deref());
        let store = open_query_db(&db_path)?;

        let stats = store.get_stats().context("failed to query stats")?;

        eprintln!("Database: {}", db_path.display());

        match format {
            OutputFormat::Json => print_json(&stats)?,
            OutputFormat::Csv => print!("{}", format_stats_csv(&stats)?),
            OutputFormat::Table => println!("{}", format_stats(&stats)),
        }
        Ok(())
    }
}
