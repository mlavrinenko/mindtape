//! One module per `mt` command: the `Args` struct, its `run` function and the
//! helpers only that command uses. `cli/` wires each `Args` into the `Command`
//! enum, and `render/` holds the output code several commands share.
//!
//! A command that outgrows one file becomes a folder with a `mod.rs`.
//!
//! # Files
//!
//! - `add/`, `check/`, `flip/`, `init/` — `mt add`, `mt check`, `mt flip` (with
//!   the status shortcut verbs) and `mt init` (with its wizard, presets, axes).
//! - `graph/`, `import/`, `install/`, `link/`, `ls/`, `recur/`, `rm/` — the
//!   commands split over helper files.
//! - `config.rs`, `eval.rs`, `find.rs`, `go.rs`, `mv.rs`, `preset.rs`,
//!   `review.rs`, `set.rs`, `views.rs`, `worktrees.rs` — one file each.

// Modules whose errors don't go through `crate::diag` yet, so `clippy.toml`'s
// `disallowed-macros` (`eprintln!`, `bail!`, `anyhow!`) is allowed in them.
// The list only shrinks: each fan-out task of
// `truth/tasks/build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ`
// converts its files and drops its module here, and
// `truth/tasks/drop-anyhow-from-the-mt-binary-so-every-error-is-a-diag.typ`
// removes the last entry. The other half of the list is in `main.rs`.
#[allow(clippy::disallowed_macros)]
pub(crate) mod add;
#[allow(clippy::disallowed_macros)]
pub(crate) mod check;
pub(crate) mod config;
#[allow(clippy::disallowed_macros)]
pub(crate) mod eval;
pub(crate) mod find;
pub(crate) mod flip;
#[allow(clippy::disallowed_macros)]
pub(crate) mod go;
pub(crate) mod graph;
#[allow(clippy::disallowed_macros)]
pub(crate) mod import;
#[allow(clippy::disallowed_macros)]
pub(crate) mod init;
pub(crate) mod install;
#[allow(clippy::disallowed_macros)]
pub(crate) mod link;
#[allow(clippy::disallowed_macros)]
pub(crate) mod ls;
#[allow(clippy::disallowed_macros)]
pub(crate) mod mv;
#[allow(clippy::disallowed_macros)]
pub(crate) mod preset;
#[allow(clippy::disallowed_macros)]
pub(crate) mod recur;
#[allow(clippy::disallowed_macros)]
pub(crate) mod review;
#[allow(clippy::disallowed_macros)]
pub(crate) mod rm;
#[allow(clippy::disallowed_macros)]
pub(crate) mod set;
pub(crate) mod views;
#[allow(clippy::disallowed_macros)]
pub(crate) mod worktrees;
