//! `MindTape` CLI entry point: parse arguments, init logging, dispatch.

// `disallowed-methods` in `clippy.toml` bans `std::fs::write`/`File::create`
// outside `mindtape::write_atomic`; unit tests (this crate's own `cfg(test)`
// build) build throwaway fixtures with the plain calls instead.
// The same goes for `disallowed-macros`: a test may print or build an
// `anyhow` error of its own.
#![cfg_attr(test, allow(clippy::disallowed_methods, clippy::disallowed_macros))]

// Every `print!`/`println!` below this point is this crate's, not the
// prelude's: a `macro_rules!` at crate root shadows a prelude macro for the
// rest of the crate, nested modules included. The prelude's pair unwraps its
// write, so `mt ls | head -3` panics out at exit 101 naming a libstd line;
// `crate::render::stdout` makes a closed reader an ordinary write error instead. That
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
    ($($arg:tt)*) => { $crate::render::stdout::write(::std::format_args!($($arg)*)) };
}

#[cfg(not(test))]
macro_rules! println {
    () => { $crate::render::stdout::write_line(::std::format_args!("")) };
    ($($arg:tt)*) => { $crate::render::stdout::write_line(::std::format_args!($($arg)*)) };
}

mod cli;
mod cmd;
mod diag;
mod git;
// An `allow` below lets a module whose errors don't go through `crate::diag`
// yet use `clippy.toml`'s disallowed macros. Each one is an entry on the one
// list in `diag/allowlist.rs`, whose test refuses a new entry.
mod pre_write;
// Infrastructure for future command migration; not every item is wired yet.
#[allow(dead_code)]
mod render;
mod task_edit;
#[cfg(test)]
mod test_util;
mod util;

use std::process::ExitCode;

use clap::Parser;

use crate::diag::Diag;

/// Run `mt` and print its error, if any, through the one renderer. Returning
/// an `ExitCode` rather than a `Result` keeps Rust's `Error:` and `Caused by:`
/// off stderr; the error's kind picks the code.
fn main() -> ExitCode {
    // Answer a completion request and exit, before anything else runs: the
    // request arrives through `$COMPLETE` rather than the argv, and stdout
    // belongs to the candidate list for its duration. Returns immediately on
    // an ordinary run.
    cli::completion::complete();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(diag) => diag::report(&diag),
    }
}

fn run() -> Result<(), Diag> {
    // Rewrite a leading `@name` into `-w <path>` before clap sees the argv, so
    // the worktree sugar reuses the existing override plumbing.
    let argv = cmd::worktrees::desugar_argv(std::env::args_os().collect())?;
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
    Ok(cli.run()?)
}

fn init_color(choice: crate::render::color::ColorChoice) {
    // Eagerly resolve and store whether colour is active for this invocation.
    // The result is cached in a module-level atomic so later callers don't
    // re-check the TTY or env on every use.
    crate::render::color::init(choice);
}
