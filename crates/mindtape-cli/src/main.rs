//! `MindTape` CLI entry point: parse arguments, init logging, dispatch.

mod check;
mod cli;
mod eval;
mod git;
mod handle;
mod init;
mod list;
mod output;
// Infrastructure for future command migration; not every item is wired yet.
#[allow(dead_code)]
mod render;
mod review;
mod rm;
mod util;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli::init_logger(cli.verbose);
    // Resolve colour once here; commands may query this via `color_enabled`.
    // The value is stored in a thread-local so the rest of the binary can
    // read it without threading the choice through every call-site.
    init_color(cli.color);
    cli.run()
}

fn init_color(choice: crate::render::color::ColorChoice) {
    // Eagerly resolve and store whether colour is active for this invocation.
    // The result is cached in a module-level atomic so later callers don't
    // re-check the TTY or env on every use.
    crate::render::color::init(choice);
}
