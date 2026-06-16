//! `MindTape` CLI entry point: parse arguments, init logging, dispatch.

mod check;
mod cleanup;
mod cli;
mod eval;
mod git;
mod init;
mod list;
mod output;
mod util;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli::init_logger(cli.verbose);
    cli.run()
}
