//! What a config layer that fails to read, parse or validate reports.
//!
//! For the TOML cases [`ConfigError`] carries the message rather than a
//! `source`, so a caller printing the chain prints it once.
//! [`crate::config`] re-exports it, and the CLI's `diag::from_lib` matches
//! every variant to pick a kind and words.

use thiserror::Error;

use crate::config::ConfigPlace;
use crate::problem::Problem;
use crate::suggest::Unknown;

/// Errors from loading or locating a config file.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The config file failed to read.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// The config file contained invalid TOML. The TOML error is the message,
    /// not a source, so a caller printing the chain prints it once.
    #[error("parse error: {0}")]
    Parse(toml::de::Error),
    /// A config layer or pack other than the project's own contained invalid
    /// TOML. `origin` names the file, or `pack <name>` for an embedded pack.
    #[error("parse error in {origin}: {error}")]
    ParseIn {
        /// The file path, or `pack <name>`, the TOML error is in.
        origin: String,
        /// The TOML error, in the message rather than a `source` link, which
        /// lets a chain print it once.
        error: toml::de::Error,
    },
    /// The config parsed but failed a semantic check (such as band coverage).
    #[error("invalid config: {message}")]
    Invalid {
        /// The problems, one per line, each led by the dotted key it is about.
        message: String,
        /// Where the first problem's key sits, when a config file declares it.
        place: Option<ConfigPlace>,
        /// What the first problem's unknown name could have been, if it has
        /// one.
        unknown: Option<Unknown>,
        /// What to try next, when the first problem has an obvious next step.
        hint: Option<&'static str>,
    },
}

impl ConfigError {
    /// A semantic failure not yet tied to a file.
    #[must_use]
    pub fn invalid(problem: impl Into<Problem>) -> Self {
        let problem = problem.into();
        Self::Invalid {
            message: problem.text,
            place: None,
            unknown: problem.unknown,
            hint: None,
        }
    }

    /// `self` with `next` as what to try next, when it is
    /// [`ConfigError::Invalid`].
    #[must_use]
    pub fn with_hint(mut self, next: &'static str) -> Self {
        if let Self::Invalid { hint, .. } = &mut self {
            *hint = Some(next);
        }
        self
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        Self::Parse(error)
    }
}
