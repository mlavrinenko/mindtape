//! How loudly a config-declared breach reports: `warn` or `error`.
//!
//! Shared by every config section that lets a project choose a level:
//! `[[tasks.rules]]` (see [`crate::rules`]), `[[commits.rules]]` and
//! `[tasks.comments]`. All spell the same two words in TOML and mean the same
//! two things by them, so they read one enum rather than several that would
//! have to be kept in step by hand.
//!
//! Every [`crate::Violation`] carries one. A malformed artifact — a status
//! outside the legend, an unresolvable edge, a value off its scale — is always
//! an error, because it is not a matter of degree. A choice of level exists for
//! the constraints a project declares, where "report it and carry on" is a
//! legitimate answer.

use serde::{Deserialize, Serialize};

/// How a breached config-declared constraint reports.
#[derive(
    Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Print the breach; the command still succeeds. The default.
    #[default]
    Warn,
    /// Print the breach and fail the command.
    Error,
}

impl Severity {
    /// Whether a breach at this severity fails the command.
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    /// The word a config writes for this severity: `warn` or `error`.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
