use anyhow::{Context, Result};

use crate::cli::format::{format_stats, format_stats_csv};
use crate::cli::util::{open_query_db, print_json};
use crate::cli::{OutputFormat, StatusArgs};
use crate::store::Store;

impl StatusArgs {
    /// Run the status command.
    ///
    /// # Errors
    /// Returns error if database open or query fails, or if JSON/CSV formatting fails.
    pub fn run(&self) -> Result<()> {
        let format = self.query.output_format();
        let store = open_query_db(self.query.db.as_deref())?;

        let stats = store.get_stats().context("failed to query stats")?;

        match format {
            OutputFormat::Json => print_json(&stats)?,
            OutputFormat::Csv => print!("{}", format_stats_csv(&stats)?),
            OutputFormat::Table => println!("{}", format_stats(&stats)),
        }
        Ok(())
    }
}
