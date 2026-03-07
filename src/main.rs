use anyhow::{Result, bail};
use clap::Parser;
use log::LevelFilter;

use mindtape::cli::{self, Cli, Command};

fn main() -> Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();
    let args = cli::preprocess_args(raw_args);
    let cli = Cli::parse_from(args);

    init_logger(cli.verbose);

    match cli.command {
        Some(Command::Watch(args)) => args.run(),
        Some(Command::List(args)) => args.run(),
        Some(Command::Inspect(args)) => args.run(),
        Some(Command::Check(args)) => args.run(),
        Some(Command::Set(args)) => args.run(),
        Some(Command::Id(args)) => args.run(),
        Some(Command::Init(args)) => args.run(),
        None => {
            // Eval mode (backwards compat: `mindtape file.typ`)
            let Some(file) = cli.file else {
                bail!("no file specified (use --help for usage)");
            };
            cli::commands::eval::run(&file, cli.due, cli.limit)
        }
    }
}

/// Initialize `env_logger` based on the `-v` count.
///
/// `RUST_LOG` overrides the flag when set.
fn init_logger(verbosity: u8) {
    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(LevelFilter::Warn)
        .filter_module("mindtape", level)
        .filter_module("mindtape_eval", level)
        .filter_module("mindtape_store", level)
        .format_target(false)
        .format_timestamp(None)
        .init();
}
