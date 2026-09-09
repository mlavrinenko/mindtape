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

use mindtape::{ConditionEnv, Config, Readiness};

/// The three things every `ls` renderer needs beyond the tasks themselves.
///
/// They travel together because [`columns::resolve_order`] needs all three at
/// once — the config for the column vocabulary, the readiness for the rows'
/// `ready` state, and the environment for the conditions a column's `rules`
/// evaluate — and because the table and CSV renderers must hand it the *same*
/// three or they would stop agreeing on the column set.
#[derive(Debug, Clone, Copy)]
pub struct RenderCtx<'a> {
    pub config: &'a Config,
    pub readiness: &'a Readiness,
    /// Ambient facts a column's `when` expression resolves against. Per-column
    /// [`mindtape::ColumnFacts`] are filled in by `resolve_order`, which is
    /// the only place that knows them.
    pub env: &'a ConditionEnv,
}

/// Owned backing for a [`RenderCtx`] in a test.
///
/// The context borrows its three pieces, so a test that built them inline
/// would be handing `render` references to temporaries. This holds them
/// instead; the default is the shipped floor, no readiness and no ambient
/// facts — the unconfigured-project case nearly every render test wants, and
/// the one the pre-context signature used to spell out at every call.
///
/// The floor rather than [`Config::default`], because every column's
/// visibility now lives in `defaults.toml` as a `when` expression: a config
/// that never went through `parse_config` describes no column, so it renders
/// none, and a render test built on one would assert against an empty table.
#[cfg(test)]
pub struct TestCtx {
    pub config: Config,
    pub readiness: Readiness,
    pub env: ConditionEnv,
}

#[cfg(test)]
impl Default for TestCtx {
    fn default() -> Self {
        Self {
            config: mindtape::test_support::floor_config(),
            readiness: Readiness::default(),
            env: ConditionEnv::default(),
        }
    }
}

#[cfg(test)]
impl TestCtx {
    /// A fixture pinning `config`, leaving readiness and the environment
    /// empty.
    pub fn with_config(config: Config) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    /// A fixture pinning `readiness`, leaving the config and environment empty.
    pub fn with_readiness(readiness: Readiness) -> Self {
        Self {
            readiness,
            ..Self::default()
        }
    }

    pub fn ctx(&self) -> RenderCtx<'_> {
        RenderCtx {
            config: &self.config,
            readiness: &self.readiness,
            env: &self.env,
        }
    }
}

pub mod color;
pub mod columns;
pub mod field;
pub mod fields;
pub mod file_path;
pub mod glyph;
pub mod link;
pub mod sep;
pub mod table;
pub mod width;
