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
//! - `allow.rs`, `config.rs`, `eval.rs`, `find.rs`, `go.rs`, `mv.rs`, `preset.rs`,
//!   `review.rs`, `set.rs`, `views.rs`, `worktrees.rs` — one file each.

pub(crate) mod add;
pub(crate) mod allow;
pub(crate) mod check;
pub(crate) mod config;
pub(crate) mod eval;
pub(crate) mod find;
pub(crate) mod flip;
pub(crate) mod go;
pub(crate) mod graph;
pub(crate) mod import;
pub(crate) mod init;
pub(crate) mod install;
pub(crate) mod link;
pub(crate) mod ls;
pub(crate) mod mv;
pub(crate) mod preset;
pub(crate) mod recur;
pub(crate) mod review;
pub(crate) mod rm;
pub(crate) mod set;
pub(crate) mod views;
pub(crate) mod worktrees;
