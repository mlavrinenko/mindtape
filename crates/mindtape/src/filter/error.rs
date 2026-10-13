//! [`FilterError`], what parsing or evaluating a filter expression can fail with.

use thiserror::Error;

/// Errors from parsing or evaluating a filter expression.
#[derive(Debug, Error)]
pub enum FilterError {
    /// The expression failed to parse.
    #[error("failed to parse filter: {0}")]
    Parse(String),
    /// An operator that the filter language does not support.
    #[error("unsupported in filter: {0}")]
    Unsupported(String),
    /// A call or operator used wrongly, with a correct example for the
    /// `hint:` line under the message.
    #[error("{problem} (for example {example})")]
    Call {
        /// What is wrong, in plain words.
        problem: String,
        /// A correct use, as the user would write it.
        example: &'static str,
    },
    /// A field name the filter language does not know.
    #[error("unknown field in filter: {0}")]
    UnknownField(String),
    /// A status compared with `status` that no `[[tasks.status]]` entry
    /// declares, so the comparison could never match.
    #[error("unknown status in filter: '{value}'")]
    UnknownStatus {
        /// The status as written in the expression.
        value: String,
        /// The closest declared status and every declared status.
        unknown: crate::suggest::Unknown,
    },
    /// An `in_view` / `@` reference naming no declared view.
    #[error("unknown view in filter: {0}")]
    UnknownView(String),
    /// An `in_dict` reference naming no compiled dictionary — neither a
    /// `[dicts.<name>]` section nor a built-in. `unknown` carries the nearest
    /// match and every dictionary name, because a dictionary name is invisible
    /// in the listing a wrong one produces.
    #[error("unknown dict in filter: {name}")]
    UnknownDict {
        /// The name as written in the expression.
        name: String,
        /// The nearest dictionary and the dictionaries to choose from.
        unknown: crate::suggest::Unknown,
    },
    /// A `@name` path set in `changed()` or `count(files, ...)` that no
    /// dictionary answers to.
    #[error("unknown path set '@{name}'; declare it as [dicts.{name}]")]
    UnknownPathSet {
        /// The set as written, without its `@`.
        name: String,
        /// The nearest dictionary and the dictionaries to choose from.
        unknown: crate::suggest::Unknown,
    },
    /// A function the filter language does not have. `nearest` is the closest
    /// real one, when any is close.
    #[error("unknown function in filter: {name}")]
    UnknownFunction {
        /// The name as written in the expression.
        name: String,
        /// The closest function of the language, if one is near.
        nearest: Option<String>,
    },
    /// A structurally invalid expression.
    #[error("invalid filter: {0}")]
    Invalid(String),
    /// `changed()` or `touches()` asked git and git could not answer —
    /// outside a repository, or git missing. A `[[tasks.rules]]` entry
    /// reading it is skipped rather than failed ([`crate::rules`]).
    #[error("{call}() cannot read git: {reason}")]
    Git {
        /// The function that asked.
        call: &'static str,
        /// Why git could not answer.
        reason: String,
    },
}

impl FilterError {
    /// A call used wrongly: `problem` in plain words, `example` a correct use.
    pub(crate) fn call(problem: impl Into<String>, example: &'static str) -> Self {
        Self::Call {
            problem: problem.into(),
            example,
        }
    }
}
