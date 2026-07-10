//! `MindTape` CLI entry point: parse arguments, init logging, dispatch.

mod add;
mod add_interactive;
mod add_values;
mod check;
mod cli;
mod config_cmd;
mod eval;
mod flip;
mod git;
mod go;
mod handle;
mod init;
mod install;
mod ls;
mod ls_interactive;
mod output;
mod recur;
mod recur_edit;
// Infrastructure for future command migration; not every item is wired yet.
#[allow(dead_code)]
mod render;
mod review;
mod rm;
mod task_edit;
#[cfg(test)]
mod test_util;
mod util;
mod views;
mod worktrees;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    // Rewrite a leading `@name` into `-w <path>` before clap sees the argv, so
    // the worktree sugar reuses the existing override plumbing.
    let argv = worktrees::desugar_argv(std::env::args_os().collect())?;
    let cli = cli::Cli::parse_from(argv);
    cli::init_logger(cli.verbose);
    // Resolve the worktree override once (flag over $MINDTAPE_WORKTREE) so every
    // command's discovery entry point honours it uniformly.
    util::init_worktree(cli.worktree.clone());
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
