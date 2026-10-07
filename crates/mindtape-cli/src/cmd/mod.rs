//! One module per `mt` command: the `Args` struct, its `run` function and the
//! helpers only that command uses. `cli/` wires each `Args` into the `Command`
//! enum, and `render/` holds the output code several commands share.
//!
//! # Files
//!
//! - `add/` — `mt add`, with the built-in task template.
//! - `check/` — `mt check`.
//! - `flip/` — `mt flip` and the status shortcut verbs.
//! - `init/` — `mt init`, its wizard, presets and axes.

pub(crate) mod add;
pub(crate) mod check;
pub(crate) mod flip;
pub(crate) mod init;
