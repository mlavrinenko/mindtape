//! Terminal-output discipline for `mindtape-cli`.
//!
//! This module owns all decisions about terminal capabilities so that
//! individual command modules never re-decide colour, glyph style, or
//! separator policy on their own.
//!
//! # Submodules
//!
//! - [`color`] — colour decision honouring `--color`, `NO_COLOR`, and TTY
//!   detection.
//! - [`glyph`] — emoji vs. ASCII vs. no-glyph mode, with TTY-aware resolution.
//! - [`width`] — Unicode display-width-aware truncation and padding helpers.
//! - [`sep`] — stable separator policy for machine-readable output.
//! - [`field`] — `Field` trait and generic cell rendering seam.
//!
//! The submodules expose infrastructure that will be consumed by command modules
//! as each command is migrated; `dead_code` is not suppressed here — see
//! the `#[allow(dead_code)]` on the `mod render` declaration in `main.rs`.

pub mod color;
pub mod field;
pub mod glyph;
pub mod sep;
pub mod width;
