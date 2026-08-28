//! The crate's stdout writer: every `print!`/`println!` in the binary lands
//! here, so a reader that walks away is a write error rather than a panic.
//!
//! The prelude's `println!` unwraps its write. Piping any `mt` command into a
//! reader that stops early — `mt ls | head -3`, a pager quit part way, an
//! `xargs` whose child failed — closes the pipe under it, and libstd (which
//! ignores `SIGPIPE` at startup, so the write returns `EPIPE` rather than
//! signalling) turns that `EPIPE` into `panicked at library/std/src/io/
//! stdio.rs: failed printing to stdout: Broken pipe (os error 32)` and exit
//! 101. That names an internal libstd line at a caller who did nothing wrong,
//! and reports a crash for what is the normal end of a pipeline.
//!
//! `main.rs` shadows both macros crate-wide (see the `macro_rules!` there) so
//! the guarantee is total rather than per-call-site: there are ~110 print sites
//! across the commands, and a policy that holds at some of them is a policy a
//! reader cannot rely on anywhere.
//!
//! ## The policy
//!
//! A broken pipe stops the *printing*, not the command. Once the reader is
//! gone every later write is dropped, and the command runs to completion and
//! exits on its own merits — `mt ls | head -1` exits 0.
//!
//! Exiting the process on the broken write was the other candidate and is
//! wrong here: `mt` is not a read-only tool. A mutating command
//! (`add`, `flip`, `rm`) prints as it goes, so bailing out at the first dropped
//! write would abort it half way — and doing that with a status of 0 would
//! claim a success that never happened, while doing it with a failing status
//! would report a failure for work that mostly landed. Running on costs a
//! rendered string nobody reads.
//!
//! Only `BrokenPipe` is swallowed. Any other write error keeps libstd's
//! behaviour — a full disk under `mt ls > out.txt` is a real failure and must
//! not be silently reported as a clean run.

// The writer's entry points are reachable only through the shadowed macros,
// which `main.rs` installs under `cfg(not(test))`. A `cargo test` build is
// therefore the one build where nothing calls them — dead by construction, not
// by oversight, so the allow is scoped to exactly that configuration rather
// than left on in the binary that ships.
#![cfg_attr(test, allow(dead_code))]

use std::fmt::Arguments;
use std::io::{self, ErrorKind, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// Set once stdout's reader is gone. Later writes are dropped rather than
/// retried: every one of them would fail the same way, and each costs a
/// syscall.
static READER_GONE: AtomicBool = AtomicBool::new(false);

/// Write `args` to stdout, as `print!` does.
pub fn write(args: Arguments) {
    emit(args, false);
}

/// Write `args` to stdout followed by a newline, as `println!` does.
pub fn write_line(args: Arguments) {
    emit(args, true);
}

/// The shared body: write, then classify a failure through [`handle`].
fn emit(args: Arguments, newline: bool) {
    if READER_GONE.load(Ordering::Relaxed) {
        return;
    }
    let mut out = io::stdout().lock();
    let written = out.write_fmt(args).and_then(|()| {
        if newline {
            out.write_all(b"\n")
        } else {
            Ok(())
        }
    });
    if let Err(err) = written {
        handle(&err);
    }
}

/// Record a broken pipe and return; re-raise anything else the way libstd's
/// `println!` would.
///
/// # Panics
/// On any write error other than [`ErrorKind::BrokenPipe`] — deliberately, to
/// keep a genuine I/O failure (a full disk under a redirect) as loud as it is
/// today.
fn handle(err: &io::Error) {
    if err.kind() == ErrorKind::BrokenPipe {
        READER_GONE.store(true, Ordering::Relaxed);
        return;
    }
    panic!("failed printing to stdout: {err}");
}

#[cfg(test)]
#[path = "stdout_tests.rs"]
mod tests;
