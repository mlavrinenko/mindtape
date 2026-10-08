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

// An `allow` below lets a module whose errors don't go through `crate::diag`
// yet use `clippy.toml`'s disallowed macros. Each one is an entry on the one
// list in `diag/allowlist.rs`, whose test refuses a new entry.
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
pub(crate) mod mv;
#[allow(clippy::disallowed_macros)]
pub(crate) mod preset;
pub(crate) mod recur;
#[allow(clippy::disallowed_macros)]
pub(crate) mod review;
pub(crate) mod rm;
#[allow(clippy::disallowed_macros)]
pub(crate) mod set;
pub(crate) mod views;
#[allow(clippy::disallowed_macros)]
pub(crate) mod worktrees;
