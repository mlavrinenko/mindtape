//! `MindTape` CLI entry point: parse arguments, init logging, dispatch.

// `disallowed-methods` in `clippy.toml` bans `std::fs::write`/`File::create`
// outside `mindtape::write_atomic`; unit tests (this crate's own `cfg(test)`
// build) build throwaway fixtures with the plain calls instead.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod stdout;

// Every `print!`/`println!` below this point is this crate's, not the
// prelude's: a `macro_rules!` at crate root shadows a prelude macro for the
// rest of the crate, nested modules included. The prelude's pair unwraps its
// write, so `mt ls | head -3` panics out at exit 101 naming a libstd line;
// `crate::stdout` makes a closed reader an ordinary write error instead. That
// module's docs carry the policy and why it is enforced here rather than at the
// ~110 call sites.
//
// The shadow is textual, so it only reaches code declared AFTER it — hence its
// place above every `mod`, ahead of the alphabetical block that rustfmt owns
// (`reorder_modules` would sort a `mod` declaration into that block; a
// `macro_rules!` item between the two groups is what keeps `mod stdout;` out of
// it).
//
// `cfg(not(test))` leaves the prelude's macros in place under `cargo test`:
// libtest captures a test's output through `print!`'s internals, which a
// direct `io::stdout()` write bypasses, so shadowing them there would spill
// every in-process command's output over the test report. The binary an e2e
// test spawns is built without `cfg(test)` and gets the real thing.
#[cfg(not(test))]
macro_rules! print {
    ($($arg:tt)*) => { $crate::stdout::write(::std::format_args!($($arg)*)) };
}

#[cfg(not(test))]
macro_rules! println {
    () => { $crate::stdout::write_line(::std::format_args!("")) };
    ($($arg:tt)*) => { $crate::stdout::write_line(::std::format_args!($($arg)*)) };
}

mod add;
mod add_dedup;
mod add_interactive;
mod add_output;
mod add_tokens;
mod add_values;
mod breach;
mod check;
mod check_commit;
mod check_range;
mod check_templates;
mod cli;
mod cli_check;
mod cli_dump;
mod command_class;
mod completion;
mod config_cmd;
mod eval;
mod find;
mod flip;
mod flip_batch;
mod flip_hooks;
mod flip_hooks_cmd;
mod git;
mod go;
mod graph;
mod graph_render;
mod import;
mod init;
mod init_compose;
mod init_wizard;
mod install;
mod install_hint;
mod link_cmd;
mod link_edit;
mod link_input;
mod link_plan;
mod ls;
mod ls_interactive;
mod ls_rules;
mod ls_rules_applied;
mod mv;
mod output;
mod pre_write;
mod preset;
mod preset_cmd;
mod recur;
mod recur_edit;
// Infrastructure for future command migration; not every item is wired yet.
#[allow(dead_code)]
mod render;
mod review;
mod rm;
mod set;
mod task_edit;
mod task_ref;
#[cfg(test)]
mod test_util;
mod util;
mod views;
mod worktrees;
mod yaml_values;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    // Answer a completion request and exit, before anything else runs: the
    // request arrives through `$COMPLETE` rather than the argv, and stdout
    // belongs to the candidate list for its duration. Returns immediately on
    // an ordinary run.
    completion::complete();
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
