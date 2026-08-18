//! `MindTape` CLI entry point: parse arguments, init logging, dispatch.

mod add;
mod add_dedup;
mod add_interactive;
mod add_output;
mod add_tokens;
mod add_values;
mod check;
mod cli;
mod cli_dump;
mod command_class;
mod config_cmd;
mod eval;
mod find;
mod flip;
mod flip_hooks;
mod flip_hooks_cmd;
mod git;
mod go;
mod graph;
mod graph_render;
mod handle;
mod import;
mod init;
mod init_compose;
mod init_wizard;
mod install;
mod install_hint;
mod ls;
mod ls_interactive;
mod ls_rules;
mod output;
mod preset;
mod preset_cmd;
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
mod yaml_values;

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
    // Stash the `--hyperlinks` flag for `util::open` to resolve against `[ls]
    // hyperlinks` once a project's config is loaded (unlike colour, this
    // decision cannot be finished here — see `render::link`'s module docs).
    util::init_hyperlinks_flag(cli.hyperlinks);
    // Resolve colour once here; commands may query this via `color_enabled`.
    // The value is stored in a thread-local so the rest of the binary can
    // read it without threading the choice through every call-site.
    init_color(cli.color);
    // Resolve the pinned-clock override ($MINDTAPE_NOW) once, before any
    // command reads a date: every `mindtape::date` helper and
    // the today-stamping paths (add/flip/review/recur) read the same cached
    // value. A malformed override fails fast here rather than silently
    // falling back to the system clock.
    mindtape::init_pinned_today()?;
    cli.run()
}

fn init_color(choice: crate::render::color::ColorChoice) {
    // Eagerly resolve and store whether colour is active for this invocation.
    // The result is cached in a module-level atomic so later callers don't
    // re-check the TTY or env on every use.
    crate::render::color::init(choice);
}
