use anyhow::{Result, bail};
use clap::Parser;

use mindtape::cli::{self, Cli, Command};

fn main() -> Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();
    let args = cli::preprocess_args(raw_args);
    let cli = Cli::parse_from(args);

    match cli.command {
        Some(Command::Watch(args)) => args.run(),
        Some(Command::List(args)) => args.run(),
        Some(Command::Search(args)) => args.run(),
        Some(Command::Agenda(args)) => args.run(),
        Some(Command::Status(args)) => args.run(),
        Some(Command::Files(args)) => args.run(),
        Some(Command::Deps(args)) => args.run(),
        Some(Command::Check(args)) => args.run(),
        Some(Command::Set(args)) => args.run(),
        None => {
            // Eval mode (backwards compat: `mindtape file.typ`)
            let Some(file) = cli.file else {
                bail!("no file specified (use --help for usage)");
            };
            cli::commands::eval::run(&file, cli.due, cli.limit)
        }
    }
}
